use crate::types::circuit::*;
use crate::types::gate;
use crate::types::gate::*;
use crate::types::gate_type::*;
use crate::types::pin_type::*;
use crate::types::wires::ConnectionUtils;
use crate::ui::draw_ui;
use crate::utils::draw_grid;
use macroquad::prelude::*;
use rstar::{AABB, PointDistance, RTree, RTreeObject};
use std::fmt::format;
use std::time::SystemTime;

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

#[derive(Clone, Copy, PartialEq)]
enum InputState {
    Idle,
    ChoosingGate {
        gate_type: GateType,
    },
    CameraDrag {
        start_world: Vec2,
    },
    GateDrag {
        gate_id: usize,
        offset: Vec2,
    },
    Wiring {
        gate: usize,
        pin: usize,
        p_type: PinType,
    },
}

pub struct Simulator {
    // Systems
    pub circuit: Circuit,
    pub tree: RTree<SpatialBlockIndex>,
    pub camera: Camera2D,
    pub zoom_factor: f32,

    // State
    state: InputState,

    // Simulation settings
    pub emulate: bool,
    last_tick: SystemTime,
    pub log_msg: String,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            circuit: Circuit::new(),
            tree: RTree::new(),
            camera: Camera2D::from_display_rect(Rect::new(0., 0., screen_width(), screen_height())),
            zoom_factor: 1.0,
            state: InputState::Idle,
            emulate: false,
            last_tick: SystemTime::now(),
            log_msg: String::new(),
        }
    }

    pub fn update(&mut self) {
        self.log_msg.clear();

        if self.emulate && (self.last_tick.elapsed().unwrap().as_millis() as i32 > 1000) {
            self.circuit.tick();
            self.last_tick = SystemTime::now();
        }

        self.handle_keyboard();
        self.handle_zoom();
        self.handle_mouse();
    }

    pub fn handle_keyboard(&mut self) {
        if is_key_pressed(KeyCode::Key0) {
            self.state = InputState::Idle;
        } else if is_key_pressed(KeyCode::Key1) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::NOT,
            };
        } else if is_key_pressed(KeyCode::Key2) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::OR,
            };
        } else if is_key_pressed(KeyCode::Key3) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::XOR,
            };
        } else if is_key_pressed(KeyCode::Key4) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::NOR,
            };
        } else if is_key_pressed(KeyCode::Key5) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::XNOR,
            };
        } else if is_key_pressed(KeyCode::Key6) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::AND,
            };
        } else if is_key_pressed(KeyCode::Key7) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::NAND,
            };
        } else if is_key_pressed(KeyCode::Key8) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::PWR,
            };
        } else if is_key_pressed(KeyCode::Key9) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::GND,
            };
        } else if is_key_pressed(KeyCode::R) {
            self.emulate = !self.emulate;
        } else if is_key_pressed(KeyCode::T) {
            self.circuit.reset_wires();
        }
    }

    pub fn handle_mouse(&mut self) {
        let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
        let mouse_world = self.camera.screen_to_world(mouse_screen);

        // find out hover
        let hovered_gate_idx = self
            .tree
            .locate_at_point(&[mouse_world.x, mouse_world.y])
            .and_then(|item| Some(item.index));

        let hovered_pin = if let Some(g_idx) = hovered_gate_idx {
            self.find_hovered_pin(g_idx, mouse_world)
        } else {
            None
        };

        if is_mouse_button_pressed(MouseButton::Right) {
            self.state = InputState::Idle;
            if let Some((g_idx, p_idx, p_type)) = hovered_pin {
                self.delete_wire_at_pin(g_idx, p_idx, p_type);
            } else if let Some(g_idx) = hovered_gate_idx {
                println!("{}", g_idx);
                self.delete_gate(g_idx);
            }
        }

        match self.state {
            InputState::Idle => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some((g, p, t)) = hovered_pin {
                        // Start Wiring
                        self.state = InputState::Wiring {
                            gate: g,
                            pin: p,
                            p_type: t,
                        };
                    } else if let Some(g_idx) = hovered_gate_idx {
                        // Start Dragging Gate
                        if let Some(gate) = &mut self.circuit.gates[g_idx] {
                            let gate_pos = Vec2::new(gate.rect.x, gate.rect.y);
                            self.state = InputState::GateDrag {
                                gate_id: g_idx,
                                offset: mouse_world - gate_pos,
                            };
                            // temporarily remove gate from tree
                            self.tree.remove(&SpatialBlockIndex {
                                rect: gate.rect,
                                index: g_idx,
                            });
                        }
                    }
                } else if is_mouse_button_down(MouseButton::Left) {
                    // if not hitting anything start camera pan
                    if hovered_gate_idx.is_none() {
                        self.state = InputState::CameraDrag {
                            start_world: mouse_world,
                        };
                    }
                }
            }
            InputState::ChoosingGate { gate_type } => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.place_gate(mouse_world, gate_type);
                    self.state = InputState::Idle;
                }
            }
            InputState::CameraDrag { start_world } => {
                if is_mouse_button_down(MouseButton::Left) {
                    let current_world = self.camera.screen_to_world(mouse_screen);
                    self.camera.target += start_world - current_world;
                } else {
                    // if just released left click
                    self.state = InputState::Idle;
                }
            }
            InputState::GateDrag { gate_id, offset } => {
                if is_mouse_button_down(MouseButton::Left) {
                    let new_pos = mouse_world - offset;
                    // Snap to grid
                    let snapped = vec2(
                        (new_pos.x / 64.0).floor() * 64.0,
                        (new_pos.y / 64.0).floor() * 64.0,
                    );

                    // Update Gate Rect directly
                    if let Some(gate) = &mut self.circuit.gates[gate_id] {
                        let diff = snapped - vec2(gate.rect.x, gate.rect.y);
                        gate.offset(diff); // Assuming you have an offset method on Gate
                    }
                } else {
                    // Drop Gate: Re-insert into Tree
                    if let Some(gate) = &mut self.circuit.gates[gate_id] {
                        self.tree.insert(SpatialBlockIndex {
                            rect: gate.rect,
                            index: gate_id,
                        });
                        self.state = InputState::Idle;
                    }
                }
            }
            InputState::Wiring { gate, pin, p_type } => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some((to_g, to_p, to_t)) = hovered_pin {
                        self.circuit
                            .connect_wire(gate, to_g, pin, p_type, to_p, to_t);
                    }
                    self.state = InputState::Idle;
                }
            }
        }
    }

    pub fn handle_zoom(&mut self) {
        let (_sx, sy) = mouse_wheel();

        // 1. Calculate Base Scale (Aspect Ratio)
        // We calculate this every frame so the grid doesn't stretch if you resize the window.
        let scr_w = screen_width();
        let scr_h = screen_height();

        // This vector ensures 1 world unit = 1 pixel at zoom_factor 1.0,
        // and maintains the aspect ratio.
        // Note: We use the exact math from your snippet.
        let base_scale = vec2(1.0 / scr_w * 2.0, 1.0 / scr_h * 2.0);

        // 2. Update Zoom Factor (if wheel moved)
        if sy != 0.0 {
            let sensitivity = 0.001;
            let min_zoom = 1.0 / 100.0; // 0.01
            let max_zoom = 100.0;

            // Calculate new factor
            let new_factor = self.zoom_factor * (1.0 + sy * sensitivity).max(0.01);
            self.zoom_factor = new_factor.clamp(min_zoom, max_zoom);

            // 3. Apply Zoom-Towards-Cursor Logic

            let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);

            // A. Where is the mouse in the world RIGHT NOW?
            let before_world = self.camera.screen_to_world(mouse_screen);

            // B. Apply the new zoom to the camera
            self.camera.zoom = base_scale * self.zoom_factor;

            // C. Where is the mouse in the world AFTER zooming?
            let after_world = self.camera.screen_to_world(mouse_screen);

            // D. Adjust the camera target so the mouse stays over the same world point
            self.camera.target += before_world - after_world;
        } else {
            // Even if we didn't scroll, we must update camera.zoom
            // in case the window size (screen_width/height) changed.
            self.camera.zoom = base_scale * self.zoom_factor;
        }
    }

    pub fn draw(&mut self) {
        set_camera(&self.camera);
        clear_background(BLUE);
        // why utils::draw_grid doesn't work?
        draw_grid(
            &self.camera,
            vec2(1. / screen_width() * 2., 1. / screen_height() * 2.),
        );

        crate::utils::draw_gates(&self.circuit, &self.camera);
        crate::utils::draw_wires(&mut self.circuit, &self.camera);
        crate::utils::draw_pins(&self.circuit, &self.camera);

        // Draw Mouse Wire if wiring
        if let InputState::Wiring { gate, pin, p_type } = self.state {
            crate::utils::draw_mouse_wire(
                &self.circuit,
                &self.camera,
                Some(gate),
                Some(pin),
                Some(p_type),
            );
        }

        // draw transparent gate at mouse if choosing gate
        match self.state {
            InputState::ChoosingGate { gate_type } => {
                let m = self
                    .camera
                    .screen_to_world(vec2(mouse_position().0, mouse_position().1));
                let snap_pos = vec2((m.x / 64.).floor() * 64., (m.y / 64.).floor() * 64.);
                let r = Rect::new(snap_pos.x, snap_pos.y, 64., 64.);
                crate::utils::draw_gate_over_mouse(&self.camera, r, &gate_type, 0.5);
            }
            _ => {}
        }

        set_default_camera();
        self.log_msg = format!("{} gate count: {}", self.log_msg, self.circuit.gates.len());
        draw_ui(self.log_msg.clone());
    }

    fn place_gate(&mut self, pos: Vec2, gate_type: GateType) {
        let snap_x = (pos.x / 64.0).floor() * 64.0;
        let snap_y = (pos.y / 64.0).floor() * 64.0;
        let rect = Rect::new(snap_x, snap_y, 64.0, 64.0);

        let idx = self
            .circuit
            .add_gate(Gate::new(rect, Rotation::Up, gate_type));
        self.tree.insert(SpatialBlockIndex { rect, index: idx });
    }

    fn find_hovered_pin(
        &self,
        gate_idx: usize,
        mouse_world: Vec2,
    ) -> Option<(usize, usize, PinType)> {
        if let Some(gate) = &self.circuit.gates[gate_idx] {
            // Check Inputs
            for pin in &gate.input {
                if pin.rect.contains(mouse_world) {
                    return Some((gate_idx, pin.index, PinType::Input));
                }
            }
            // Check Outputs
            for pin in &gate.output {
                if pin.rect.contains(mouse_world) {
                    return Some((gate_idx, pin.index, PinType::Output));
                }
            }
        }
        None
    }

    fn delete_gate(&mut self, gate_idx: usize) {
        if let Some(optional_gate_ref) = self.circuit.gates.get_mut(gate_idx) {
            if let Some(gate) = optional_gate_ref.take() {
                let inputs_len = gate.input.len();

                for p in 0..inputs_len {
                    self.delete_wire_at_pin(gate_idx, p, PinType::Input);
                }
                let outputs_len = gate.output.len();
                for p in 0..outputs_len {
                    self.delete_wire_at_pin(gate_idx, p, PinType::Output);
                }
                self.circuit.gates_freed[gate_idx] = true;
            }
        }
    }

    fn delete_wire_at_pin(&mut self, g: usize, p: usize, t: PinType) {
        if let Some(gate) = &mut self.circuit.gates[g] {
            let wire_index = gate.get_pin(p, t).wire_index;
            if wire_index.is_some() {
                let wire = &mut self.circuit.wires_meta[wire_index.unwrap()];
                // if is a source pin of the wire
                if (wire.source.gate_index == g && wire.source.pin_index == p)
                // if is the only connected pin to wire
                || wire.connections.len() == 1 && wire.connections[0].gate_index == g && wire.connections[0].pin_index == p
                {
                    // remove the wire
                    self.circuit.remove_wire(wire_index.unwrap());
                // if there are more connections to wire
                } else {
                    let element_index: Option<usize> = wire.connections.find_pin_index(g, p);
                    if element_index.is_some() {
                        // remove from wire
                        wire.connections.remove(element_index.unwrap());
                    } else {
                        panic!(
                            "pin {p} of gate {g} is connected to {} but is not source or connection",
                            wire_index.unwrap()
                        );
                    }
                }
            }
        }
    }
}
