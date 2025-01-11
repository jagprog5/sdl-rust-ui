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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_tie_tests() {
        // whole numbers unaffected
        assert_eq!(rect_position_round(1.), 1);
        assert_eq!(rect_position_round(2.), 2);
        assert_eq!(rect_position_round(0.), 0);
        assert_eq!(rect_position_round(-1.), -1);
        assert_eq!(rect_position_round(-2.), -2);

        // typical rounding is fine

        // close to 0
        assert_eq!(rect_position_round(0.00001), 0);
        assert_eq!(rect_position_round(-0.00001), 0);

        // close
        assert_eq!(rect_position_round(1.0001), 1);
        assert_eq!(rect_position_round(0.9999), 1);

        // far
        assert_eq!(rect_position_round(1.4999), 1);
        assert_eq!(rect_position_round(0.5001), 1);

        // close (negative)
        assert_eq!(rect_position_round(-1.0001), -1);
        assert_eq!(rect_position_round(-0.9999), -1);

        // far negative
        assert_eq!(rect_position_round(-1.4999), -1);
        assert_eq!(rect_position_round(-0.5001), -1);

        // rounding away from 0 on positive side unaffected
        assert_eq!(rect_position_round(0.5), 1);
        assert_eq!(rect_position_round(1.5), 2); 

        // checks special functionality (rounding up and not away from zero)
        assert_eq!(rect_position_round(-0.5), 0);
        assert_eq!(rect_position_round(-1.5), -1);
        assert_eq!(rect_position_round(-2.5), -2);
    }
}

/// round, but if exactly between numbers, always round up.
/// this is required or else a 1 pixel gap can appear
/// 
/// this should be used in contexts where it should match the conversion to
/// sdl2::rect::Rect from crate::util::rect::FRect
pub fn rect_position_round(i: f32) -> i32 {
    let i_whole = i.trunc();
    let i_frac = i - i_whole;
    if i_frac != -0.5 {
        i.round() as i32
    } else {
        i_whole as i32
    }
}

/// round, only giving positive output
/// 
/// this should be used in contexts where it should match the conversion to
/// sdl2::rect::Rect from crate::util::rect::FRect
pub fn rect_len_round(i: f32) -> Option<u32> {
    let i = i.round();
    if i < 1. { // must be positive
        None
    } else {
        Some(i as u32)
    }
}

impl Into<Option<sdl2::rect::Rect>> for FRect {
    fn into(self) -> Option<sdl2::rect::Rect> {
        let w = match rect_len_round(self.w) {
            Some(v) => v,
            None => return None,
        };
        let h = match rect_len_round(self.h) {
            Some(v) => v,
            None => return None,
        };
        let x = rect_position_round(self.x);
        let y = rect_position_round(self.y);
        Some(sdl2::rect::Rect::new(
            x,
            y,
            w,
            h,
        ))
    }
}
