use macroquad::prelude::*;
use std::thread::current;
use std::time::SystemTime;
mod events;
mod types;
mod ui;
mod utils;
use crate::types::circuit::*;
use crate::types::gate::*;
use crate::types::gate_type::*;
use crate::types::pin_type::*;
use crate::ui::draw_ui_gate;
use rstar::{AABB, PointDistance, RTree, RTreeObject};
use ui::draw_ui;

#[derive(Debug, PartialEq, Clone, Copy)]
struct SpatialBlockIndex {
    pub rect: Rect,
    pub index: usize,
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

        let dx = if x < min_x {
            min_x - x
        } else if x > max_x {
            x - max_x
        } else {
            0.0
        };
        let dy = if y < min_y {
            min_y - y
        } else if y > max_y {
            y - max_y
        } else {
            0.0
        };
        dx * dx + dy * dy
    }

    fn contains_point(&self, point: &[f32; 2]) -> bool {
        let x = point[0];
        let y = point[1];
        (x >= self.rect.x)
            && (x <= self.rect.x + self.rect.w)
            && (y >= self.rect.y)
            && (y <= self.rect.y + self.rect.h)
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
    // let base_zoom = camera.zoom;
    let mut zoom_factor = 1.0;

    let mut current_selection = GateType::AND;

    let mut circuit: Circuit = Circuit::new();

    let mut tree: RTree<SpatialBlockIndex> = RTree::new();

    let mut start_gate_index: Option<usize> = None;
    let mut start_pin_index: Option<usize> = None;
    let mut start_pin_type: Option<PinType> = None;

    let mut emulate = false;
    let mut last_tick_time = SystemTime::now();

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
        let closest_item = tree
            .nearest_neighbor(&[mouse_world.x, mouse_world.y])
            .copied();
        let cursor_item = tree
            .locate_at_point(&[mouse_world.x, mouse_world.y])
            .copied();

        if is_key_pressed(KeyCode::Key1) {
            current_selection = GateType::NOT;
        } else if is_key_pressed(KeyCode::Key2) {
            current_selection = GateType::OR;
        } else if is_key_pressed(KeyCode::Key3) {
            current_selection = GateType::XOR;
        } else if is_key_pressed(KeyCode::Key4) {
            current_selection = GateType::NOR;
        } else if is_key_pressed(KeyCode::Key5) {
            current_selection = GateType::XNOR;
        } else if is_key_pressed(KeyCode::Key6) {
            current_selection = GateType::AND;
        } else if is_key_pressed(KeyCode::Key7) {
            current_selection = GateType::NAND;
        } else if is_key_pressed(KeyCode::Key8) {
            current_selection = GateType::PWR;
        } else if is_key_pressed(KeyCode::Key9) {
            current_selection = GateType::GND;
        } 

        if is_key_pressed(KeyCode::R) {
            emulate = !emulate; 
        }

        if is_key_pressed(KeyCode::T) {
            circuit.reset_wires();
        }
        
        
        log_msg = format!("{log_msg} emulate {emulate} |");

        match cursor_item {
            //  print the id of the hovering element and the element count
            Some(hover_spatial_gate) => {
                log_msg = format!(
                    "{} hovering element id: {} |",
                    log_msg, hover_spatial_gate.index
                );
                // println!("hovering element id: {}, gate count: {}", hover_spatial_gate.index, tree.iter().count());
                let gate = circuit.gates[hover_spatial_gate.index].clone();
                // are you hovering on a pin?
                let pins_blocks = gate.get_pins_blocks();
                let mouse_pos_world =
                    camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                for pin_block in pins_blocks {
                    let pin_rect = pin_block.rect;
                    let mut hover_pin_index: Option<usize> = None;
                    let mut hover_pin_type: Option<PinType> = None;
                    if pin_rect.contains(mouse_pos_world) {
                        log_msg = format!(
                            "{} hovering {} pin {} |",
                            log_msg,
                            pin_block.pin_type.to_string(),
                            pin_block.index
                        );
                        hover_pin_index = Some(pin_block.index);
                        hover_pin_type = Some(pin_block.pin_type);
                    }

                    match (hover_pin_index, hover_pin_type, hover_spatial_gate.index) {
                        (Some(to_pin_index), Some(to_pin_type), to_gate_index) => {
                            // cancel start of cable
                            // delete existing cable
                            if is_mouse_button_pressed(MouseButton::Right) {
                                let to_pin =
                                    circuit.gates[to_gate_index].get_pin(to_pin_index, to_pin_type);
                                match (
                                    to_pin.other_gate_index,
                                    to_pin.other_pin_index,
                                    to_pin.other_pin_type,
                                ) {
                                    (
                                        Some(other_gate_index),
                                        Some(other_pin_index),
                                        Some(other_pin_type),
                                    ) => {
                                        circuit.remove_wire(
                                            circuit.gates[to_gate_index]
                                                .get_pin(to_pin_index, to_pin_type)
                                                .wire_index
                                                .unwrap(),
                                        );

                                        {
                                            let gate = &mut circuit.gates[other_gate_index];
                                            let pins = match other_pin_type {
                                                PinType::Input => &mut gate.input,
                                                PinType::Output => &mut gate.output,
                                            };

                                            let pin = &mut pins[other_pin_index];
                                            pin.other_gate_index = None;
                                            pin.other_pin_index = None;
                                            pin.other_pin_type = None;
                                            pin.wire_index = None;
                                        }

                                        {
                                            let gate = &mut circuit.gates[to_gate_index];
                                            let pins = match to_pin_type {
                                                PinType::Input => &mut gate.input,
                                                PinType::Output => &mut gate.output,
                                            };

                                            let pin = &mut pins[to_pin_index];
                                            pin.other_gate_index = None;
                                            pin.other_pin_index = None;
                                            pin.other_pin_type = None;
                                            pin.wire_index = None;
                                        }
                                    }

                                    _ => {}
                                }

                                start_gate_index = None;
                                start_pin_index = None;
                                start_pin_type = None;
                            } else if is_mouse_button_pressed(MouseButton::Left) {
                                match (start_pin_index, start_pin_type, start_gate_index) {
                                    (
                                        Some(from_pin_index),
                                        Some(from_pin_type),
                                        Some(from_gate_index),
                                    ) => {
                                        // new cable
                                        // only use 'get_pin' to get values not to set them
                                        let from_pin = circuit.gates[from_gate_index]
                                            .get_pin(from_pin_index, from_pin_type);
                                        let to_pin = circuit.gates[to_gate_index]
                                            .get_pin(to_pin_index, to_pin_type);
                                        // not in the same gate
                                        if from_gate_index != to_gate_index &&
                                           // not the same type of pin
                                           to_pin_type.to_string() != from_pin_type.to_string() && 
                                           // not an already existing cable
                                           (from_pin.other_gate_index != Some(to_gate_index) || from_pin.other_pin_index != Some(to_pin_index))
                                        {
                                            circuit.connect_wire(
                                                from_gate_index,
                                                to_gate_index,
                                                from_pin.index,
                                                from_pin.pin_type,
                                                to_pin.index,
                                                to_pin.pin_type,
                                            );
                                            start_gate_index = None;
                                            start_pin_index = None;
                                            start_pin_type = None;
                                        }
                                    }
                                    (None, None, None) => {
                                        start_gate_index = Some(to_gate_index);
                                        start_pin_index = Some(to_pin_index);
                                        start_pin_type = Some(to_pin_type);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
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
                    let mouse_pos_world =
                        camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                    let gate_rect = Rect {
                        w: 64.0,
                        h: 64.0,
                        x: (mouse_pos_world.x / 64.0).floor() * 64.0,
                        y: (mouse_pos_world.y / 64.0).floor() * 64.0,
                    };
                    let gate_index = circuit.add_gate(Gate::new(gate_rect, current_selection));
                    // add to tree with id
                    tree.insert(SpatialBlockIndex {
                        rect: gate_rect,
                        index: gate_index,
                    });
                }
            }
        }

        log_msg = format!("{} gate count: {} |\n", log_msg, tree.iter().count());

        if is_mouse_button_pressed(MouseButton::Right) {
            start_gate_index = None;
            start_pin_index = None;
            start_pin_type = None;
        }

        match (start_gate_index, start_pin_index, start_pin_type) {
            (Some(a), Some(b), Some(c)) => {
                log_msg = format!(
                    "{} st_gate_index: {} st_pin_index: {} st_pin_type {}|\n",
                    log_msg,
                    a,
                    b,
                    c.to_string()
                );
            }
            _ => {}
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
        let screen_width = screen_width();
        let screen_height = screen_height();
        let zoom_rect = vec2(1. / screen_width * 2., 1. / screen_height * 2.);
        if sy != 0.0 {
            let sensitivity = 0.001; // tune
            let max_zoom = 100.0;
            // clamp factor to avoid zero/negative scaling
            // let factor = (1.0 + sy * sensitivity).max(0.01); // >1 zooms in, <1 zooms out
            zoom_factor *= (1.0 + sy * sensitivity).max(0.01);
            zoom_factor = zoom_factor.clamp(1.0 / max_zoom, max_zoom);
        }

        let new_zoom: Vec2 = zoom_rect * zoom_factor;

        // zoom toward mouse position:
        let mouse = Vec2::new(mouse_position().0, mouse_position().1);
        let before = camera.screen_to_world(mouse);

        camera.zoom = new_zoom;

        let after = camera.screen_to_world(mouse);
        camera.target += before - after; // keep focus under cursor

        counter += 1;

        set_camera(&camera);
        clear_background(BLUE);

        utils::draw_grid(&camera, zoom_rect);

        // draw transparent gate over mouse.
        // doesn't matter if its drawn because a gate will be drawn over it
        match (start_gate_index, start_pin_index, start_pin_type) {
            (None, None, None) => {
                let mouse_pos_world =
                    camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                let gate_rect = Rect {
                    w: 64.0,
                    h: 64.0,
                    x: (mouse_pos_world.x / 64.0).floor() * 64.0,
                    y: (mouse_pos_world.y / 64.0).floor() * 64.0,
                };
                circuit.draw_gate_over_mouse(&camera, gate_rect, &current_selection);
            }
            _ => {}
        }

        if emulate && (last_tick_time.elapsed().unwrap().as_millis() as i32 > 1000) {
            circuit.tick();
            last_tick_time = SystemTime::now();
            log_msg = format!("{log_msg} | tick happened!!");
        }

        circuit.draw_gates(&camera);
        circuit.draw_wires(&camera);
        circuit.draw_pins(&camera);
        circuit.draw_mouse_wire(&camera, start_gate_index, start_pin_index, start_pin_type);

        // println!("{}", draw_counter);
        set_default_camera();

        draw_text(&format!("{}", fps), 20.0, 30.0, 40.0, WHITE);
        // Advance to the next frame
        draw_ui(log_msg);
        draw_ui_gate(current_selection);
        next_frame().await
    }
}
