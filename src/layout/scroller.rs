use std::cell::Cell;

use sdl2::mouse::MouseButton;

use crate::{
    util::length::frect_to_rect,
    widget::widget::{Widget, WidgetEvent},
};

/// translates its content - facilitates scrolling
///
/// does NOT do any form of culling for widgets which are not visible in the
/// current viewing area - all contained widgets are updated and drawn. it is
/// the responsibility of the contained widgets themselves to cull if they
/// choose to
///
/// it is the responsibility of the contained widget to filter out mouse
/// events which are not within the clipping rectangle (which is set for
/// both draw, as well as update, for convenience)
pub struct Scroller<'sdl, 'state> {
    /// for drag scrolling
    drag_diff: Option<(i32, i32)>,
    pub scroll_x_enabled: bool,
    pub scroll_y_enabled: bool,
    pub scroll_x: &'state Cell<i32>,
    pub scroll_y: &'state Cell<i32>,
    pub contains: Box<dyn Widget + 'sdl>,
}

impl<'sdl, 'state> Scroller<'sdl, 'state> {
    pub fn new(
        scroll_x_enabled: bool,
        scroll_y_enabled: bool,
        scroll_x: &'state Cell<i32>,
        scroll_y: &'state Cell<i32>,
        contains: Box<dyn Widget + 'sdl>,
    ) -> Self {
        Self {
            drag_diff: None,
            scroll_x_enabled,
            scroll_y_enabled,
            scroll_x,
            scroll_y,
            contains,
        }
    }
}

impl<'sdl, 'state> Widget for Scroller<'sdl, 'state> {
    fn min(
        &mut self,
    ) -> Result<(crate::util::length::MinLen, crate::util::length::MinLen), String> {
        self.contains.min()
    }

    fn min_w_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        self.contains.min_w_fail_policy()
    }

    fn min_h_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        self.contains.min_h_fail_policy()
    }

    fn max(
        &mut self,
    ) -> Result<(crate::util::length::MaxLen, crate::util::length::MaxLen), String> {
        self.contains.max()
    }

    fn max_w_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        self.contains.max_w_fail_policy()
    }

    fn max_h_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        self.contains.max_h_fail_policy()
    }

    fn preferred_portion(
        &self,
    ) -> (
        crate::util::length::PreferredPortion,
        crate::util::length::PreferredPortion,
    ) {
        self.contains.preferred_portion()
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        self.contains.preferred_width_from_height(pref_h)
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        self.contains.preferred_height_from_width(pref_w)
    }

    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.contains.preferred_link_allowed_exceed_portion()
    }

    fn update(&mut self, mut event: WidgetEvent) -> Result<(), String> {
        // translate all mouse events before sending to contained widget
        let scroll_x = self.scroll_x.get();
        let scroll_y = self.scroll_y.get();

        event.events.iter_mut().filter(|e| e.available()).for_each(|e| {
            match e.e {
                sdl2::event::Event::MouseMotion {
                    mousestate, x, y, ..
                } => {
                    if let Some(drag_diff) = self.drag_diff {
                        if mousestate.left() {
                            // happen before update on contained widget.
                            // otherwise small rounding differences could cause
                            // the focus indicator to flicker when clicking and
                            // dragging close to a contained widget
                            e.set_consumed();
                            if self.scroll_x_enabled {
                                self.scroll_x.set(x + drag_diff.0);
                            }
                            if self.scroll_y_enabled {
                                self.scroll_y.set(y + drag_diff.1);
                            }
                        }
                    }
                }
                _ => {}
            };
        });

        event
            .canvas
            .set_clip_rect(event.position.map(|frect| frect_to_rect(frect)));
        event.position.as_mut().map(|position| {
            position.x += scroll_x as f32;
            position.y += scroll_y as f32;
        });

        let update_results = self.contains.update(event.dup());
        event.canvas.set_clip_rect(None);

        event.position.as_mut().map(|position| {
            position.x -= scroll_x as f32;
            position.y -= scroll_y as f32;
        });

        update_results?;

        event.events.iter_mut().filter(|e| e.available()).for_each(|e| {
            match e.e {
                // accumulating MouseMotion xrel yrel is not sufficient (is
                // reported as integers, which leads to a large drift from
                // rounding over mouse drag)
                //
                // click and drag scrolling
                sdl2::event::Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    if event.position.map(|pos| frect_to_rect(pos).contains_point((x, y))).unwrap_or(false) {
                        self.drag_diff = Some((scroll_x - x, scroll_y - y));
                    }
                }
                sdl2::event::Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    self.drag_diff = None;
                }
                // mouse wheel scrolling
                sdl2::event::Event::MouseWheel {
                    x,
                    y,
                    mouse_x,
                    mouse_y,
                    ..
                } => {
                    if event.position.map(|pos| frect_to_rect(pos).contains_point((mouse_x, mouse_y))).unwrap_or(false) {
                        e.set_consumed();
                        if self.scroll_x_enabled {
                            self.scroll_x.set(scroll_x - x * 7);
                        }
                        if self.scroll_y_enabled {
                            self.scroll_y.set(scroll_y + y * 7);
                        }
                    }
                }
                _ => {}
            }
        });

        Ok(())
    }

    fn draw(&mut self, mut event: WidgetEvent) -> Result<(), String> {
        // translate all mouse events before sending to contained widget
        let scroll_x = self.scroll_x.get();
        let scroll_y = self.scroll_y.get();

        event.canvas.set_clip_rect(event.position.map(|pos| frect_to_rect(pos)));
        event.position.as_mut().map(|position| {
            position.x += scroll_x as f32;
            position.y += scroll_y as f32;
        });

        let draw_result = self.contains.draw(event.dup());
        event.canvas.set_clip_rect(None);

        event.position.as_mut().map(|position| {
            position.x -= scroll_x as f32;
            position.y -= scroll_y as f32;
        });

        draw_result?;
        Ok(())
    }
}
