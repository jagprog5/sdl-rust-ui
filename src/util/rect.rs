/// NOT an sdl2::rect::FRect. this one has no restriction on members
#[derive(Debug, Clone, Copy)]
pub struct FRect {
    /// can be any value
    pub x: f32,
    /// can be any value
    pub y: f32,
    /// can be any value
    pub w: f32,
    /// can be any value
    pub h: f32,
}

impl Into<Option<sdl2::rect::Rect>> for FRect {
    fn into(self) -> Option<sdl2::rect::Rect> {
        let w = self.w.round();
        let h = self.h.round();
        if w < 1. || h < 1. {
            return None;
        }
        let x = self.x.round();
        let y = self.y.round();
        Some(sdl2::rect::Rect::new(
            x as i32,
            y as i32,
            w as u32,
            h as u32,
        ))
    }
}
