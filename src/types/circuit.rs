use macroquad::prelude::*;
use crate::types::gate::*;
use crate::types::gate_type::*;
use crate::types::pins::*;
use crate::types::pin_type::*;
use crate::utils::*;

pub struct Circuit {
    pub emulation_done: bool,
    pub wires_read: Vec<bool>,
    pub wires_write: Vec<bool>,
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
        };
    }
    // make it so a 'NOT' gate automatically makes the wire yellow.
    // pub fn set_wire(&mut self, wire_index: usize, value: bool) {
    //     if !(0..self.wires_read.len()).contains(&(wire_index)) {
    //         panic!("invalid wire_index {} for wires_read", wire_index)
    //     }
    //     self.wires_read[wire_index] = value;
    //     self.wires_write[wire_index] = value;
    // }

    // pub fn get_wire(&mut self, wire_index: usize) -> bool {
    //     if !(0..self.wires_read.len()).contains(&(wire_index)) {
    //         panic!("invalid wire_index {} for wires_read", wire_index)
    //     }
    //     return self.wires_read[wire_index];
    // }

    pub fn evaluate(&self, gate: &Gate) -> bool {
        // each wire sample should be false if theres no wire.
        let get_pin = |index: usize| -> bool {
            gate.input[index].wire_index
                .map(|idx| self.wires_read[idx]) // Transform index -> value
                .unwrap_or(false)                // Handle None -> false
        };

        match (&gate.gate_type, gate.input.len()) {
            (GateType::NOT, 1) => {
                !get_pin(0)
            }
            (GateType::OR, 2) => {
                get_pin(0) | get_pin(1)
            }
            (GateType::XOR, 2) => {
                get_pin(0) ^ get_pin(1)
            }
            (GateType::XNOR, 2) => {
                !(get_pin(0) ^ get_pin(1))
            }
            (GateType::NOR, 2) => {
                !(get_pin(0) | get_pin(1))
            }
            (GateType::AND, 2) => {
                get_pin(0) & get_pin(1)
            }
            (GateType::NAND, 2) => {
                !(get_pin(0) & get_pin(1))
            }
            (GateType::PWR, 0) => true,
            // Assuming GND input might be connected or floating, but output is always false
            (GateType::GND, _) => false, 
            _ => panic!("Unsupported gate type or input configuration"),
        }
    }

    pub fn connect_wire(
        &mut self,
        output_gate_index: usize,
        input_gate_index: usize,
        output_pin_index: usize,
        input_pin_index: usize,
    ) {

        {
            if input_gate_index > self.gates.len() {
                panic!("invalid input_gate_index {} for self.gates of length {}", input_pin_index, self.gates.len());
            } 
            if output_gate_index > self.gates.len() {
                panic!("invalid output_gate_index {} for self.gates of length {}", output_pin_index, self.gates.len());
            }

            let output_gate = &self.gates[output_gate_index];
            let input_gate = &self.gates[input_gate_index];

            if input_pin_index > input_gate.input.len() {
                panic!("invalid input_pin_index {} for input_gate of length {}", input_pin_index, input_gate.input.len());
            } 
            if output_pin_index > output_gate.output.len() {
                panic!("invalid output_pin_index {} for output_gate of length {}", output_pin_index, output_gate.output.len());
            }
        }

        self.wires_read.push(false);
        self.wires_write.push(false); // to make them equal in length so no problems when swapping
        // wire goes from 'output_gate' to 'input_gate'
        let wire_index = self.wires_read.len() - 1;
        // connect output to wire
        self.gates[output_gate_index].output[output_pin_index].other_gate_index = Some(input_gate_index);
        self.gates[output_gate_index].output[output_pin_index].wire_index = Some(wire_index);
        // connect input to wire
        self.gates[input_gate_index].input[input_pin_index].other_gate_index = Some(output_gate_index);
        self.gates[input_gate_index].input[input_pin_index].wire_index = Some(wire_index);
    }

    pub fn tick(&mut self) {
        // check for same length so no problems when swapping
        if self.wires_read.len() != self.wires_write.len() {
            panic!("wire buffers are not of the same length");
        }

        let mut changed_wires = vec![false; self.wires_read.len()];
        // read
        for gate in &self.gates {
            let result = self.evaluate(gate);
            // only do outputs by the first bit for now
            let output_wire_index = gate.output[0].wire_index;

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

    pub fn add_gate(&mut self, gate : Gate) -> usize {
        // not necessarily in topological order
        let index = self.gates.len();
        self.gates.push(gate);
        return index;
    }

    pub fn draw_gates(&self, camera : &Camera2D) {
        set_camera(camera);

        for gate in &self.gates {
            let camera_view_rect = camera_view_rect(&camera);
            let rect = gate.rect;

            if intersects(rect, camera_view_rect) {        
                let color = GateType::color(&gate.gate_type);
                let text = GateType::text(&gate.gate_type);
                
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
                let dims = measure_text(text, None, 32, 1.0);
                let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
                let ty = rect.y + rect.h * 0.35 + dims.height * 0.5; 

                draw_text_ex(
                    text, 
                    tx, 
                    ty, 
                    TextParams {
                        font_size: 32.0 as u16,
                        font_scale: -1.0,
                        font_scale_aspect: -1.0, 
                        color: BLACK,
                        ..Default::default()
                    }
                );
            }

            // draw wires
            // to cut down on the processing each frame, it would be good to have the blocks be part of the gate, as these never change.
            let pin_blocks = &gate.clone().get_pins_blocks(); // this is kinda scuffed
            // only draw output -> input wires, otherwise double draw calls
            // let spatial_input_pin_blocks = gate.get_side_pins_blocks(PinType::Input);
            let spatial_output_pin_blocks = gate.get_side_pins_blocks(PinType::Output);
           
            for (current_pin_index, current_pin_block) in spatial_output_pin_blocks.iter().enumerate() {
                let other_gate_index = gate.output[current_pin_index].other_gate_index;

                match other_gate_index {
                    Some(index) => {
                        let spatial_input_pin_blocks = self.gates[index].get_side_pins_blocks(PinType::Input);
                        let other_pin_block = spatial_input_pin_blocks[current_pin_index].clone();

                        let Vec2 {x: output_center_x, y: output_center_y} = current_pin_block.rect.center();
                        let Vec2 {x: input_center_x, y: input_center_y} = other_pin_block.rect.center();

                        draw_line(output_center_x, output_center_y, input_center_x, input_center_y, 3.0, BLACK);
                    }
                    None => {}
                }
            }
           
            // draw blocks
            for pin_block in pin_blocks {
                let pin_rect = pin_block.rect;
                if intersects(pin_rect, camera_view_rect) {
                    draw_rectangle(pin_rect.x, pin_rect.y, pin_rect.w, pin_rect.h, BLACK);
                }
            }
        }
    }
}