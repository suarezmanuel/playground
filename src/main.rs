use macroquad::prelude::*;
use std::time::{SystemTime};
mod types;
mod gates;
mod events;
use types::gate;
use gates::*;
use events::*;

fn camera_view_rect(camera: &Camera2D) -> Rect {
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

fn draw_grid(camera: &Camera2D, base_zoom: Vec2) {

    let MAX_ZOOM = 100.0;
    let max_zoom = (base_zoom.x*MAX_ZOOM, base_zoom.y*(1.0/MAX_ZOOM));
    let min_zoom = (base_zoom.x*(1.0/MAX_ZOOM), base_zoom.y*MAX_ZOOM);
    let zoom_percentage = ((camera.zoom.x - min_zoom.0) / (base_zoom.x - min_zoom.0)).max(0.01).min(1.0);
    println!("{}", zoom_percentage);

    let mut scale : f32 = 10.0;
    let rect = camera_view_rect(&camera);

    let mut tile_count : u32 = ((rect.w / scale) * (rect.h / scale)).round() as u32; // estimate
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
        draw_line(x as f32, min_y as f32, x as f32, max_y as f32, thickness as f32, GRAY);
        x += scale;
    }
    
    let mut y = min_y;
    while y < max_y {
        draw_line(min_x as f32, y as f32, max_x as f32, y as f32, thickness as f32, GRAY);
        y += scale;
    }
}

fn intersects(a: Rect, b: Rect) -> bool {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = a.right().min(b.right());
    let bottom = a.bottom().min(b.bottom());

    return right >= left && bottom >= top;
}

#[macroquad::main("My Macroquad Demo")]
async fn main() {

    let mut counter = 0;
    let mut fps: i32 = 0;
    let mut now = SystemTime::now();
    let FPS_REST = 10;
    
    let mut rectangles: Vec<Rect>  = Vec::new();
    for i in (0..1) {
        for j in (0..1) {
            rectangles.push({Rect{w: 100.0, h: 100.0, x: i as f32, y: j as f32}});
        }
    }
    let mut camera: Camera2D = Camera2D::from_display_rect(Rect{w: screen_width(), h: screen_height(), x: 0.0, y: 0.0});
    let mut starting_drag_world: Option<Vec2> = None;
    let base_zoom = camera.zoom;
    
    loop {
        if counter == FPS_REST {
            let total_time_elapsed = now.elapsed().unwrap().as_millis() as i32;
            fps = (FPS_REST * 1000) / total_time_elapsed;
            counter = 0;
            now = SystemTime::now();
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            // remember the world point under cursor when starting drag
            starting_drag_world = Some(camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1)));
        } else if is_mouse_button_down(MouseButton::Left) {
            if let Some(start_world) = starting_drag_world {
                let current_world = camera.screen_to_world(Vec2::new(mouse_position().0, mouse_position().1));
                // move camera target so the world point under cursor follows the drag
                camera.target += start_world - current_world;
            }
        } else {
            starting_drag_world = None;
        }
        
        let (_sx, sy) = mouse_wheel();
        if sy != 0.0 {
            let sensitivity = 0.001; // tune
            let MAX_ZOOM = 100.0;
            // clamp factor to avoid zero/negative scaling
            let factor = (1.0 + sy * sensitivity).max(0.01); // >1 zooms in, <1 zooms out

            let mut new_zoom:Vec2 = camera.zoom * Vec2::new(factor, factor);
            new_zoom.x = new_zoom.x.clamp(base_zoom.x*(1.0/MAX_ZOOM), base_zoom.x*MAX_ZOOM);
            new_zoom.y = new_zoom.y.clamp(base_zoom.y*MAX_ZOOM,base_zoom.y*(1.0/MAX_ZOOM));

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

        draw_grid(&camera, base_zoom);

        draw_line(50.0, 50.0, 200.0, 150.0, 5.0, RED);
        draw_circle(100.0, screen_height() - 100.0, 40.0, YELLOW);
        // let mut draw_counter = 0;
        let camera_view_rect = camera_view_rect(&camera);
        for rectangle in &rectangles {
            if intersects(*rectangle, camera_view_rect) {
                draw_rectangle(rectangle.x, rectangle.y, rectangle.w,rectangle.h, RED);
                // draw_counter += 1;
            }
        }
        // println!("{}", draw_counter);
        set_default_camera();
        draw_text(&format!("{}", fps), 20.0, 30.0, 40.0, WHITE);
        // Advance to the next frame
        next_frame().await
    }
}