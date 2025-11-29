use macroquad::prelude::*;
use crate::types::*;
use crate::utils::*;
use std::mem;

impl Circuit {

    pub fn add_gate(&mut self, gate : Gate) -> usize {
        // not necessarily in topological order
        self.gates.push(gate);
        return self.gates.len()-1;
    }

    pub fn draw_gates(&self, camera : &Camera2D) {
        set_camera(camera);

        for gate in &self.gates {
            let camera_view_rect = camera_view_rect(&camera);
            let rect = gate.rect;
            if intersects(rect, camera_view_rect) {        
                let color: Color = match gate.gate_type {
                    GateType::NOT  => RED,
                    GateType::OR   => PINK,
                    GateType::XOR  => BLUE,
                    GateType::XNOR => GRAY,
                    GateType::NOR  => ORANGE,
                    GateType::AND  => PURPLE,
                    GateType::NAND => BROWN,
                    GateType::PWR  => YELLOW,
                    GateType::GND  => DARKGRAY,
                };

                let text: &str = match gate.gate_type {
                    GateType::NOT  => "not",
                    GateType::OR   => "or",
                    GateType::XOR  => "xor",
                    GateType::XNOR => "xnor",
                    GateType::NOR  => "nor",
                    GateType::AND  => "and",
                    GateType::NAND => "nand",
                    GateType::PWR  => "pwr",
                    GateType::GND  => "gnd",
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