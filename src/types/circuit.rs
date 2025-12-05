use crate::types::gate::*;
use crate::types::gate_type::*;
use crate::types::pin_type::*;
use crate::types::wires::*;
use crate::types::keys::*;
use macroquad::prelude::*;
use slotmap::{SlotMap, SecondaryMap};

pub struct Circuit {
    pub emulation_done: bool,
    pub wires: SlotMap<WireKey, Wire>,
    pub wires_read: SecondaryMap<WireKey, bool>,
    pub wires_write: SecondaryMap<WireKey, bool>,
    pub gates: SlotMap<GateKey, Gate>,
}

impl Circuit {
    pub fn new() -> Circuit {
        // first wire acts as input (later generalize to handle multiple inputs e.g. binary numbers)
        return Circuit {
            emulation_done: false,
            wires: SlotMap::with_key(),
            wires_read: SecondaryMap::new(),
            wires_write: SecondaryMap::new(),
            gates: SlotMap::with_key(),
        };
    }

    pub fn evaluate(&self, gate: &Gate) -> bool {
        // each wire sample should be false if theres no wire.
        let get_pin = |index: usize| -> bool {
            gate.input[index]
                .wire_index
                .and_then(|index| self.wires_read.get(index))
                .copied()
                .unwrap_or(false)
        };

        match (&gate.gate_type, gate.input.len()) {
            (GateType::NOT, 1) => !get_pin(0),
            (GateType::OR, 2) => get_pin(0) | get_pin(1),
            (GateType::XOR, 2) => get_pin(0) ^ get_pin(1),
            (GateType::XNOR, 2) => !(get_pin(0) ^ get_pin(1)),
            (GateType::NOR, 2) => !(get_pin(0) | get_pin(1)),
            (GateType::AND, 2) => get_pin(0) & get_pin(1),
            (GateType::NAND, 2) => !(get_pin(0) & get_pin(1)),
            (GateType::PWR, 0) => true,
            // Assuming GND input might be connected or floating, but output is always false
            (GateType::GND, _) => false,
            _ => panic!("Unsupported gate type or input configuration"),
        }
    }

    pub fn connect_wire(
        &mut self,
        from_gate_index: GateKey,
        to_gate_index: GateKey,
        from_pin_index: usize,
        from_pin_type: PinType,
        to_pin_index: usize,
        to_pin_type: PinType,
    ) {
        if from_pin_type.to_string() == to_pin_type.to_string() {
            panic!("there shouldn't be a cable between pins of the same type");
        }

        // from_pin, to_pin are general pins, to actually know the connection direction we need the types
        let input_pin_index: usize;
        let input_pin_type: PinType;
        let output_pin_index: usize;
        let output_pin_type: PinType;
        let input_gate_index: GateKey;
        let output_gate_index: GateKey;

        match from_pin_type {
            PinType::Input => {
                input_pin_index = from_pin_index;
                input_pin_type = from_pin_type;
                input_gate_index = from_gate_index;
                output_pin_index = to_pin_index;
                output_pin_type = to_pin_type;
                output_gate_index = to_gate_index;
            }
            PinType::Output => {
                input_pin_index = to_pin_index;
                input_pin_type = to_pin_type;
                input_gate_index = to_gate_index;
                output_pin_index = from_pin_index;
                output_pin_type = from_pin_type;
                output_gate_index = from_gate_index;
            }
        }

        if let Some(output_gate) = &mut self.gates.get(output_gate_index) {
            let output_pin = output_gate.get_pin(output_pin_index, output_pin_type);
            if let Some(input_gate) = &mut self.gates.get(input_gate_index) {
                let input_pin = input_gate.get_pin(input_pin_index, input_pin_type);

                match output_pin.wire_index {
                    Some(wire_index) => {
                        // check that they aren't connected already
                        let connected = self.wires.get(wire_index).unwrap()
                            .connections
                            .find_pin_index(input_gate_index, input_pin.index)
                            .is_some();

                        if !connected {
                            // if input_pin is part of another wire
                            if let Some(wire_index) = input_pin.wire_index {
                                // input_pin can only be in 'connections'
                                // if other wire only connects to input_pin, remove wire
                                if self.wires.get(wire_index).unwrap()
                                    .connections
                                    .len()
                                    == 1
                                {
                                    self.wires.remove(wire_index);
                                } else {
                                    // if other wire has more connections, remove input_pin from connections
                                    let index_to_remove = self.wires.get(wire_index).unwrap()
                                    .connections
                                    .find_pin_index(input_gate_index, input_pin.index);
                                    if index_to_remove.is_some() {
                                        self.wires.get_mut(wire_index).unwrap()
                                            .connections
                                            .remove(index_to_remove.unwrap());
                                    }
                                }
                            }

                            self.wires.get_mut(wire_index).unwrap().connections.push(Connection {
                                pin_index: input_pin.index,
                                gate_index: input_gate_index,
                            });
                            // input_pin.wire_index = Some(wire_index);
                            if let Some(gate) = self.gates.get_mut(input_gate_index).as_mut() {
                                gate.input[input_pin_index].wire_index = Some(wire_index); // fixed
                            }
                        }
                        // if connected don't do anything
                    }
                    None => {
                        // if input_pin already has a wire, remove it from other wire
                        if let Some(wire_index) = input_pin.wire_index {
                            if self.wires.get(wire_index).unwrap().connections.len() == 1 {
                                self.wires.remove(wire_index);
                            } else {
                                // usize is correct, only wire itself references connections
                                let index_to_remove = self.wires.get(wire_index).unwrap()
                                .connections
                                .find_pin_index(input_gate_index, input_pin.index).unwrap();
                                self.wires.get_mut(wire_index).unwrap()
                                    .connections
                                    .remove(index_to_remove);
                            }
                        }

                        let wire_index = self.new_wire(Wire {
                            source: Connection {
                                pin_index: output_pin.index,
                                gate_index: output_gate_index,
                            },
                            connections: vec![Connection {
                                pin_index: input_pin.index,
                                gate_index: input_gate_index,
                            }],
                        });

                        let connection = &mut self.wires.get_mut(wire_index).unwrap();
                        connection.source = Connection {
                            pin_index: output_pin.index,
                            gate_index: output_gate_index,
                        };
                        connection.connections = vec![Connection {
                            pin_index: input_pin.index,
                            gate_index: input_gate_index,
                        }];
                        {
                            let gate = &mut self.gates.get_mut(input_gate_index).unwrap();
                            gate.input[input_pin_index].wire_index = Some(wire_index);
                        }

                        {
                            let gate = &mut self.gates.get_mut(output_gate_index).unwrap();
                            gate.output[output_pin_index].wire_index = Some(wire_index);
                        }
                    }
                }
            }
        }
    }

    pub fn reset_wires(&mut self) {
        if self.wires_read.len() != self.wires_write.len() {
            panic!("wires_read, wires_write not of the same length")
        }
        for (_, value) in self.wires_read.iter_mut() {
            *value = false;
        }
        for (_, value) in self.wires_write.iter_mut() {
            *value = false;
        }
    }

    pub fn new_wire(&mut self, wire: Wire) -> WireKey {
        let key = self.wires.insert(wire);
        self.wires_read.insert(key, false);
        self.wires_write.insert(key, false);
        return key;
    }

     pub fn remove_wire(&mut self, key: WireKey) {
        // Step A: Get the topology data before deleting
        // We need to know who was connected to this wire
        let (source, destinations) = if let Some(wire) = self.wires.get(key) {
            (wire.source.clone(), wire.connections.clone())
        } else {
            return; // Wire doesn't exist, nothing to do
        };

        // Step B: Clear the Source Pin (Output)
        if let Some(gate) = self.gates.get_mut(source.gate_index) {
            // Check if it's actually pointing to this wire before clearing
            // (It might have moved to a new wire already)
            if gate.output[source.pin_index].wire_index == Some(key) {
                gate.output[source.pin_index].wire_index = None;
            }
        }

        // Step C: Clear the Destination Pins (Inputs)
        for dest in destinations {
            if let Some(gate) = self.gates.get_mut(dest.gate_index) {
                if gate.input[dest.pin_index].wire_index == Some(key) {
                    gate.input[dest.pin_index].wire_index = None;
                }
            }
        }

        // Step D: Actually delete the data
        self.wires.remove(key);
        self.wires_read.remove(key);
        self.wires_write.remove(key);
    }

    pub fn tick(&mut self) {
        // check for same length so no problems when swapping
        if self.wires_read.len() != self.wires_write.len() {
            panic!("wire buffers are not of the same length");
        }

        let mut changed_wires: SlotMap<WireKey, bool> = SlotMap::with_key();
        // read
        for (_, gate) in &self.gates {
            for output in &gate.output {
                let result = self.evaluate(gate);
                // only do outputs by the first bit for now
                let output_wire_index = output.wire_index;

                match output_wire_index {
                    Some(index) => {
                        if changed_wires.get(index).is_some() && *self.wires_write.get(index).unwrap() == !result {
                            panic!("short circuit on wire");
                        }
                        *self.wires_write.get_mut(index).unwrap() = result;
                        changed_wires.insert(true);
                    }
                    None => {
                        // dont write to a wire if there's no connected wire
                    }
                }
            }
        }

        // check if emulation is done (add output test in future)
        self.emulation_done = true;
        for (index, value) in self.wires_read.iter() {
            if *value != self.wires_write[index] {
                self.emulation_done = false;
            }
        }
        // write
        std::mem::swap(&mut self.wires_read, &mut self.wires_write);
    }
}