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

            let pin_blocks = gate.clone().get_pins_blocks(); // this is kinda scuffed
            for pin_block in pin_blocks {
                let pin_rect = pin_block.rect;
                if intersects(pin_rect, camera_view_rect) {
                    draw_rectangle(pin_rect.x, pin_rect.y, pin_rect.w, pin_rect.h, BLACK);
                }
            }
        }
    }
}