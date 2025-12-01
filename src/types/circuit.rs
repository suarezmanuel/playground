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
    pub wires_freed: Vec<bool>,
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
            wires_freed: vec![] // if wires_read gets really big and then all the wires are deleted, wires_freed will be wasted memory. a compression algo is needed
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

    pub fn add_wire(&mut self) -> usize {
        for (index, value) in self.wires_freed.iter().enumerate() {
            if *value == true {
                self.wires_freed[index] = false;
                self.wires_read.insert(index, false);
                self.wires_write.insert(index, false);
                return index;
            }
        }
        self.wires_read.push(false);
        self.wires_write.push(false); // to make them equal in length so no problems when swapping
        self.wires_freed.push(false);
        return self.wires_read.len()-1;
    }

    pub fn remove_wire(&mut self, index: usize) {
        if self.wires_freed[index] == true {
            panic!("double free of wire at index {index}");
        }
        self.wires_freed[index] = true;
        self.wires_read.remove(index);
        self.wires_write.remove(index);
    }

    pub fn connect_wire(
        &mut self,
        from_gate_index: usize,
        to_gate_index: usize,
        from_pin_index: usize,
        from_pin_type: PinType,
        to_pin_index: usize,
        to_pin_type: PinType
    ) {

        let wire_index = self.add_wire();

        {
            let gate = &mut self.gates[from_gate_index];

            let pins_list= &mut match from_pin_type {
                PinType::Input => { &mut gate.input }
                PinType::Output => { &mut gate.output }
            };

            let pin = &mut pins_list[from_pin_index];

            // connect output to wire
            pin.other_gate_index = Some(to_gate_index);
            pin.other_pin_index = Some(to_pin_index);
            pin.other_pin_type = Some(to_pin_type);
            pin.wire_index = Some(wire_index);
        }

        {
            let gate: &mut Gate = &mut self.gates[to_gate_index];

            let pins_list= match to_pin_type {
                PinType::Input => { &mut gate.input }
                PinType::Output => { &mut gate.output }
            };

            let pin = &mut pins_list[to_pin_index];

            // connect output to wire
            pin.other_gate_index = Some(from_gate_index);
            pin.other_pin_index = Some(from_pin_index);
            pin.other_pin_type = Some(from_pin_type);
            pin.wire_index = Some(wire_index);
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
            for current_pin in gate.output.iter() {
                match (current_pin.other_gate_index, current_pin.other_pin_index, current_pin.other_pin_type) {
                    // we only connect outputs to inputs for now
                    (Some(other_gate_index),  Some(other_pin_index), Some(PinType::Input)) => {
                        let current_gate = gate;
                        let other_gate = self.gates[other_gate_index].clone();
                        let other_pin = other_gate.get_pin(other_pin_index, PinType::Input);

                        let Vec2 {x: output_center_x, y: output_center_y} = current_gate.get_pin_block(current_pin.clone()).rect.center();
                        let Vec2 {x: input_center_x, y: input_center_y} = other_gate.get_pin_block(other_pin).rect.center();

                        draw_line(output_center_x, output_center_y, input_center_x, input_center_y, 3.0, BLACK);
                    }
                    _ => {}
                }
            }
           
            // to cut down on the processing each frame, it would be good to have the blocks be part of the gate, as these never change.
            let pin_blocks = &gate.clone().get_pins_blocks(); // this is kinda scuffed
            // draw blocks
            for pin_block in pin_blocks {
                let pin_rect = pin_block.rect;
                if intersects(pin_rect, camera_view_rect) {
                    draw_rectangle(pin_rect.x, pin_rect.y, pin_rect.w, pin_rect.h, BLACK);
                }
            }
        }
    }

    pub fn draw_wires(&self, camera : &Camera2D) {

        set_camera(camera);

        for gate in &self.gates {
            // check intersection of wire with screen (later)
            // draw wires
            for current_pin in gate.output.iter() {
                match (current_pin.other_gate_index, current_pin.other_pin_index, current_pin.other_pin_type) {
                    // we only connect outputs to inputs for now
                    (Some(other_gate_index),  Some(other_pin_index), Some(PinType::Input)) => {
                        let current_gate = gate;
                        let other_gate = self.gates[other_gate_index].clone();
                        let other_pin = other_gate.get_pin(other_pin_index, PinType::Input);

                        let Vec2 {x: output_center_x, y: output_center_y} = current_gate.get_pin_block(current_pin.clone()).rect.center();
                        let Vec2 {x: input_center_x, y: input_center_y} = other_gate.get_pin_block(other_pin).rect.center();

                        draw_line(output_center_x, output_center_y, input_center_x, input_center_y, 3.0, BLACK);
                    }
                    _ => {}
                }
            }
           
        }
    }
    
    pub fn draw_pins(&self, camera : &Camera2D) {
        
        set_camera(camera);

        for gate in &self.gates {
            let camera_view_rect = camera_view_rect(&camera);

            // to cut down on the processing each frame, it would be good to have the blocks be part of the gate, as these never change.
            let pin_blocks = &gate.clone().get_pins_blocks(); // this is kinda scuffed
            // draw blocks
            for pin_block in pin_blocks {
                let pin_rect = pin_block.rect;
                if intersects(pin_rect, camera_view_rect) {
                    draw_rectangle(pin_rect.x, pin_rect.y, pin_rect.w, pin_rect.h, BLACK);
                }
            }
        }
    }

    pub fn draw_mouse_wire(&self, camera: &Camera2D, gate_index: Option<usize>, pin_index: Option<usize>, pin_type: Option<PinType>) {

        match (gate_index, pin_index, pin_type) {
            (Some(gate_index), Some(pin_index), Some(pin_type)) => {
                let gate = self.gates[gate_index].clone(); // this is fine because we only read and don't write
                let block = gate.get_pin_block(gate.get_pin(pin_index, pin_type));
                let Vec2{x: center_x, y: center_y} = block.rect.center();
                let mouse_world = camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                draw_line(center_x, center_y, mouse_world.x, mouse_world.y, 3.0, BLACK);
            }
            _ => {}
        };
    }

    pub fn draw_gate_over_mouse(&self, camera: &Camera2D, rect : Rect, gate_type : &GateType) {
        // just to be sure
        if intersects(rect, camera_view_rect(camera)) {
            let color = GateType::color(gate_type);
            let text = GateType::text(gate_type);

            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color.with_alpha(0.5));
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
                    color: BLACK.with_alpha(0.5),
                    ..Default::default()
                }
            );
        }     
    }   
}
