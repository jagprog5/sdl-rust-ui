use crate::util::{
    focus::FocusManager,
    length::{MaxLen, MinLen, PreferredPortion},
};

use super::Widget;

pub struct Strut {
    pub min_w: MinLen,
    pub min_h: MinLen,
    pub max_w: MaxLen,
    pub max_h: MaxLen,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,
}

impl Strut {
    pub fn fixed(w: f32, h: f32) -> Self {
        Strut {
            min_w: MinLen(w),
            min_h: MinLen(h),
            max_w: MaxLen(w),
            max_h: MaxLen(h),
            preferred_w: PreferredPortion(0.),
            preferred_h: PreferredPortion(0.),
        }
    }

    // prefers to be at its largest, but will shrink as needed
    pub fn shrinkable(max_w: MaxLen, max_h: MaxLen) -> Self {
        Strut {
            min_w: MinLen::LAX,
            min_h: MinLen::LAX,
            max_w,
            max_h,
            preferred_w: PreferredPortion::FULL,
            preferred_h: PreferredPortion::FULL,
        }
    }
}

impl Widget for Strut {
    fn draw(
        &mut self,
        _canvas: &mut sdl2::render::WindowCanvas,
        _focus_manager: &FocusManager,
    ) -> Result<(), String> {
        Ok(())
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        Ok((self.max_w, self.max_h))
    }

    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        Ok((self.min_w, self.min_h))
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.preferred_w, self.preferred_h)
    }
}
