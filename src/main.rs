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

#[derive(Debug, PartialEq, Clone, Copy)] 
struct SpatialBlockIndex {
    pub rect : Rect,
    pub index : usize
}

impl RTreeObject for SpatialBlockIndex {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.rect.x, self.rect.y],
            [self.rect.x + self.rect.w, self.rect.y + self.rect.h],
        )
    }
}

impl PointDistance for SpatialBlockIndex {
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

    let mut tree: RTree<SpatialBlockIndex> = RTree::new();

    loop {

        let mut log_msg: String = "".to_string();

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
        let closest_item = tree.nearest_neighbor(&[mouse_world.x, mouse_world.y]).copied();
        let cursor_item = tree.locate_at_point(&[mouse_world.x, mouse_world.y]).copied();

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
        
        match cursor_item {
            //  print the id of the hovering element and the element count
            Some(spatial_gate_index) => {
                log_msg = format!("{} hovering element id: {} |", log_msg, spatial_gate_index.index);
                // println!("hovering element id: {}, gate count: {}", spatial_gate_index.index, tree.iter().count());
                let gate = circuit.gates[spatial_gate_index.index].clone();
                // are you hovering on a pin?
                let pins_blocks = gate.get_pins_blocks();
                let mouse_pos_world = camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                for pin_block in pins_blocks {
                    let pin_rect = pin_block.rect;
                    if pin_rect.contains(mouse_pos_world) {
                        log_msg = format!("{} hovering {} pin {} |", log_msg, pin_block.pin_type.to_string(), pin_block.index);
                    }
                }
            }

            // if there is no element here, make a new one.
            _ => {

                match closest_item {
                    Some(item) => {
                        log_msg = format!("{} closest element id: {} |", log_msg, item.index);
                        // println!("closest element id: {}", item.index);
                    }
                    _ => {}
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    let mouse_pos_world = camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                    let gate_rect = Rect { w: 64.0, h: 64.0, x: (mouse_pos_world.x / 64.0).floor() * 64.0, y: (mouse_pos_world.y / 64.0).floor() * 64.0 };
                    let gate_index = circuit.add_gate(Gate::new(gate_rect, current_selection));
                    // add to tree with id 
                    tree.insert(SpatialBlockIndex{rect: gate_rect, index: gate_index});
                }
            }
        }

        log_msg = format!("{} gate count: {} |\n", log_msg, tree.iter().count());

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
        draw_ui(log_msg);
        next_frame().await
    }
}
