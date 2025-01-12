use sdl2::{rect::Rect, render::ClippingRect};

use crate::widget::widget::Widget;

/// contains something. when it is draw, a clipping rect is set to not allow
/// drawing to go past the widget's given position
pub struct Clipper<'sdl> {
    pub contained: &'sdl mut dyn Widget,
}


pub fn clipping_rect_intersection(
    existing_clipping_rect: ClippingRect,
    position: Option<Rect>,
) -> ClippingRect {
    match position {
        Some(position) => {
            match existing_clipping_rect {
                ClippingRect::Some(rect) => match rect.intersection(position) {
                    Some(v) => ClippingRect::Some(v),
                    None => ClippingRect::Zero,
                },
                ClippingRect::Zero => ClippingRect::Zero,
                ClippingRect::None => {
                    // clipping rect has infinite area, so it's just whatever position is
                    ClippingRect::Some(position)
                }
            }
        }
        None => {
            // position is zero area so intersection result is zero
            ClippingRect::Zero
        }
    }
}

impl<'sdl> Widget for Clipper<'sdl> {
    fn update(&mut self, mut event: crate::widget::widget::WidgetEvent) -> Result<(), String> {
        let previous_clipping_rect = event.canvas.clip_rect();
        let clipping_rect =
            clipping_rect_intersection(previous_clipping_rect, event.position.into());
        event.canvas.set_clip_rect(clipping_rect);
        let ret = self.contained.update(event.dup());
        event.canvas.set_clip_rect(previous_clipping_rect); // restore
        ret
    }

    fn draw(&mut self, mut event: crate::widget::widget::WidgetEvent) -> Result<(), String> {
        let previous_clipping_rect = event.canvas.clip_rect();
        let clipping_rect =
            clipping_rect_intersection(previous_clipping_rect, event.position.into());
        event.canvas.set_clip_rect(clipping_rect);
        let ret = self.contained.draw(event.dup());
        event.canvas.set_clip_rect(previous_clipping_rect); // restore
        ret
    }
    
    fn min(&mut self) -> Result<(crate::util::length::MinLen, crate::util::length::MinLen), String> {
        self.contained.min()
    }
    
    fn min_w_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        self.contained.min_w_fail_policy()
    }
    
    fn min_h_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        self.contained.min_h_fail_policy()
    }
    
    fn max(&mut self) -> Result<(crate::util::length::MaxLen, crate::util::length::MaxLen), String> {
        self.contained.max()
    }
    
    fn max_w_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        self.contained.max_w_fail_policy()
    }
    
    fn max_h_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        self.contained.max_h_fail_policy()
    }
    
    fn preferred_portion(&self) -> (crate::util::length::PreferredPortion, crate::util::length::PreferredPortion) {
        self.contained.preferred_portion()
    }
    
    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        self.contained.preferred_width_from_height(pref_h)
    }
    
    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        self.contained.preferred_height_from_width(pref_w)
    }
    
    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.contained.preferred_link_allowed_exceed_portion()
    }
}
