use sdl2::{
    mouse::MouseButton,
    pixels::Color,
    rect::{Point, Rect},
};

use crate::util::length::{
    frect_to_rect, AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion
};

use super::widget::{Widget, WidgetEvent};

/// super simple debug widget. draws a outline at its position. use for testing
/// purposes. brief flash when clicked
pub struct Debug {
    pub min_w: MinLen,
    pub min_h: MinLen,
    pub max_w: MaxLen,
    pub max_h: MaxLen,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,
    pub aspect_ratio: Option<f32>,
    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,
    pub preferred_link_allowed_exceed_portion: bool,
}

impl Default for Debug {
    fn default() -> Self {
        Self {
            min_w: Default::default(),
            min_h: Default::default(),
            max_w: Default::default(),
            max_h: Default::default(),
            preferred_w: Default::default(),
            preferred_h: Default::default(),
            aspect_ratio: Default::default(),
            min_w_fail_policy: Default::default(),
            max_w_fail_policy: Default::default(),
            min_h_fail_policy: Default::default(),
            max_h_fail_policy: Default::default(),
            preferred_link_allowed_exceed_portion: Default::default(),
        }
    }
}

/// use as a placeholder if some texture is missing, etc.
pub fn debug_rect_outline(
    color: sdl2::pixels::Color,
    position: Rect,
    canvas: &mut sdl2::render::WindowCanvas,
) -> Result<(), String> {
    // debug is super simple. simply re-render every frame
    canvas.set_draw_color(Color::RGB(50, 50, 50));
    canvas.fill_rect(position)?;

    canvas.set_draw_color(color);

    let points: [Point; 6] = [
        Point::new(position.x, position.y),
        Point::new(position.x + position.w - 1, position.y),
        Point::new(position.x + position.w - 1, position.y + position.h - 1),
        Point::new(position.x, position.y + position.h - 1),
        Point::new(position.x, position.y),
        Point::new(position.x + position.w - 1, position.y + position.h - 1),
    ];
    canvas.draw_lines(points.as_ref())
}

impl Widget for Debug {
    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.preferred_link_allowed_exceed_portion
    }
    
    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        Ok((self.min_w, self.min_h))
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.min_w_fail_policy
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.min_h_fail_policy
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        Ok((self.max_w, self.max_h))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_w_fail_policy
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_h_fail_policy
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.preferred_w, self.preferred_h)
    }

    fn preferred_width_from_height(
        &mut self,
        pref_h: f32,
    ) -> Option<Result<f32, String>> {
        let ratio = match &self.aspect_ratio {
            None => return None,
            Some(v) => v,
        };

        Some(Ok(AspectRatioPreferredDirection::width_from_height(
            *ratio,
            pref_h,
        )))
    }

    fn preferred_height_from_width(
        &mut self,
        pref_w: f32,
    ) -> Option<Result<f32, String>> {
        let ratio = match &self.aspect_ratio {
            None => return None,
            Some(v) => v,
        };

        Some(Ok(AspectRatioPreferredDirection::height_from_width(
            *ratio,
            pref_w,
        )))
    }

    fn draw(&mut self, event: WidgetEvent) -> Result<(), String> {
        // as always, snap to integer grid before rendering / using,
        // plus checks that draw area is non-zero
        let pos = match frect_to_rect(event.position){
            Some(v) => v,
            None => return Ok(()),
        };

        let mut color_to_use = Color::RED;

        for e in event.events.iter_mut().filter(|e| e.available()) {
            match e.e {
                sdl2::event::Event::MouseButtonUp {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    if pos.contains_point((x, y)) {
                        // ignore mouse events out of scroll area
                        let point_contained_in_clipping_rect = match event.canvas.clip_rect() {
                            sdl2::render::ClippingRect::Some(rect) => rect.contains_point((x, y)),
                            sdl2::render::ClippingRect::Zero => false,
                            sdl2::render::ClippingRect::None => true,
                        };
                        if !point_contained_in_clipping_rect {
                            continue;
                        }

                        e.set_consumed();
                        color_to_use = Color::GREEN;
                        println!("debug rect at {:?} was clicked!", pos);
                    }
                }
                _ => {}
            }
        }

        debug_rect_outline(color_to_use, pos, event.canvas)
    }
}
