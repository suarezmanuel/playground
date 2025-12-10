use std::{time::SystemTime};

use macroquad::prelude::*;

mod simulator;
mod types;
mod ui;
mod utils; // Import the new module

use simulator::Simulator;

#[macroquad::main("Logic Sim")]
async fn main() {
    let mut sim = Simulator::new();

    let mut counter = 0;
    let mut now = SystemTime::now();
    let mut fps = 0;

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        // 1. Update Logic (Input, Simulation, Physics)
        sim.update();

        // 2. Render
        sim.draw();

        // 3. Debug / FPS
        draw_text(&format!("FPS: {}", fps), 20.0, 30.0, 30.0, WHITE);

        next_frame().await;
        counter += 1;
        let total_time_elapsed = now.elapsed().unwrap().as_millis() as i32;

        if total_time_elapsed >= 1000 {
            fps = counter;
            counter = 0;
            now = SystemTime::now();
        }
    }
}
