use crate::types::circuit::*;
use crate::types::gate::*;
use crate::types::gate_type::*;
use crate::types::keys::*;
use crate::types::pin_type::*;
use crate::types::wires::*;
use crate::utils::*;
use crate::ui::draw_ui;
use crate::utils::camera_view_rect;
use crate::utils::draw_grid;
use macroquad::prelude::*;
use rstar::{AABB, PointDistance, RTree, RTreeObject};
use std::collections::HashMap;
use std::fmt::Write;
use std::time::SystemTime;
use crate::utils::{save_to_file, load_from_file};
use std::io;


#[derive(Debug, PartialEq, Clone, Copy)]
struct SpatialBlockIndex {
    pub rect: Rect,
    pub index: GateKey,
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

const DRAG_MIN_DIST: f32 = 5.0;

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

#[derive(Clone, PartialEq)]
enum InputState {
    Idle,
    ChoosingGate {
        gate_type: GateType,
        gate_rotation: Rotation
    },
    DragCamera {
        start_world: Vec2,
    },
    ClickGate {
        initial_click_pos: Vec2,
        gate_key: GateKey,
    },
    DraggingGate {
        gate_id: GateKey,
    },
    Wiring {
        gate: GateKey,
        pin: usize,
        p_type: PinType,
    },
    PastingGates {
        sp_gates: Vec<SpatialBlockIndex>,
        initial_rect: Rect,
        mouse_rect: Rect
    },
    SelectingGates {
        start_pos: Vec2,
        selection: Vec<SpatialBlockIndex>,
        gates_rect: Rect,
        selection_rect: Rect,
    }, 
    SelectedGates {
        sp_gates: Vec<SpatialBlockIndex>,
        gates_rect: Rect,
    },
    ClickSelectedGates {
        initial_click_pos: Vec2,
        initial_gates_rect: Rect,
        sp_gates: Vec<SpatialBlockIndex>,
    },
    DraggingSelectedGates {
        initial_gates: Vec<SpatialBlockIndex>,
        sp_gates: Vec<SpatialBlockIndex>,
        initial_click_pos:  Vec2,
    },
}

impl InputState {
    pub fn to_string(&self) -> &str {
        match self {
            InputState::Idle => "idle",
            InputState::ChoosingGate { .. } => "choosing gate",
            InputState::DragCamera { .. } => "camera drag",
            InputState::ClickGate { .. } => "clicking gate",
            InputState::DraggingGate { .. } => "gate drag",
            InputState::Wiring { .. } => "wiring",
            InputState::PastingGates { ..} => "pasting gates",
            InputState::SelectingGates { .. } => "selecting gates",
            InputState::SelectedGates { .. } => "selected gates",
            InputState::ClickSelectedGates { .. } => "clicking selected gates",
            InputState::DraggingSelectedGates{ .. } => "dragging selecting gates",
        }
    }
}

pub struct Simulator {
    // Systems
    pub circuit: Circuit,
    pub camera: Camera2D,
    pub zoom_factor: f32,

    // Simulation settings
    pub emulate: bool,
    pub log_msg: String,

    // State
    tree: RTree<SpatialBlockIndex>,
    state: InputState,
    last_tick: SystemTime,
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

        // if self.emulate && (self.last_tick.elapsed().unwrap().as_millis() as i32 > 1000) {
        if self.emulate {
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
                gate_type: GateType::AND,
                gate_rotation: Rotation::Up,
            };
        } else if is_key_pressed(KeyCode::Key2) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::OR,
                gate_rotation: Rotation::Up,
            };
        } else if is_key_pressed(KeyCode::Key3) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::NOT,
                gate_rotation: Rotation::Up,
            };
        } else if is_key_pressed(KeyCode::Key4) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::XOR,
                gate_rotation: Rotation::Up,
            };
        } else if is_key_pressed(KeyCode::Key5) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::XNOR,
                gate_rotation: Rotation::Up,
            };
        } else if is_key_pressed(KeyCode::Key6) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::NOR,
                gate_rotation: Rotation::Up,
            };
        } else if is_key_pressed(KeyCode::Key7) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::NAND,
                gate_rotation: Rotation::Up,
            };
        } else if is_key_pressed(KeyCode::Key8) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::IN,
                gate_rotation: Rotation::Up,
            };
        } else if is_key_pressed(KeyCode::Key9) {
            self.state = InputState::ChoosingGate {
                gate_type: GateType::OUT,
                gate_rotation: Rotation::Up,
            };
        } else if is_key_pressed(KeyCode::R) {
            self.emulate = !self.emulate;
        } else if is_key_pressed(KeyCode::T) {
            self.circuit.reset_wires();
        } else if is_key_pressed(KeyCode::F) {
            println!("enter file name: ");
            let mut file_name: String = "".to_string();
            io::stdin()
                .read_line(&mut file_name) 
                .expect("Failed to read line");

            match save_to_file(&self.circuit, file_name.trim().to_string()) {
                Ok(path) => println!("Saved to {}", path),
                Err(e) => println!("Error saving: {}", e),
            }
        } else if is_key_pressed(KeyCode::L) {
            let mut input = "".to_string();
            println!("enter file name: ");
            io::stdin()
                .read_line(&mut input) 
                .expect("Failed to read line");

            let file_path = "tmp/saves/".to_string() + input.trim();

            println!("loading from: {}", &file_path);
            match load_from_file(&file_path) { // Pass &String as &str
                Ok(mut new_circuit) => {

                    for wire_key in new_circuit.wires.keys() {
                        new_circuit.wires_read.insert(wire_key, false);
                        new_circuit.wires_write.insert(wire_key, false);
                    }

                    println!("gates {:?} \nwires {:?} \nwires_r {:?} \nwires_w {:?} \nemulation_done {:?}", new_circuit.gates, new_circuit.wires, new_circuit.wires_read, new_circuit.wires_write, new_circuit.emulation_done);

                    self.circuit = new_circuit;
                    self.tree = RTree::new();
                    for (key, gate) in &self.circuit.gates {
                        self.tree.insert(SpatialBlockIndex {
                            rect: gate.rect,
                            index: key,
                        });
                    }
                    // Rebuild RTree here (omitted for brevity)
                    println!("Loaded successfully");
                }
                Err(e) => println!("Error loading: {}", e),
            }
        } 
    
        let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
        let mouse_world = self.camera.screen_to_world(mouse_screen);

        match self.state.clone() {
            InputState::Idle => {
                if is_key_pressed(KeyCode::LeftShift) {
                    self.state = InputState::SelectingGates { start_pos: mouse_world, gates_rect: Rect::new(mouse_world.x, mouse_world.y, 0.0, 0.0), selection_rect: Rect::new(mouse_world.x, mouse_world.y, 0.0, 0.0), selection: vec![]};
                }
            }
            InputState::ChoosingGate { .. } => {
                if let InputState::ChoosingGate { gate_rotation, ..} = &mut self.state {
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                        *gate_rotation = Rotation::Left;
                    } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                        *gate_rotation = Rotation::Right;
                    } else if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                        *gate_rotation = Rotation::Up;
                    } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                        *gate_rotation = Rotation::Down;
                    }
                }
            }
            InputState::DraggingGate { gate_id } => {

                let gate = self.circuit.gates.get_mut(gate_id).unwrap();

                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    gate.change_rotation(Rotation::Left);
                } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    gate.change_rotation(Rotation::Right);
                } else if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    gate.change_rotation(Rotation::Up);
                } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    gate.change_rotation(Rotation::Down);
                }
            }
            InputState::SelectedGates { sp_gates, gates_rect } => {
                // if something was selected, then when right click or click not on a gate
                if is_key_down(KeyCode::LeftControl) && is_key_down(KeyCode::C) {
                    self.state = InputState::PastingGates{ initial_rect: gates_rect, mouse_rect: gates_rect, sp_gates };
                } else if is_key_down(KeyCode::Escape) {
                    for key in sp_gates {
                        self.circuit.remove_gate(key.index);
                        // they are not on tree so no need to delete them from tree
                    }
                    self.state = InputState::Idle;
                }
            }
            _ => {}
        }
    }

    pub fn handle_mouse(&mut self) {
        let mouse_screen = Vec2::new(mouse_position().0, mouse_position().1);
        let mouse_world = self.camera.screen_to_world(mouse_screen);
        let mouse_aligned = mouse_world.div_euclid((64.0, 64.0).into()).floor().mul_add((64.0, 64.0).into(), (0.0, 0.0).into());

        // find out hover
        let hovered_gate_key = self
            .tree
            .locate_at_point(&[mouse_world.x, mouse_world.y])
            .and_then(|item| Some(item.index));

        let hovered_pin = if let Some(g_idx) = hovered_gate_key {
            self.find_hovered_pin(g_idx, mouse_world)
        } else {
            None
        };

        match self.state.clone() {
            InputState::Idle => {
                if is_mouse_button_pressed(MouseButton::Right) {
                    if let Some((g_idx, p_idx, p_type)) = hovered_pin {
                        self.delete_wire_at_pin(g_idx, p_idx, p_type);
                    } else if let Some(g_idx) = hovered_gate_key {
                        self.tree.remove(&SpatialBlockIndex {
                            rect: self.circuit.gates.get(g_idx).as_ref().unwrap().rect,
                            index: g_idx,
                        });
                        self.circuit.remove_gate(g_idx);
                    }
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some((g, p, t)) = hovered_pin {
                        // Wiring starts immediately (pins are small, no drag check needed usually)
                        self.state = InputState::Wiring {
                            gate: g,
                            pin: p,
                            p_type: t,
                        };
                    } else if let Some(gate_key) = hovered_gate_key {
                        self.state = InputState::ClickGate { initial_click_pos: mouse_world, gate_key }
                    }
                } else if is_mouse_button_down(MouseButton::Left) {
                    // If we didn't click a gate (background), start Camera Drag
                    // (You can add a threshold here too if you want, but immediate is usually fine for panning)
                    self.state = InputState::DragCamera {
                        start_world: mouse_world,
                    };
                } 
            }
            InputState::ChoosingGate { gate_type, gate_rotation } => {
                if is_mouse_button_pressed(MouseButton::Right) {
                    self.state = InputState::Idle;
                } else if is_mouse_button_pressed(MouseButton::Left) && hovered_gate_key.is_none() {
                    self.place_gate(mouse_world, gate_type, gate_rotation);
                    self.state = InputState::Idle;
                }
            }
            InputState::DragCamera { start_world } => {
                if is_mouse_button_down(MouseButton::Left) {
                    let current_world = self.camera.screen_to_world(mouse_screen);
                    self.camera.target += start_world - current_world;
                } else {
                    self.state = InputState::Idle;
                }
            }
            InputState::ClickGate { initial_click_pos, gate_key } => {
                // if left click is released 
                if is_mouse_button_released(MouseButton::Left) {
                    // mouse_world is equal to initial_click_pos its a click (what if they with luck move the mouse back to start without going farther than MIN)
                    if mouse_world == initial_click_pos {
                        if let Some(gate) = self.circuit.gates.get_mut(gate_key) && gate.gate_type == GateType::IN {
                            gate.active = !gate.active;
                        }
                    }
                    self.state = InputState::Idle;
                } else {
                    // if distance from initial_click_pos to mouse_world is more than MIN
                    let dist = initial_click_pos.distance(mouse_world);
                    if dist > DRAG_MIN_DIST {
                        // Remove from tree for the duration of the drag
                        if let Some(gate) = self.circuit.gates.get(gate_key) {
                            self.tree.remove(&SpatialBlockIndex {
                                rect: gate.rect,
                                index: gate_key,
                            });
                        }
                        self.state = InputState::DraggingGate { gate_id: gate_key };
                    }
                }
            }
            InputState::DraggingGate { gate_id } => {
                let gate = &mut self.circuit.gates.get_mut(gate_id);
                if let Some(gate) = gate {
                    gate.offset(Vec2 {
                        x: mouse_world.x - gate.rect.x - gate.rect.w * 0.5,
                        y: mouse_world.y - gate.rect.y - gate.rect.h * 0.5,
                    });

                    let envelope = AABB::from_corners(
                        [mouse_world.x.align(64.0)+1.0,  mouse_world.y.align(64.0)+1.0],
                        [mouse_world.x.align(64.0) + gate.rect.w-2.0, mouse_world.y.align(64.0) + gate.rect.h-2.0],
                    );

                    let sp_intersection: Vec<SpatialBlockIndex> = self.tree
                    .locate_in_envelope_intersecting(&envelope)
                    .map(|item| *item) 
                    .collect();

                    if is_mouse_button_released(MouseButton::Left) && sp_intersection.is_empty() {
                        // Snap to grid on release
                        let grid_x = mouse_world.x.align(64.0);
                        let grid_y = mouse_world.y.align(64.0);

                        gate.offset(Vec2 {
                            x: grid_x - gate.rect.x,
                            y: grid_y - gate.rect.y,
                        });
                        
                        // Re-insert into tree
                        self.tree.insert(SpatialBlockIndex {
                            rect: gate.rect,
                            index: gate_id,
                        });
                        self.state = InputState::Idle;
                    }
                }
            }
            InputState::Wiring { gate, pin, p_type } => {
                if is_mouse_button_pressed(MouseButton::Right) {
                    self.state = InputState::Idle;
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some((to_g, to_p, to_t)) = hovered_pin {
                        if !((gate == to_g)
                            || (gate == to_g && pin == to_p)
                            || (p_type.to_string() == to_t.to_string()))
                        {
                            self.circuit
                                .connect_wire(gate, to_g, pin, p_type, to_p, to_t);
                            self.state = InputState::Idle;
                        }
                    }
                }
            }
            // read from start_pos
            InputState::SelectingGates { start_pos, selection, gates_rect, .. } => {

                if is_mouse_button_released(MouseButton::Left) || is_mouse_button_pressed(MouseButton::Left) {
                    if !selection.is_empty() {
                        self.state = InputState::SelectedGates { sp_gates: selection, gates_rect };
                    } else  {
                        self.state = InputState::Idle;
                    }
                } else if is_mouse_button_pressed(MouseButton::Right) {
                    self.state = InputState::Idle;
                } else {
                    let current_rect = Rect::new(
                        start_pos.x.min(mouse_world.x),
                        start_pos.y.min(mouse_world.y),
                        (mouse_world.x - start_pos.x).abs(),
                        (mouse_world.y - start_pos.y).abs(),
                    );

                    let envelope = AABB::from_corners(
                        [current_rect.x, current_rect.y],
                        [current_rect.x + current_rect.w, current_rect.y + current_rect.h],
                    );

                    let spatial_gates: Vec<SpatialBlockIndex> = self.tree
                    .locate_in_envelope_intersecting(&envelope)
                    .map(|item| *item) // Extract the GateKey
                    .collect();

                    let current_gates_rect;

                    if spatial_gates.len() > 0 {
                        
                        let mut min_x = spatial_gates[0].rect.x;
                        let mut min_y = spatial_gates[0].rect.y;
                        let mut max_x = spatial_gates[0].rect.x + spatial_gates[0].rect.w;
                        let mut max_y = spatial_gates[0].rect.y + spatial_gates[0].rect.h;

                        for idx in &spatial_gates {
                            min_x = min_x.min(idx.rect.x);
                            min_y = min_y.min(idx.rect.y);
                            max_x = max_x.max(idx.rect.x + idx.rect.w);
                            max_y = max_y.max(idx.rect.y + idx.rect.h);
                        }

                        current_gates_rect = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);

                    
                        if let InputState::SelectingGates { gates_rect, ..} = &mut self.state {
                            *gates_rect = current_gates_rect;
                        }
                    }

                    if let InputState::SelectingGates { selection_rect, selection, ..} = &mut self.state {
                        *selection_rect = current_rect;
                        *selection = spatial_gates;
                    }
                }
            }

            InputState::PastingGates { sp_gates, initial_rect, mouse_rect } => {

                if is_mouse_button_pressed(MouseButton::Right) {
                    self.state = InputState::Idle;
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    
                    let mut intersection = false;
                    for sp in sp_gates.clone() {
                        let gate = &mut self.circuit.gates.get_mut(sp.index).unwrap();

                        let rx = sp.rect.x + mouse_rect.x - initial_rect.x;
                        let ry = sp.rect.y + mouse_rect.y - initial_rect.y;
                        let envelope = AABB::from_corners(
                            [rx+1.0, ry+1.0],
                            [rx + gate.rect.w-2.0, ry + gate.rect.h-2.0],
                        );

                        let spatial_gates: Vec<SpatialBlockIndex> = self.tree
                        .locate_in_envelope_intersecting(&envelope)
                        .map(|item| *item) 
                        .collect();

                        // doesn't restrict dragging to a rectangle
                        if !spatial_gates.is_empty() { intersection = true; }
                    }

                    if !sp_gates.is_empty() && !intersection {

                        let dx = mouse_rect.x - initial_rect.x;
                        let dy = mouse_rect.y - initial_rect.y;
                        let mut gate_map: HashMap<GateKey, GateKey> = HashMap::new();
                        let mut wire_map: HashMap<WireKey, WireKey> = HashMap::new();

                        for SpatialBlockIndex{rect, index} in &sp_gates {
                            let old_gate = self.circuit.gates.get(*index).unwrap();
                            let rotation = old_gate.rotation.clone();
                            let gate_type = old_gate.gate_type.clone();
                            let new_gate_rect = Rect::new(rect.x + dx, rect.y + dy, rect.w, rect.h);
                            let new_gate_id = self.circuit.gates.insert(Gate::new(new_gate_rect, rotation, gate_type));
                            self.tree.insert(SpatialBlockIndex { rect: new_gate_rect, index: new_gate_id });
                            gate_map.insert(*index, new_gate_id);
                        
                            let new_gate_index = gate_map.get_mut(index).unwrap();

                            // create wires that have a source in 'intersection'
                            let mut wires_to_create = Vec::new();
                            for (old_key, old_wire) in &self.circuit.wires {
                                if old_wire.source.gate_index == *index {
                                    wires_to_create.push((old_key, old_wire.source.pin_index));
                                }
                            }

                            for (old_key, pin_idx) in wires_to_create {
                                // connect the source pins
                                let new_wire = Wire::new(
                                    Connection { 
                                        gate_index: *new_gate_index, 
                                        pin_index: pin_idx 
                                    }, 
                                    vec![] 
                                );
                                
                                let new_wire_id = self.circuit.wires.insert(new_wire);
                                self.circuit.gates.get_mut(*new_gate_index).unwrap().output[pin_idx].wire_index = Some(new_wire_id);
                                
                                wire_map.insert(old_key, new_wire_id);
                                self.circuit.wires_read.insert(new_wire_id, false);
                                self.circuit.wires_write.insert(new_wire_id, false);
                            }
                        }

                        for SpatialBlockIndex { rect: _, index } in &sp_gates {
                            let new_gate_id = *gate_map.get(index).unwrap();
                            let old_inputs = self.circuit.gates.get(*index).unwrap().input.clone();

                            for (pin_index, old_pin) in old_inputs.iter().enumerate() {
                                if let Some(wire_index) = &old_pin.wire_index {
                                    let old_wire_source = self.circuit.wires.get(*wire_index).unwrap().source.clone();
                                    // the wire might not have a source in 'intersection'
                                    if let Some(new_wire_id) = wire_map.get(wire_index) {
                                        // if it has an input in 'intersection'
                                        let new_wire = self.circuit.wires.get_mut(*new_wire_id).unwrap();
                                        // if the connected pin_index is not a source
                                        if old_wire_source.pin_index != old_pin.index || old_wire_source.gate_index != *index {
                                            new_wire.connections.push(Connection{ pin_index: pin_index, gate_index: new_gate_id });
                                            self.circuit.gates.get_mut(new_gate_id).unwrap().input[pin_index].wire_index = Some(*new_wire_id); 
                                        }
                                    }
                                }
                            }
                        }
                        self.state = InputState::Idle;
                    }
                }
                // write to rect
                else if let InputState::PastingGates { mouse_rect, .. } = &mut self.state {
                    // rect should already be aligned
                    mouse_rect.x = mouse_aligned.x - (mouse_rect.w * 0.5).align(64.0);
                    mouse_rect.y = mouse_aligned.y - (mouse_rect.h * 0.5).align(64.0);
                }
            }
            InputState::SelectedGates { sp_gates, gates_rect } => {
                if is_mouse_button_pressed(MouseButton::Right) {
                    self.state = InputState::Idle;
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    for gate in sp_gates.clone() {
                        if gate.rect.contains(mouse_world) {
                            self.state = InputState::ClickSelectedGates{ initial_click_pos: mouse_world, initial_gates_rect: gates_rect, sp_gates: sp_gates.clone()} // i guess this will be very slow
                        }
                    }
                }
            }
            InputState::ClickSelectedGates { initial_click_pos, initial_gates_rect, sp_gates } => {
                // if left click is released 
                if is_mouse_button_released(MouseButton::Left) {
                    self.state = InputState::Idle;
                } else {
                    // if distance from initial_click_pos to mouse_world is more than MIN
                    let dist = initial_click_pos.distance(mouse_world);
                    if dist > DRAG_MIN_DIST {
                        //remove from tree all dragging gates
                        for SpatialBlockIndex{ index: gate_key, .. } in sp_gates.clone() {
                            if let Some(gate) = self.circuit.gates.get(gate_key) {
                                self.tree.remove(&SpatialBlockIndex {
                                    rect: gate.rect,
                                    index: gate_key,
                                });
                            }
                        }
                        self.state = InputState::DraggingSelectedGates { sp_gates: sp_gates.clone(), initial_gates: sp_gates.clone(), initial_click_pos };
                    }
                }
            }
            // draw them after drawing other gates
            InputState::DraggingSelectedGates { initial_gates, sp_gates, initial_click_pos } => {

                // place when left mouse released or pressed
                let mut intersection = false;
                for (index, SpatialBlockIndex{ index: gate_key, .. }) in sp_gates.iter().enumerate() {
                    let gate = &mut self.circuit.gates.get_mut(*gate_key).unwrap();
                    let initial_gate_rect = initial_gates[index].rect;

                    let rx = initial_gate_rect.x + (mouse_world.x - initial_click_pos.x + gate.rect.w * 0.5).align(64.0);
                    let ry = initial_gate_rect.y + (mouse_world.y - initial_click_pos.y + gate.rect.h * 0.5).align(64.0);
                    let envelope = AABB::from_corners(
                        [rx+1.0, ry+1.0],
                        [rx + gate.rect.w-2.0, ry + gate.rect.h-2.0],
                    );

                    let spatial_gates: Vec<SpatialBlockIndex> = self.tree
                    .locate_in_envelope_intersecting(&envelope)
                    .map(|item| *item) 
                    .collect();

                    // doesn't restrict dragging to a rectangle
                    if !spatial_gates.is_empty() { intersection = true; }

                    gate.offset(Vec2::new(
                        initial_gate_rect.x + mouse_world.x - initial_click_pos.x - gate.rect.x,
                        initial_gate_rect.y + mouse_world.y - initial_click_pos.y - gate.rect.y
                    ));
                }

                if is_mouse_button_released(MouseButton::Left) && !intersection {
                    for (index, SpatialBlockIndex{ index: gate_key, .. }) in sp_gates.iter().enumerate() {
                        let gate = &mut self.circuit.gates.get_mut(*gate_key).unwrap();
                        let initial_gate_rect = initial_gates[index].rect;
                        // align the gate
                        gate.offset(Vec2::new(
                            initial_gate_rect.x + (mouse_world.x - initial_click_pos.x + gate.rect.w * 0.5).align(64.0) - gate.rect.x,
                            initial_gate_rect.y + (mouse_world.y - initial_click_pos.y + gate.rect.h * 0.5).align(64.0) - gate.rect.y
                        ));
                        // Re-insert into tree
                        self.tree.insert(SpatialBlockIndex {
                            rect: gate.rect,
                            index: *gate_key,
                        });
                        self.state = InputState::Idle;
                    }
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

        let mouse_world =  self.camera.screen_to_world(vec2(mouse_position().0, mouse_position().1));
        
        // draw transparent gate at mouse if choosing gate
        match self.state.clone() {
            InputState::ChoosingGate { gate_type, gate_rotation } => {
                let snap_pos = vec2((mouse_world.x / 64.).floor() * 64., (mouse_world.y / 64.).floor() * 64.);
                let r: Rect = Rect::new(snap_pos.x, snap_pos.y, 64., 64.);
    
                crate::utils::draw_gate_over_mouse(&self.camera, r, gate_type, gate_rotation, 0.5);
            }

            InputState::DraggingGate { gate_id } => {
                if let Some(gate) = self.circuit.gates.get(gate_id).as_ref() {
                    let snap_pos = vec2((mouse_world.x / 64.).floor() * 64., (mouse_world.y / 64.).floor() * 64.);
                    let r = Rect::new(snap_pos.x, snap_pos.y, 64., 64.);

                    crate::utils::draw_gate_over_mouse(&self.camera, r, gate.gate_type.clone(), gate.rotation.clone(), 0.5);
                }
            }

            InputState::DraggingSelectedGates { initial_gates, sp_gates, initial_click_pos } => {
                // place when left mouse released or pressed
                for (index, SpatialBlockIndex{ index: gate_key, .. }) in sp_gates.iter().enumerate() {
                    let gate = &mut self.circuit.gates.get_mut(*gate_key).unwrap().clone();
                    let initial_gate_rect = initial_gates[index].rect;

                    gate.offset(Vec2::new(
                        initial_gate_rect.x + (mouse_world.x - initial_click_pos.x + gate.rect.w * 0.5).align(64.0) - gate.rect.x,
                        initial_gate_rect.y + (mouse_world.y - initial_click_pos.y + gate.rect.h * 0.5).align(64.0) - gate.rect.y
                    ));

                    gate.draw(camera_view_rect(&self.camera), gate.gate_type.color().with_alpha(0.5));
                    gate.draw_pins(&self.circuit, camera_view_rect(&self.camera), BLACK.with_alpha(0.5));

                    // fix the previous offset, for the utils::draw_gates
                    gate.offset(Vec2::new(
                        initial_gate_rect.x + mouse_world.x - initial_click_pos.x - gate.rect.x,
                        initial_gate_rect.y + mouse_world.y - initial_click_pos.y - gate.rect.y
                    ));
                }
            }

            InputState::PastingGates { sp_gates, initial_rect, mouse_rect } => {
                for sp in sp_gates.clone() {
                    if let Some(gate) = self.circuit.gates.get_mut(sp.index) {
                        gate.offset(Vec2::new(sp.rect.x + mouse_rect.x - initial_rect.x - gate.rect.x, sp.rect.y + mouse_rect.y - initial_rect.y - gate.rect.y));
                    }
                }

                for sp in sp_gates.clone() {
                    if let Some(gate) = self.circuit.gates.get(sp.index) {

                        gate.draw(camera_view_rect(&self.camera), gate.gate_type.color().with_alpha(0.8));
                        gate.draw_wires(&self.circuit, camera_view_rect(&self.camera));
                        gate.draw_pins(&self.circuit, camera_view_rect(&self.camera), BLACK.with_alpha(0.8));
                    }
                }

                for sp in sp_gates.clone() {
                    if let Some(gate) = self.circuit.gates.get_mut(sp.index) {
                        gate.offset(Vec2::new(sp.rect.x - gate.rect.x, sp.rect.y - gate.rect.y));
                    }
                }
            }
            _ => {}
        }

        crate::utils::draw_gates(&self.circuit, &self.camera);
        crate::utils::draw_wires(&mut self.circuit, &self.camera);
        crate::utils::draw_pins(&self.circuit, &self.camera);

        // draw hover gate
        match self.state.clone() {
            InputState::DraggingGate { gate_id: id } => {
               if let Some(gate) = self.circuit.gates.get(id) {
                    gate.draw(camera_view_rect(&self.camera), gate.gate_type.color());
                    gate.draw_wires(&self.circuit, camera_view_rect(&self.camera));
                    gate.draw_pins(&self.circuit, camera_view_rect(&self.camera), BLACK);
               }
            }
            InputState::Wiring { gate, pin, p_type } => {
                crate::utils::draw_mouse_wire(
                    &self.circuit,
                    &self.camera,
                    Some(gate),
                    Some(pin),
                    Some(p_type),
                );
            }
            InputState::SelectingGates { selection_rect, .. } => {
                draw_rectangle_lines(selection_rect.x, selection_rect.y, selection_rect.w, selection_rect.h, 3.0, BLACK);
            }
            InputState::PastingGates { sp_gates, .. } => {
                for sp_index in sp_gates {
                    let gate = self.circuit.gates.get(sp_index.index).unwrap();
                    gate.draw(camera_view_rect(&self.camera), gate.gate_type.color().lerp(BLUE, 0.5));
                    gate.draw_wires(&self.circuit, camera_view_rect(&self.camera));
                    gate.draw_pins(&self.circuit, camera_view_rect(&self.camera), BLACK.lerp(BLUE, 0.5));
                }
            }
            InputState::SelectedGates { sp_gates: indices, .. } => {
                for index in indices {
                    let gate = self.circuit.gates.get(index.index).unwrap(); // should be a valid index
                    gate.draw(camera_view_rect(&self.camera), gate.gate_type.color().lerp(BLUE, 0.5));
                    gate.draw_wires(&self.circuit, camera_view_rect(&self.camera));
                    gate.draw_pins(&self.circuit, camera_view_rect(&self.camera), BLACK.lerp(BLUE, 0.5));
                }
            }
            InputState::DraggingSelectedGates { initial_gates, sp_gates, initial_click_pos } => {
                // draw it over everything else
                for (index, SpatialBlockIndex{ index: gate_key, .. }) in sp_gates.iter().enumerate() {
                    let gate = &mut self.circuit.gates.get_mut(*gate_key).unwrap().clone();
                    let initial_gate_rect = initial_gates[index].rect;

                    gate.offset(Vec2::new(
                        initial_gate_rect.x + mouse_world.x - initial_click_pos.x - gate.rect.x,
                        initial_gate_rect.y + mouse_world.y - initial_click_pos.y - gate.rect.y
                    ));

                    gate.draw(camera_view_rect(&self.camera), gate.gate_type.color());
                    gate.draw_pins(&self.circuit, camera_view_rect(&self.camera), BLACK);
                }
            }
            _ => {}
        }


        set_default_camera();

        write!(self.log_msg, "input state: {} |", self.state.to_string()).unwrap();

        draw_ui(self.log_msg.clone());
    }

    fn place_gate(&mut self, pos: Vec2, gate_type: GateType, gate_rotation: Rotation) {
        let snap_x = pos.x.align(64.0);
        let snap_y = pos.y.align(64.0);
        let rect = Rect::new(snap_x, snap_y, 64.0, 64.0);

        let idx = self
            .circuit
            .gates
            .insert(Gate::new(rect, gate_rotation, gate_type));
        self.tree.insert(SpatialBlockIndex { rect, index: idx });
    }

    fn find_hovered_pin(
        &self,
        gate_idx: GateKey,
        mouse_world: Vec2,
    ) -> Option<(GateKey, usize, PinType)> {
        if let Some(gate) = &self.circuit.gates.get(gate_idx) {
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

    fn delete_wire_at_pin(&mut self, g: GateKey, p: usize, t: PinType) {
        if let Some(gate) = self.circuit.gates.get_mut(g) {
            let wire_index = gate.get_pin(p, t).wire_index;

            if let Some(wirekey) = wire_index {
                if let Some(wire) = self.circuit.wires.get_mut(wirekey) {
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
                        if let Some(idx) = element_index {
                            // remove from wire
                            wire.connections.remove(idx);
                            gate.input[p].wire_index = None;
                        } else {
                            panic!(
                                "pin {p} of gate {:?} is connected to {:?} but is not source or connection",
                                g, wire_index
                            );
                        }
                    }
                }
            }
        }
    }
}
