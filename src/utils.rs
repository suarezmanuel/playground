use crate::types::circuit::*;
use crate::types::gate_type::*;
use crate::types::keys::*;
use crate::types::pin_type::*;
use macroquad::prelude::*;
use macroquad::prelude::Rect;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;
use std::io::{Read, Write};

pub const FONT_SIZE: u16 = 32;

pub fn camera_view_rect(camera: &Camera2D) -> Rect {
    let tl = camera.screen_to_world(Vec2::new(0.0, 0.0));
    let br = camera.screen_to_world(Vec2::new(screen_width(), screen_height()));

    let minx = tl.x.min(br.x);
    let miny = tl.y.min(br.y);
    let maxx = tl.x.max(br.x);
    let maxy = tl.y.max(br.y);

    Rect {
        x: minx,
        y: miny,
        w: maxx - minx,
        h: maxy - miny,
    }
}

pub fn draw_grid(camera: &Camera2D, base_zoom: Vec2) {
    let MAX_ZOOM = 100.0;
    let max_zoom = (base_zoom.x * MAX_ZOOM, base_zoom.y * (1.0 / MAX_ZOOM));
    let min_zoom = (base_zoom.x * (1.0 / MAX_ZOOM), base_zoom.y * MAX_ZOOM);
    let zoom_percentage = ((camera.zoom.x - min_zoom.0) / (base_zoom.x - min_zoom.0))
        .max(0.01)
        .min(1.0);
    // println!("{}", zoom_percentage);

    let mut scale: f32 = 8.0;
    let rect = camera_view_rect(&camera);

    let mut tile_count: u32 = ((rect.w / scale) * (rect.h / scale)).round() as u32; // estimate
    while tile_count > 8_000 {
        scale *= 2.0;
        tile_count = ((rect.w / scale) * (rect.h / scale)).round() as u32; // estimate
    }

    let thickness = 1.0 / zoom_percentage;

    let min_x = (rect.x / scale).floor() * scale;
    let max_x = rect.x + rect.w;
    let min_y = (rect.y / scale).floor() * scale;
    let max_y = rect.y + rect.h;

    let mut x = min_x;
    while x < max_x {
        draw_line(
            x as f32,
            min_y as f32,
            x as f32,
            max_y as f32,
            thickness as f32,
            GRAY,
        );
        x += scale;
    }

    let mut y = min_y;
    while y < max_y {
        draw_line(
            min_x as f32,
            y as f32,
            max_x as f32,
            y as f32,
            thickness as f32,
            GRAY,
        );
        y += scale;
    }
}

pub fn intersects(a: Rect, b: Rect) -> bool {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = a.right().min(b.right());
    let bottom = a.bottom().min(b.bottom());

    return right >= left && bottom >= top;
}

pub fn draw_gates(circuit: &Circuit, camera: &Camera2D) {
    set_camera(camera);
    let camera_view_rect = camera_view_rect(&camera);

    for (_, gate) in &circuit.gates {
        gate.draw(camera_view_rect);
    }
}

pub fn draw_wires(circuit: &mut Circuit, camera: &Camera2D) {
    set_camera(camera);

    for (wire_key, wire) in &circuit.wires {
        if let Some(source_gate) = circuit.gates.get(wire.source.gate_index).as_mut() {
            let Vec2 {
                x: start_x,
                y: start_y,
            } = source_gate
                .get_pin_rect(wire.source.pin_index, PinType::Output)
                .center();

            for connection in &wire.connections {
                if let Some(connection_gate) = circuit.gates.get(connection.gate_index).as_mut() {
                    // println!("connection pin index: {}", connection.pin_index);
                    let Vec2 { x: end_x, y: end_y } = connection_gate
                        .get_pin_rect(connection.pin_index, PinType::Input)
                        .center();

                    let color = match circuit.wires_read.get(wire_key).unwrap() {
                        true => YELLOW,
                        false => BLACK,
                    };

                    draw_line(start_x, start_y, end_x, end_y, 3.0, color);
                }
            }
        }
    }
}

pub fn draw_pins(circuit: &Circuit, camera: &Camera2D) {
    set_camera(camera);
    for (_, gate) in &circuit.gates {
        gate.draw_pins(camera_view_rect(&camera));
    }
}

pub fn draw_mouse_wire(
    circuit: &Circuit,
    camera: &Camera2D,
    gate_index: Option<GateKey>,
    pin_index: Option<usize>,
    pin_type: Option<PinType>,
) {
    match (gate_index, pin_index, pin_type) {
        (Some(gate_index), Some(pin_index), Some(pin_type)) => {
            if let Some(gate) = circuit.gates.get(gate_index).clone() {
                // this is fine because we only read and don't write
                let rect = gate.get_pin(pin_index, pin_type).rect;
                let Vec2 {
                    x: center_x,
                    y: center_y,
                } = rect.center();
                let mouse_world =
                    camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                draw_line(center_x, center_y, mouse_world.x, mouse_world.y, 3.0, BLACK);
            }
        }
        _ => {}
    };
}

pub fn draw_gate_over_mouse(camera: &Camera2D, rect: Rect, gate_type: &GateType, alpha: f32) {
    // just to be sure
    if intersects(rect, camera_view_rect(camera)) {
        let color = gate_type.color();
        let text = gate_type.text();

        draw_rectangle(rect.x, rect.y, rect.w, rect.h, color.with_alpha(0.5));
        let dims = measure_text(text, None, FONT_SIZE, 1.0);
        let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
        let ty = rect.y + rect.h * 0.5 + FONT_SIZE as f32 / 4.0;

        draw_text_ex(
            text,
            tx,
            ty,
            TextParams {
                font_size: FONT_SIZE,
                color: BLACK.with_alpha(alpha),
                ..Default::default()
            },
        );
    }
}

// A helper module to handle Rect serialization
pub mod rect_serde {
    use super::*;

    #[derive(Serialize, Deserialize)]
    struct RectSurrogate {
        x: f32, y: f32, w: f32, h: f32,
    }

    pub fn serialize<S>(rect: &Rect, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let s = RectSurrogate { x: rect.x, y: rect.y, w: rect.w, h: rect.h };
        s.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Rect, D::Error>
    where D: Deserializer<'de> {
        let s = RectSurrogate::deserialize(deserializer)?;
        Ok(Rect::new(s.x, s.y, s.w, s.h))
    }
}

pub fn save_to_file(circuit: &Circuit) -> std::io::Result<String> {
    // 1. Define the directory path relative to project root
    let save_dir = "tmp/saves";

    // 2. Create the directory if it doesn't exist
    // create_dir_all does nothing if the dir already exists, which is perfect
    fs::create_dir_all(save_dir)?;

    // 3. Find the highest existing save number
    let mut max_save_num = 0;

    // Read the directory
    if let Ok(entries) = fs::read_dir(save_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                // We only care about files, not subdirectories
                if path.is_file() {
                    // Get the filename string (e.g., "save5.json")
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        // Check if it starts with "save" and ends with ".json"
                        if filename.starts_with("save") && filename.ends_with(".json") {
                            // Extract the middle part ("5")
                            let num_part = filename
                                .trim_start_matches("save")
                                .trim_end_matches(".json");

                            // Try to parse it as a number
                            if let Ok(num) = num_part.parse::<u32>() {
                                if num > max_save_num {
                                    max_save_num = num;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Determine the next filename
    let next_save_num = max_save_num + 1;
    let file_path = format!("{}/save{}.json", save_dir, next_save_num);

    // 5. Serialize and Write
    let json_data = serde_json::to_string_pretty(circuit)?;
    let mut file = fs::File::create(&file_path)?;
    file.write_all(json_data.as_bytes())?;

    println!("Success: Saved circuit to {}", file_path);
    
    Ok(file_path)
}

pub fn load_from_file(file_path: &str) -> std::io::Result<Circuit> {
    match fs::read_to_string(file_path) {
        Ok(contents) => {
            let circuit: Circuit = serde_json::from_str(&contents)?;
            println!("Loaded circuit from {}", file_path);
            Ok(circuit)
        }
        Err(e) => {
            Err(e)
        }
    }
}