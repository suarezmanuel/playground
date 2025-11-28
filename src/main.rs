use macroquad::prelude::*;
use std::time::SystemTime;
mod events;
mod gates;
mod types;
mod ui;
mod utils;
use types::*;
use ui::draw_ui;
use utils::*;

use crate::types::Circuit;

#[macroquad::main("My Macroquad Demo")]
async fn main() {
    let mut counter = 0;
    let mut fps: i32 = 0;
    let mut now = SystemTime::now();
    let FPS_REST = 10;

    let mut camera: Camera2D = Camera2D::from_display_rect(Rect {
        w: screen_width(),
        h: screen_height(),
        x: 0.0,
        y: 0.0,
    });
    let mut starting_drag_world: Option<Vec2> = None;
    let base_zoom = camera.zoom;

    let mut circuit: Circuit = Circuit {
        wires: Vec::new(),
        gates: Vec::new(),
    };
    circuit.add_gate(
        Rect {
            w: 100.0,
            h: 100.0,
            x: 0 as f32,
            y: 0 as f32,
        },
        0,
        0,
        GateType::AND,
    );
    circuit.add_gate(
        Rect {
            w: 100.0,
            h: 100.0,
            x: 100 as f32,
            y: 0 as f32,
        },
        0,
        0,
        GateType::OR,
    );
    circuit.add_gate(
        Rect {
            w: 100.0,
            h: 100.0,
            x: 200 as f32,
            y: 0 as f32,
        },
        0,
        0,
        GateType::NAND,
    );
    circuit.add_gate(
        Rect {
            w: 100.0,
            h: 100.0,
            x: 300 as f32,
            y: 0 as f32,
        },
        0,
        0,
        GateType::XOR,
    );
    circuit.add_gate(
        Rect {
            w: 100.0,
            h: 100.0,
            x: 400 as f32,
            y: 0 as f32,
        },
        0,
        0,
        GateType::XNOR,
    );
    circuit.add_gate(
        Rect {
            w: 100.0,
            h: 100.0,
            x: 500 as f32,
            y: 0 as f32,
        },
        0,
        0,
        GateType::NOR,
    );

    loop {
        if counter == FPS_REST {
            let total_time_elapsed = now.elapsed().unwrap().as_millis() as i32;
            fps = (FPS_REST * 1000) / total_time_elapsed;
            counter = 0;
            now = SystemTime::now();
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            // remember the world point under cursor when starting drag
            starting_drag_world =
                Some(camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1)));
        } else if is_mouse_button_down(MouseButton::Left) {
            if let Some(start_world) = starting_drag_world {
                let current_world =
                    camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                // move camera target so the world point under cursor follows the drag
                camera.target += start_world - current_world;
            }
        } else {
            starting_drag_world = None;
        }

        let (_sx, sy) = mouse_wheel();
        if sy != 0.0 {
            let sensitivity = 0.001; // tune
            let MAX_ZOOM = 100.0;
            // clamp factor to avoid zero/negative scaling
            let factor = (1.0 + sy * sensitivity).max(0.01); // >1 zooms in, <1 zooms out

            let mut new_zoom: Vec2 = camera.zoom * Vec2::new(factor, factor);
            new_zoom.x = new_zoom
                .x
                .clamp(base_zoom.x * (1.0 / MAX_ZOOM), base_zoom.x * MAX_ZOOM);
            new_zoom.y = new_zoom
                .y
                .clamp(base_zoom.y * MAX_ZOOM, base_zoom.y * (1.0 / MAX_ZOOM));

            // zoom toward mouse position:
            let mouse = Vec2::new(mouse_position().0, mouse_position().1);
            let before = camera.screen_to_world(mouse);

            camera.zoom = new_zoom;

            let after = camera.screen_to_world(mouse);
            camera.target += before - after; // keep focus under cursor
        }

        counter += 1;

        set_camera(&camera);
        clear_background(BLUE);

        utils::draw_grid(&camera, base_zoom);

        circuit.draw_gates(&camera);

        // println!("{}", draw_counter);
        set_default_camera();
        draw_text(&format!("{}", fps), 20.0, 30.0, 40.0, WHITE);
        // Advance to the next frame
        draw_ui();
        next_frame().await
    }
}
