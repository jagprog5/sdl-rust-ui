use sdl2::{
    mouse::MouseButton,
    pixels::Color,
    rect::{Point, Rect},
};

use crate::util::{
    focus::FocusManager,
    length::{
        AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy,
        PreferredPortion,
    },
    rect::FRect,
};

use super::{Widget, WidgetUpdateEvent};

/// super simple debug widget. draws a outline at its position. use for testing
/// purposes. brief flash when clicked
#[derive(Debug, Clone, Copy)]
#[derive(Default)]
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

    /// internal state. set during update. used during draw
    clicked_this_frame: bool,
    /// state stored for draw from update
    draw_pos: FRect,
}

/// better name for where it isn't being used as a widget, just as a member for
/// sizing info
pub type CustomSizingControl = Debug;


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

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        let ratio = match &self.aspect_ratio {
            None => return None,
            Some(v) => v,
        };

        Some(Ok(AspectRatioPreferredDirection::width_from_height(
            *ratio, pref_h,
        )))
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        let ratio = match &self.aspect_ratio {
            None => return None,
            Some(v) => v,
        };

        Some(Ok(AspectRatioPreferredDirection::height_from_width(
            *ratio, pref_w,
        )))
    }

    fn update(&mut self, event: WidgetUpdateEvent) -> Result<(), String> {
        self.clicked_this_frame = false; // reset each frame
        self.draw_pos = event.position;

        let pos: Option<sdl2::rect::Rect> = event.position.into();
        let pos = match pos {
            Some(v) => v,
            None => return Ok(()), // only functionality is being clicked
        };

        for e in event.events.iter_mut().filter(|e| e.available()) {
            if let sdl2::event::Event::MouseButtonUp {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    window_id,
                    ..
                } = e.e {
                if event.window_id != window_id {
                    continue; // not for me!
                }
                if pos.contains_point((x, y)) {
                    // ignore mouse events out of scroll area
                    let point_contained_in_clipping_rect = match event.clipping_rect {
                        sdl2::render::ClippingRect::Some(rect) => rect.contains_point((x, y)),
                        sdl2::render::ClippingRect::Zero => false,
                        sdl2::render::ClippingRect::None => true,
                    };
                    if !point_contained_in_clipping_rect {
                        continue;
                    }

                    e.set_consumed();
                    self.clicked_this_frame = true;
                }
            }
        }

        Ok(())
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        self.draw_pos.x += pos_delta.0 as f32;
        self.draw_pos.y += pos_delta.1 as f32;
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        _focus_manager: &FocusManager,
    ) -> Result<(), String> {
        // as always, snap to integer grid before rendering / using,
        // plus checks that draw area is non-zero
        let pos: Option<sdl2::rect::Rect> = self.draw_pos.into();
        let pos = match pos {
            Some(v) => v,
            None => return Ok(()),
        };

        let mut color_to_use = Color::RED;
        if self.clicked_this_frame {
            color_to_use = Color::GREEN;
            println!("debug rect at {:?} was clicked!", pos);
        }

        debug_rect_outline(color_to_use, pos, canvas)
    }
}
