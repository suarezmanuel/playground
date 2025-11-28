use macroquad::prelude::*;
use crate::types::*;
use crate::utils::*;

impl circuit {

    pub fn add_gate(&mut self, rect : Rect, input1 : u32, input2 : u32, gate_type : gate_type) {

        let newInput2 = match gate_type {
            gate_type::NOT => 0,
            _ => input2
        };
        // not necessarily in topological order
        self.gates.push(gate{rect : rect, input1: input1, input2: newInput2, output: 0 as u32, gate_type : gate_type});
    }

    // the wires result can only be trusted after a 'emulate'
    pub fn add_wire(&mut self, gateOut: gate, gateIn: u32, inputProbe: u32, wireStart : u32, wireEnd : u32) {
        // should remove the already existing
        // gateOut.output = wireStart;
        // updateWireVisual(gateOut);

        // add multiplexers in future
        // let mut probe = match inputProbe {
            
        // }

        // show have some good (x,y) data
    }

    // assumes topological order
    pub fn tick(&mut self) {
        for gate in &self.gates {
            let a = self.wires[gate.input1 as usize];
            let b = self.wires[gate.input2 as usize]; // b should always have a value, even for 'NOT'

            let result: bool = match gate.gate_type {
                gate_type::NOT  => !a,
                gate_type::OR   => a | b,
                gate_type::XOR  => a ^ b,
                gate_type::XNOR => !(a ^ b),
                gate_type::NOR  => !(a | b),
                gate_type::AND  => a & b,
                gate_type::NAND => !(a & b),
            };

            self.wires[gate.output as usize] = result;
        }
    }

    pub fn draw_gates(&self, camera : &Camera2D) {
        set_camera(camera);

        for gate in &self.gates {
            let camera_view_rect = camera_view_rect(&camera);
            let rect = gate.rect;
            if intersects(rect, camera_view_rect) {        
                let color: Color = match gate.gate_type {
                    gate_type::NOT  => RED,
                    gate_type::OR   => PINK,
                    gate_type::XOR  => BLUE,
                    gate_type::XNOR => GRAY,
                    gate_type::NOR  => ORANGE,
                    gate_type::AND  => PURPLE,
                    gate_type::NAND => BROWN,
                };

                let text: &str = match gate.gate_type {
                    gate_type::NOT  => "not",
                    gate_type::OR   => "or",
                    gate_type::XOR  => "xor",
                    gate_type::XNOR => "xnor",
                    gate_type::NOR  => "nor",
                    gate_type::AND  => "and",
                    gate_type::NAND => "nand",
                };
                
                draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
                let dims = measure_text(text, None, 32, 1.0);
                let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
                let ty = rect.y + rect.h * 0.35 + dims.height * 0.5; 
                // draw_text_ex(text, tx, ty, TextParams { font: None, font_size: 32, font_scale: 1.0, rotation: std::f32::consts::PI, color: BLACK, ..Default::default() });
                // draw_text(text, tx, ty,  32.0, BLACK);

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
        }
    }
}