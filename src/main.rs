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
    let fps_rest = 10;

    let mut camera: Camera2D = Camera2D::from_display_rect(Rect {
        w: screen_width(),
        h: screen_height(),
        x: 0.0,
        y: 0.0,
    });
    let mut starting_drag_world: Option<Vec2> = None;
    let base_zoom = camera.zoom;

    let mut circuit: Circuit = Circuit::new();

    let pwr1 = Gate::new(Rect { w: 64.0, h: 64.0, x: -128 as f32, y: 0 as f32 }, GateType::PWR);
    let pwr2 = Gate::new(Rect { w: 64.0, h: 64.0, x: -128 as f32, y: 0 as f32 }, GateType::PWR);
    let and = Gate::new(Rect { w: 64.0, h: 64.0, x: 0 as f32, y: 0 as f32 }, GateType::AND);
    let not = Gate::new(Rect { w: 64.0, h: 64.0, x: 64 as f32, y: 0 as f32 }, GateType::NOT);
    let or = Gate::new(Rect { w: 64.0, h: 64.0, x: 128 as f32, y: 0 as f32 }, GateType::OR);
    let or2 = Gate::new(Rect { w: 64.0, h: 64.0, x: 192 as f32, y: 0 as f32 }, GateType::OR);
    let ground = Gate::new(Rect { w: 64.0, h: 64.0, x: -128 as f32, y: 128 as f32 }, GateType::GND);

    let pwr1_index= circuit.add_gate(pwr1);
    let pwr2_index= circuit.add_gate(pwr2);
    let and_index= circuit.add_gate(and);
    let not_index= circuit.add_gate(not);
    let or_index = circuit.add_gate(or);
    let or_index2 = circuit.add_gate(or2);
    let ground_index = circuit.add_gate(ground);

    let source1= circuit.new_wire();
    let source2= circuit.new_wire();
    let wire1 = circuit.new_wire();
    let wire2 = circuit.new_wire();
    let wire3 = circuit.new_wire();
    let result = circuit.new_wire();
    // only the 'electricity source' wire should be set
    // circuit.set_wire(source, true);

    circuit.connect_wire(source1, pwr1_index, and_index, 0, 0);
    circuit.connect_wire(source1, pwr1_index, or_index, 0, 0);
    circuit.connect_wire(source2, pwr2_index, and_index, 0, 1);
    circuit.connect_wire(source2, pwr1_index, or_index, 0, 1);

    circuit.connect_wire(wire1, and_index, not_index, 0, 0);
    circuit.connect_wire(wire2, not_index, or_index2, 0, 0);
    circuit.connect_wire(wire3, or_index, or_index2, 0, 1);

    circuit.connect_wire(result, or_index2, ground_index, 0, 0);


    while !circuit.emulation_done {
        circuit.tick();
        println!("{}", circuit.get_wire(source1));
    }
    println!("emulation done! {}", circuit.get_wire(result));

    loop {
        if counter == fps_rest {
            let total_time_elapsed = now.elapsed().unwrap().as_millis() as i32;
            fps = (fps_rest * 1000) / total_time_elapsed;
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
            let max_zoom = 100.0;
            // clamp factor to avoid zero/negative scaling
            let factor = (1.0 + sy * sensitivity).max(0.01); // >1 zooms in, <1 zooms out

            let mut new_zoom: Vec2 = camera.zoom * Vec2::new(factor, factor);
            new_zoom.x = new_zoom
                .x
                .clamp(base_zoom.x * (1.0 / max_zoom), base_zoom.x * max_zoom);
            new_zoom.y = new_zoom
                .y
                .clamp(base_zoom.y * max_zoom, base_zoom.y * (1.0 / max_zoom));

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
