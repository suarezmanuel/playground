use crate::types::gate::*;
use crate::types::gate_type::*;
use crate::types::pin_type::*;
use crate::utils::*;
use crate::types::wires::*;
use macroquad::prelude::*;
use crate::types::pins::*;
use crate::types::gate::*;

const FONT_SIZE: u16 = 32;

pub struct Circuit {
    pub emulation_done: bool,
    pub wires_read: Vec<bool>,
    pub wires_write: Vec<bool>,
    pub wires_freed: Vec<bool>,
    pub wires_meta: Vec<Wire>,
    pub gates: Vec<Gate>,
}

impl Circuit {
    pub fn new() -> Circuit {
        // first wire acts as input (later generalize to handle multiple inputs e.g. binary numbers)
        return Circuit {
            emulation_done: false,
            wires_read: vec![],
            wires_write: vec![],
            gates: Vec::new(),
            wires_freed: vec![], // if wires_read gets really big and then all the wires are deleted, wires_freed will be wasted memory. a compression algo is needed
            wires_meta: vec![]
        };
    }

    pub fn evaluate(&self, gate: &Gate) -> bool {
        // each wire sample should be false if theres no wire.
        let get_pin = |index: usize| -> bool {
            gate.input[index]
                .wire_index
                .map(|idx| self.wires_read[idx]) // Transform index -> value
                .unwrap_or(false) // Handle None -> false
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

    pub fn new_wire(&mut self) -> usize {
        for (index, value) in self.wires_freed.iter().enumerate() {
            if *value == true {
                self.wires_freed[index] = false;
                self.wires_read.insert(index, false);
                self.wires_write.insert(index, false);
                self.wires_meta.insert(index, Wire{source: Connection{ pin_index: 0, gate_index: 0}, connections: vec![], wire_index: self.wires_read.len() - 1});
                return index;
            }
        }
        self.wires_read.push(false);
        self.wires_write.push(false); // to make them equal in length so no problems when swapping
        self.wires_freed.push(false);
        self.wires_meta.push(Wire{source: Connection{ pin_index: 0, gate_index: 0}, connections: vec![], wire_index: self.wires_read.len() - 1});
        return self.wires_read.len() - 1;
    }

    pub fn remove_wire(&mut self, index: usize) {
        if self.wires_freed[index] == true {
            panic!("double free of wire at index {index}");
        }
        self.wires_freed[index] = true;
        // remove value from wires_meta
        // this makes sure its not drawn anymore
        let wire = &self.wires_meta[index];
        let source_gate_index = wire.source.gate_index;
        let source_pin_index = wire.source.pin_index;
        self.gates[source_gate_index].output[source_pin_index].wire_index = None;
        
        for connection in wire.connections.iter() {
            self.gates[connection.gate_index].input[connection.pin_index].wire_index = None;
        }

        self.wires_meta[index].connections = vec![];

    }

    pub fn connect_wire(
        &mut self,
        from_gate_index: usize,
        to_gate_index: usize,
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
        let input_gate_index: usize;
        let output_gate_index: usize;

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

        let input_pin = &mut self.gates[input_gate_index].get_pin(input_pin_index, input_pin_type);
        let output_pin = &mut self.gates[output_gate_index].get_pin(output_pin_index, output_pin_type);


        match output_pin.wire_index {
            Some(wire_index) => {
                // check that they aren't connected already
                let connected = self.wires_meta[wire_index].connections.find_pin_index(input_gate_index, input_pin.index).is_some();

                if !connected {
                    // if input_pin is part of another wire
                    if input_pin.wire_index.is_some() { // input_pin can only be in 'connections'
                        // if other wire only connects to input_pin, remove wire
                        if self.wires_meta[input_pin.wire_index.unwrap()].connections.len() == 1 {
                            self.remove_wire(input_pin.wire_index.unwrap());
                        } else {
                            // if other wire has more connections, remove input_pin from connections 
                            let index_to_remove = self.wires_meta[input_pin.wire_index.unwrap()].connections.find_pin_index(input_gate_index, input_pin.index);
                            if index_to_remove.is_some() { self.wires_meta[input_pin.wire_index.unwrap()].connections.remove(index_to_remove.unwrap()); }
                        }
                    }
                    
                    self.wires_meta[wire_index].connections.push(Connection{pin_index: input_pin.index, gate_index: input_gate_index});
                    // input_pin.wire_index = Some(wire_index); 
                    self.gates[input_gate_index].input[input_pin_index].wire_index = Some(wire_index); // fixed
                }
                // if connected don't do anything
            }
            None => {
                
                // if input_pin already has a wire, remove it from other wire
                if input_pin.wire_index.is_some() {
                    if self.wires_meta[input_pin.wire_index.unwrap()].connections.len() == 1 {
                        self.remove_wire(input_pin.wire_index.unwrap());
                    } else {
                        let index_to_remove = self.wires_meta[input_pin.wire_index.unwrap()].connections.find_pin_index(input_gate_index, input_pin.index);
                        if index_to_remove.is_some() { self.wires_meta[input_pin.wire_index.unwrap()].connections.remove(index_to_remove.unwrap()); }
                    }
                }

                let new_wire_index = self.new_wire();
                let connection = &mut self.wires_meta[new_wire_index];
                connection.source = Connection{pin_index: output_pin.index, gate_index: output_gate_index };
                connection.connections = vec![Connection{pin_index: input_pin.index, gate_index: input_gate_index}];
                {
                    let gate = &mut self.gates[input_gate_index];
                    gate.input[input_pin_index].wire_index = Some(new_wire_index);
                    // input_pin.wire_index = Some(new_wire_index);
                }

                {
                    let gate = &mut self.gates[output_gate_index];
                    gate.output[output_pin_index].wire_index = Some(new_wire_index);
                }
            }
        }
    }

    pub fn reset_wires(&mut self) {
        for index in 0..self.wires_read.len() {
            self.wires_read[index] = false;
            self.wires_write[index] = false;
        }
    }

    pub fn tick(&mut self) {
        // check for same length so no problems when swapping
        if self.wires_read.len() != self.wires_write.len() {
            panic!("wire buffers are not of the same length");
        }

        let mut changed_wires = vec![false; self.wires_read.len()];
        // read
        for gate in &self.gates {

            for output in &gate.output {
                let result = self.evaluate(gate);
                // only do outputs by the first bit for now
                let output_wire_index = output.wire_index;

                match output_wire_index {
                    Some(index) => {
                        if changed_wires[index] && self.wires_write[index] == !result {
                            panic!("short circuit on wire {}", index);
                        }
                        self.wires_write[index] = result;
                        changed_wires[index] = true;
                    }
                    None => {
                        // dont write to a wire if there's no connected wire
                    }
                }
            }
        }

        // check if emulation is done (add output test in future)
        self.emulation_done = true;
        for (index, value) in self.wires_read.iter().enumerate() {
            if *value != self.wires_write[index] {
                self.emulation_done = false;
            }
        }
        // write
        std::mem::swap(&mut self.wires_read, &mut self.wires_write);
    }

    pub fn add_gate(&mut self, gate: Gate) -> usize {
        // not necessarily in topological order
        let index = self.gates.len();
        self.gates.push(gate);
        return index;
    }

    pub fn draw_gates(&self, camera: &Camera2D) {
        set_camera(camera);

        for gate in &self.gates {
            let camera_view_rect = camera_view_rect(&camera);
            let rect = gate.rect;

            if intersects(rect, camera_view_rect) {
                let color = GateType::color(&gate.gate_type);
                let text = GateType::text(&gate.gate_type);

                draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
                let dims = measure_text(text, None, FONT_SIZE, 1.0);
                let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
                let ty = rect.y + rect.h * 0.5 + dims.offset_y * 0.5 as f32;
                
                draw_text_ex(
                    text,
                    tx,
                    ty,
                    TextParams {
                        font_size: FONT_SIZE,
                        color: BLACK,
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn draw_wires(&self, camera: &Camera2D) {
        set_camera(camera);

        for wire in &self.wires_meta {
            let Vec2 { x: start_x, y: start_y } = self.gates[wire.source.gate_index].get_pin_rect(wire.source.pin_index, PinType::Output).center();

            for connection in &wire.connections {
                // println!("connection pin index: {}", connection.pin_index);
                let Vec2 { x: end_x, y: end_y } = self.gates[connection.gate_index].get_pin_rect(connection.pin_index, PinType::Input).center();
            
                let color = match self.wires_read[wire.wire_index] {
                    true => { YELLOW }
                    false => { BLACK }
                };

                draw_line(start_x, start_y, end_x, end_y, 3.0, color);
            }
        }
    }

    pub fn draw_pins(&self, camera: &Camera2D) {
        set_camera(camera);

        for gate in &self.gates {
            let camera_view_rect = camera_view_rect(&camera);

            let pins= gate.input.iter().chain(gate.output.iter());

            for pin in pins {
                let pin_rect = pin.rect;
                if intersects(pin_rect, camera_view_rect) {
                    draw_rectangle(pin_rect.x, pin_rect.y, pin_rect.w, pin_rect.h, BLACK);
                }
            }
        }
    }

    pub fn draw_mouse_wire(
        &self,
        camera: &Camera2D,
        gate_index: Option<usize>,
        pin_index: Option<usize>,
        pin_type: Option<PinType>,
    ) {
        match (gate_index, pin_index, pin_type) {
            (Some(gate_index), Some(pin_index), Some(pin_type)) => {
                let gate = self.gates[gate_index].clone(); // this is fine because we only read and don't write
                let rect = gate.get_pin(pin_index, pin_type).rect;
                let Vec2 {
                    x: center_x,
                    y: center_y,
                } = rect.center();
                let mouse_world =
                    camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                draw_line(center_x, center_y, mouse_world.x, mouse_world.y, 3.0, BLACK);
            }
            _ => {}
        };
    }

    pub fn draw_gate_over_mouse(
        &self,
        camera: &Camera2D,
        rect: Rect,
        gate_type: &GateType,
        alpha: f32,
    ) {
        // just to be sure
        if intersects(rect, camera_view_rect(camera)) {
            let color = gate_type.color();
            let text = gate_type.text();

            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color.with_alpha(0.5));
            let dims = measure_text(text, None, FONT_SIZE, 1.0);
            let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
            let ty = rect.y + rect.h * 0.5 + FONT_SIZE as f32 / 4.0;

            draw_text_ex(
                text,
                tx,
                ty,
                TextParams {
                    font_size: FONT_SIZE,
                    color: BLACK.with_alpha(alpha),
                    ..Default::default()
                },
            );
        }
    }
}
