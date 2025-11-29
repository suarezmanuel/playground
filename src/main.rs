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
use rstar::{RTree, RTreeObject, AABB, PointDistance};

use crate::types::Circuit;

struct SpatialGateIndex {
    rect : Rect,
    index : usize
}

impl RTreeObject for SpatialGateIndex {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.rect.x, self.rect.y],
            [self.rect.x + self.rect.w, self.rect.y + self.rect.h],
        )
    }
}

impl PointDistance for SpatialGateIndex {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        let x = point[0];
        let y = point[1];
        let min_x = self.rect.x;
        let min_y = self.rect.y;
        let max_x = self.rect.x + self.rect.w;
        let max_y = self.rect.y + self.rect.h;

        let dx = if x < min_x { min_x - x } else if x > max_x { x - max_x } else { 0.0 };
        let dy = if y < min_y { min_y - y } else if y > max_y { y - max_y } else { 0.0 };
        dx * dx + dy * dy
    }

    fn contains_point(&self, point: &[f32; 2]) -> bool {
        let x = point[0];
        let y = point[1];
        (x >= self.rect.x) && (x <= self.rect.x + self.rect.w) && (y >= self.rect.y) && (y <= self.rect.y + self.rect.h)
    }
}

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

    let mut current_selection = GateType::AND;

    let mut circuit: Circuit = Circuit::new();

    let mut tree: RTree<SpatialGateIndex> = RTree::new();

    // let pwr1 = Gate::new(Rect { w: 64.0, h: 64.0, x: -128 as f32, y: 0 as f32 }, GateType::PWR);
    // let pwr2 = Gate::new(Rect { w: 64.0, h: 64.0, x: -128 as f32, y: 0 as f32 }, GateType::PWR);
    // let and = Gate::new(Rect { w: 64.0, h: 64.0, x: 0 as f32, y: 0 as f32 }, GateType::AND);
    // let not = Gate::new(Rect { w: 64.0, h: 64.0, x: 64 as f32, y: 0 as f32 }, GateType::NOT);
    // let or = Gate::new(Rect { w: 64.0, h: 64.0, x: 128 as f32, y: 0 as f32 }, GateType::OR);
    // let or2 = Gate::new(Rect { w: 64.0, h: 64.0, x: 192 as f32, y: 0 as f32 }, GateType::OR);
    // let ground = Gate::new(Rect { w: 64.0, h: 64.0, x: -128 as f32, y: 128 as f32 }, GateType::GND);

    // let pwr1_index= circuit.add_gate(pwr1);
    // let pwr2_index= circuit.add_gate(pwr2);
    // let and_index= circuit.add_gate(and);
    // let not_index= circuit.add_gate(not);
    // let or_index = circuit.add_gate(or);
    // let or_index2 = circuit.add_gate(or2);
    // let ground_index = circuit.add_gate(ground);

    // let source1= circuit.new_wire();
    // let source2= circuit.new_wire();
    // let wire1 = circuit.new_wire();
    // let wire2 = circuit.new_wire();
    // let wire3 = circuit.new_wire();
    // let result = circuit.new_wire();
    // // only the 'electricity source' wire should be set
    // // circuit.set_wire(source, true);

    // circuit.connect_wire(source1, pwr1_index, and_index, 0, 0);
    // circuit.connect_wire(source1, pwr1_index, or_index, 0, 0);
    // circuit.connect_wire(source2, pwr2_index, and_index, 0, 1);
    // circuit.connect_wire(source2, pwr1_index, or_index, 0, 1);

    // circuit.connect_wire(wire1, and_index, not_index, 0, 0);
    // circuit.connect_wire(wire2, not_index, or_index2, 0, 0);
    // circuit.connect_wire(wire3, or_index, or_index2, 0, 1);

    // circuit.connect_wire(result, or_index2, ground_index, 0, 0);


    // while !circuit.emulation_done {
    //     circuit.tick();
    //     println!("{}", circuit.get_wire(source1));
    // }
    // println!("emulation done! {}", circuit.get_wire(result));

    loop {

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if counter == fps_rest {
            let total_time_elapsed = now.elapsed().unwrap().as_millis() as i32;
            fps = (fps_rest * 1000) / total_time_elapsed;
            counter = 0;
            now = SystemTime::now();
        }

        // locate objects under the mouse cursor using a Point ([f32; 2]) so rstar's Point trait is satisfied
        let mouse_world = camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
        let item = tree.locate_at_point(&[mouse_world.x, mouse_world.y]);

        if is_key_pressed(KeyCode::Key0) {
            current_selection = GateType::NOT;
        } else if is_key_pressed(KeyCode::Key1) {
            current_selection = GateType::OR;
        } else if is_key_pressed(KeyCode::Key2) {
            current_selection = GateType::XOR;
        } else if is_key_pressed(KeyCode::Key3) {
            current_selection = GateType::NOR;
        } else if is_key_pressed(KeyCode::Key4) {
            current_selection = GateType::XNOR;
        } else if is_key_pressed(KeyCode::Key5) {
            current_selection = GateType::AND;
        } else if is_key_pressed(KeyCode::Key6) {
            current_selection = GateType::NAND;
        } else if is_key_pressed(KeyCode::Key7) {
            current_selection = GateType::PWR;
        } else if is_key_pressed(KeyCode::Key8) {
            current_selection = GateType::GND;
        } else if is_key_pressed(KeyCode::Key9) {
        } 
        
        match item {
            //  print the id of the hovering element and the element count
            Some(spatial_gate_index) => {
                println!("id: {}, gate count: {}", spatial_gate_index.index, tree.iter().count());
            }

            // if there is no element here, make a new one.
            _ => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    let mouse_pos_world = camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                    let gate_rect = Rect { w: 64.0, h: 64.0, x: (mouse_pos_world.x / 64.0).floor() * 64.0, y: (mouse_pos_world.y / 64.0).floor() * 64.0 };
                    let gate_index = circuit.add_gate(Gate::new(gate_rect, current_selection));
                    // add to tree with id 
                    tree.insert(SpatialGateIndex{rect: gate_rect, index: gate_index});
                }
            }
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
