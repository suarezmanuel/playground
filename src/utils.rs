use macroquad::prelude::*;
use crate::types::gate_type::*;
use crate::types::pin_type::*;
use crate::types::circuit::*;

const FONT_SIZE: u16 = 32;

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

    for gate in &circuit.gates {
        let camera_view_rect = camera_view_rect(&camera);
        let rect = gate.rect;

        if intersects(rect, camera_view_rect) {
            let color = GateType::color(&gate.gate_type);
            let text = GateType::text(&gate.gate_type);

            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
            let dims = measure_text(text, None, FONT_SIZE, 1.0);
            let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
            let ty = rect.y + rect.h * 0.5 + dims.offset_y * 0.5 as f32;
            
            draw_text_ex(
                text,
                tx,
                ty,
                TextParams {
                    font_size: FONT_SIZE,
                    color: BLACK,
                    ..Default::default()
                },
            );
        }
    }
}

pub fn draw_wires(circuit: &Circuit, camera: &Camera2D) {
    set_camera(camera);

    for wire in &circuit.wires_meta {
        let Vec2 { x: start_x, y: start_y } = circuit.gates[wire.source.gate_index].get_pin_rect(wire.source.pin_index, PinType::Output).center();

        for connection in &wire.connections {
            // println!("connection pin index: {}", connection.pin_index);
            let Vec2 { x: end_x, y: end_y } = circuit.gates[connection.gate_index].get_pin_rect(connection.pin_index, PinType::Input).center();
        
            let color = match circuit.wires_read[wire.wire_index] {
                true => { YELLOW }
                false => { BLACK }
            };

            draw_line(start_x, start_y, end_x, end_y, 3.0, color);
        }
    }
}

pub fn draw_pins(circuit: &Circuit, camera: &Camera2D) {
    set_camera(camera);

    for gate in &circuit.gates {
        let camera_view_rect = camera_view_rect(&camera);

        let pins= gate.input.iter().chain(gate.output.iter());

        for pin in pins {
            let pin_rect = pin.rect;
            if intersects(pin_rect, camera_view_rect) {
                draw_rectangle(pin_rect.x, pin_rect.y, pin_rect.w, pin_rect.h, BLACK);
            }
        }
    }
}

pub fn draw_mouse_wire(
    circuit: &Circuit,
    camera: &Camera2D,
    gate_index: Option<usize>,
    pin_index: Option<usize>,
    pin_type: Option<PinType>,
) {
    match (gate_index, pin_index, pin_type) {
        (Some(gate_index), Some(pin_index), Some(pin_type)) => {
            let gate = circuit.gates[gate_index].clone(); // this is fine because we only read and don't write
            let rect = gate.get_pin(pin_index, pin_type).rect;
            let Vec2 {
                x: center_x,
                y: center_y,
            } = rect.center();
            let mouse_world =
                camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
            draw_line(center_x, center_y, mouse_world.x, mouse_world.y, 3.0, BLACK);
        }
        _ => {}
    };
}

pub fn draw_gate_over_mouse(
    camera: &Camera2D,
    rect: Rect,
    gate_type: &GateType,
    alpha: f32,
) {
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
