use macroquad::prelude::*;

pub fn draw_ui(log_msg : String) {
    // ensure we're in screen-space (pixels)
    set_default_camera();

    // full white background
    // centered text

    let dims = measure_text("hello from ui", None, 32, 1.0);
    let x = 0.0;
    // measure_text.height is in pixels from baseline; draw_text expects y as baseline
    let y = screen_height() - 100.0;

    draw_rectangle(x, y, screen_width()-x, screen_height()-y, WHITE);
    draw_text(log_msg.as_str(), x, y+50.0, 32 as f32, BLACK);
}
