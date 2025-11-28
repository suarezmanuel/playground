use macroquad::prelude::*;

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
    let max_zoom = (base_zoom.x*MAX_ZOOM, base_zoom.y*(1.0/MAX_ZOOM));
    let min_zoom = (base_zoom.x*(1.0/MAX_ZOOM), base_zoom.y*MAX_ZOOM);
    let zoom_percentage = ((camera.zoom.x - min_zoom.0) / (base_zoom.x - min_zoom.0)).max(0.01).min(1.0);
    // println!("{}", zoom_percentage);

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

pub fn intersects(a: Rect, b: Rect) -> bool {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = a.right().min(b.right());
    let bottom = a.bottom().min(b.bottom());

    return right >= left && bottom >= top;
}