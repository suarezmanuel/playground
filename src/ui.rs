use crate::GateType;
use macroquad::prelude::*;

pub fn draw_ui_gate(current_selection: GateType) {
    set_default_camera();

    let gate_side_len = 80.0;
    let padding = 10.0;

    draw_rectangle(
        screen_width() - gate_side_len - padding,
        screen_height() - gate_side_len - padding,
        gate_side_len,
        gate_side_len,
        current_selection.color(),
    );  

    let measured_text = measure_text(current_selection.text(), None, 16, 1.0);
    let x = screen_width() - gate_side_len - padding + (gate_side_len - measured_text.width) * 0.5;
    let y = screen_height() - gate_side_len - padding + (gate_side_len + measured_text.height) * 0.5;

    draw_text(current_selection.text(), x, y, 16 as f32, BLACK);
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
