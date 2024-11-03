use sdl2::{pixels::Color, rect::Point};

// various drawing utilities

pub fn interpolate_color(start: Color, stop: Color, progress: f32) -> Color {
    let r = (start.r as f32 + (stop.r as f32 - start.r as f32) * progress) as u8;
    let g = (start.g as f32 + (stop.g as f32 - start.g as f32) * progress) as u8;
    let b = (start.b as f32 + (stop.b as f32 - start.b as f32) * progress) as u8;
    let a = (start.a as f32 + (stop.a as f32 - start.a as f32) * progress) as u8;
    Color::RGBA(r, g, b, a)
}

/// points which traces the perimeter of a rectangle  
/// moves inward by inward_amount (0 indicates the outer perimeter)
pub fn center_seeking_rect_points(inward_amount: i32, size: (u32, u32)) -> [Point; 5] {
    [
        Point::new(inward_amount, inward_amount),
        Point::new(size.0 as i32 - 1 - inward_amount, inward_amount),
        Point::new(
            size.0 as i32 - 1 - inward_amount,
            size.1 as i32 - 1 - inward_amount,
        ),
        Point::new(inward_amount, size.1 as i32 - 1 - inward_amount),
        Point::new(inward_amount, inward_amount),
    ]
}

pub fn up_left_center_seeking_rect_points(inward_amount: i32, size: (u32, u32)) -> [Point; 3] {
    [
        Point::new(size.0 as i32 - 1 - inward_amount, inward_amount),
        Point::new(inward_amount, inward_amount),
        Point::new(inward_amount, size.1 as i32 - 1 - inward_amount),
    ]
}

pub fn bottom_right_center_seeking_rect_points(inward_amount: i32, size: (u32, u32)) -> [Point; 3] {
    [
        Point::new(size.0 as i32 - 1 - inward_amount, inward_amount),
        Point::new(
            size.0 as i32 - 1 - inward_amount,
            size.1 as i32 - 1 - inward_amount,
        ),
        Point::new(inward_amount, size.1 as i32 - 1 - inward_amount),
    ]
}
