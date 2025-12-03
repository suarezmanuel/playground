use macroquad::prelude::*;

mod events;
mod types;
mod ui;
mod utils;
mod simulator; // Import the new module

use simulator::Simulator;

#[macroquad::main("Logic Sim")]
async fn main() {
    let mut sim = Simulator::new();

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        // 1. Update Logic (Input, Simulation, Physics)
        sim.update();

        // 2. Render
        sim.draw();
        
        // 3. Debug / FPS
        draw_text(&format!("FPS: {}", get_fps()), 20.0, 30.0, 30.0, WHITE);

        next_frame().await
    }
}