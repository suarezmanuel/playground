use crate::{Gate, GateType};
use macroquad::prelude::*;

fn draw_ui_gate(current_selection: GateType) {
    let gate_side_len = 64.0;
    let padding = 10.0;
    let color = Color::new();
        match current_selection {
            GateType::AND => {
                draw_rectangle(screen_width() - gate_side_len - padding, screen_height() - gate_side_len - padding, gate_side_len, gate_side_len, color);
                measure_text();
                draw_text();

                // draw_text((rect_width * 0.5) - (text_width * 0.5), rect_height * 0.5 - text_height * 0.5, font, text_size, text)
            }   // draw_text(rect_width * 0.5, rect_height * 0.5, ...)
            _ => {}
        }
    }

pub fn draw_ui(log_msg: String) {
    // ensure we're in screen-space (pixels)
    set_default_camera();

    // full white background
    // centered text

    let dims = measure_text("hello from ui", None, 32, 1.0);
    let x = 0.0;
    // measure_text.height is in pixels from baseline; draw_text expects y as baseline
    let y = screen_height() - 100.0;

    draw_rectangle(x, y, screen_width() - x, screen_height() - y, WHITE);
    draw_text(log_msg.as_str(), x, y + 50.0, 16 as f32, BLACK);
}
