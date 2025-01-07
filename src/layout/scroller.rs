use std::cell::Cell;

use sdl2::{mouse::MouseButton, rect::Rect, render::ClippingRect};

use crate::{
    util::length::AspectRatioPreferredDirection,
    widget::{
        debug::CustomSizingControl,
        widget::{place, ConsumedStatus, Widget, WidgetEvent},
    },
};

#[derive(Debug)]
enum DragState {
    None,
    /// waiting for mouse to move far enough before beginning dragging
    DragStart((i32, i32)),
    /// contains drag diff
    Dragging((i32, i32)),
}

pub enum ScrollerSizingPolicy {
    /// inherit sizing from the contained widget
    Children,
    /// states literally, ignoring the contained widget. the contained widget
    /// will then be placed within the scroll widget's bounds
    Custom(CustomSizingControl),
}

/// translates its content - facilitates scrolling
///
/// does NOT do any form of culling for widgets which are not visible in the
/// current viewing area - all contained widgets are updated and drawn. it is
/// the responsibility of the contained widgets themselves to cull if they
/// choose to
///
/// it is the responsibility of the contained widget to filter out mouse events
/// which are not within the sdl clipping rectangle (which is set for both draw,
/// as well as update, for convenience)
///
/// all sizing is inherited from the contained widget
pub struct Scroller<'sdl, 'state> {
    /// for drag scrolling
    drag_state: DragState,
    /// how many pixels to move per unit of received mouse wheel
    pub mouse_wheel_sensitivity: i32,
    /// manhattan distance that the mouse must travel before it's considered a
    /// click and drag scroll
    pub drag_deadzone: u32,
    pub scroll_x_enabled: bool,
    pub scroll_y_enabled: bool,
    pub scroll_x: &'state Cell<i32>,
    pub scroll_y: &'state Cell<i32>,
    pub contained: &'sdl mut dyn Widget,
    pub sizing_policy: ScrollerSizingPolicy,
    /// true restricts the scrolling to keep the contained in frame. TODO!
    pub restrict_scroll: bool,
}

impl<'sdl, 'state> Scroller<'sdl, 'state> {
    pub fn new(
        scroll_x_enabled: bool,
        scroll_y_enabled: bool,
        scroll_x: &'state Cell<i32>,
        scroll_y: &'state Cell<i32>,
        contains: &'sdl mut dyn Widget,
    ) -> Self {
        Self {
            drag_state: DragState::None,
            mouse_wheel_sensitivity: 7,
            drag_deadzone: 10,
            scroll_x_enabled,
            scroll_y_enabled,
            scroll_x,
            scroll_y,
            contained: contains,
            restrict_scroll: true,
            sizing_policy: ScrollerSizingPolicy::Children,
        }
    }
}

fn clipping_rect_intersection(
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

impl<'sdl, 'state> Widget for Scroller<'sdl, 'state> {
    fn min(
        &mut self,
    ) -> Result<(crate::util::length::MinLen, crate::util::length::MinLen), String> {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.min(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => {
                Ok((scroller_literal_sizing.min_w, scroller_literal_sizing.min_h))
            }
        }
    }

    fn min_w_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.min_w_fail_policy(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => {
                scroller_literal_sizing.min_w_fail_policy
            }
        }
    }

    fn min_h_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.min_h_fail_policy(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => {
                scroller_literal_sizing.min_h_fail_policy
            }
        }
    }

    fn max(
        &mut self,
    ) -> Result<(crate::util::length::MaxLen, crate::util::length::MaxLen), String> {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.max(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => {
                Ok((scroller_literal_sizing.max_w, scroller_literal_sizing.max_h))
            }
        }
    }

    fn max_w_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.max_w_fail_policy(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => {
                scroller_literal_sizing.max_w_fail_policy
            }
        }
    }

    fn max_h_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.max_h_fail_policy(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => {
                scroller_literal_sizing.max_h_fail_policy
            }
        }
    }

    fn preferred_portion(
        &self,
    ) -> (
        crate::util::length::PreferredPortion,
        crate::util::length::PreferredPortion,
    ) {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.preferred_portion(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => (
                scroller_literal_sizing.preferred_w,
                scroller_literal_sizing.preferred_h,
            ),
        }
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        match &mut self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.preferred_width_from_height(pref_h),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => {
                let ratio = match &scroller_literal_sizing.aspect_ratio {
                    None => return None,
                    Some(v) => v,
                };

                Some(Ok(AspectRatioPreferredDirection::width_from_height(
                    *ratio, pref_h,
                )))
            }
        }
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        match &mut self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.preferred_height_from_width(pref_w),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => {
                let ratio = match &scroller_literal_sizing.aspect_ratio {
                    None => return None,
                    Some(v) => v,
                };

                Some(Ok(AspectRatioPreferredDirection::height_from_width(
                    *ratio, pref_w,
                )))
            }
        }
    }

    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => {
                self.contained.preferred_link_allowed_exceed_portion()
            }
            ScrollerSizingPolicy::Custom(scroller_literal_sizing) => {
                scroller_literal_sizing.preferred_link_allowed_exceed_portion
            }
        }
    }

    fn update(&mut self, mut event: WidgetEvent) -> Result<(), String> {
        if let DragState::Dragging(_) = self.drag_state {
            // consume related events if currently dragging. do this before
            // passing event to contained
            event
                .events
                .iter_mut()
                .filter(|e| e.available())
                .for_each(|e| match e.e {
                    sdl2::event::Event::MouseButtonDown { .. }
                    | sdl2::event::Event::MouseMotion { .. }
                    | sdl2::event::Event::MouseButtonUp { .. } => {
                        e.set_consumed();
                    }
                    _ => {}
                });
        }

        // translate events before sending to contained. then translate back again when done
        let scroll_x = self.scroll_x.get();
        let scroll_y = self.scroll_y.get();

        let previous_clipping_rect = event.canvas.clip_rect();
        let clipping_rect =
            clipping_rect_intersection(previous_clipping_rect, event.position.into());
        event.canvas.set_clip_rect(clipping_rect);
        event.position.x += scroll_x as f32;
        event.position.y += scroll_y as f32;

        let update_result = match &self.sizing_policy {
            ScrollerSizingPolicy::Children => {
                // scroller exactly passes sizing information to parent in this
                // case, no need to place again
                self.contained.update(event.dup())
            }
            ScrollerSizingPolicy::Custom(_) => {
                // whatever the sizing of the parent, properly place the
                // contained within it
                let position_for_contained =
                    place(self.contained, event.position, event.aspect_ratio_priority)?;
                self.contained
                    .update(event.sub_event(position_for_contained))
            }
        };
        event.canvas.set_clip_rect(previous_clipping_rect); // restore
        event.position.x -= scroll_x as f32;
        event.position.y -= scroll_y as f32;

        // min_x, min_y, max_x, max_y
        // let mut scroll_restrictions: Option<(i32, i32, i32, i32)> = None;

        // if self.restrict_scroll {
        //     if let Some(position) = event.position {
        //         let pos = place(self.contained, position, event.aspect_ratio_priority)?;
        //         // println!("{:?}", self.contains);
        //         println!("{:?}", self.contained.min());
        //         println!("{:?}", self.contained.max());
        //         println!("============= {:?}", pos);
        //         if let Some(pos) = pos {
        //             let min_x = pos.x.round() as i32;
        //             let min_y = pos.y.round() as i32;
        //             let max_x = (pos.x + pos.width()).round() as i32;
        //             let max_y = (pos.y + pos.height()).round() as i32;
        //             scroll_restrictions = Some((min_x, min_y, max_x, max_y));
        //         }
        //     }
        // }

        // handle mouse wheel. happens after update, as it allows contained
        // to consume it first (for example, with nested scrolls)
        event
            .events
            .iter_mut()
            .filter(|e| match e.consumed_status() {
                // only look at not consumed by layout
                ConsumedStatus::ConsumedByLayout => false,
                _ => true,
            })
            .for_each(|e| match e.e {
                // mouse wheel logic
                sdl2::event::Event::MouseWheel {
                    x,
                    y,
                    mouse_x,
                    mouse_y,
                    direction,
                    ..
                } => {
                    let multiplier: i32 = match direction {
                        sdl2::mouse::MouseWheelDirection::Flipped => -1,
                        _ => 1,
                    };

                    // only look at wheel when mouse over scroll area
                    let pos: Option<sdl2::rect::Rect> = event.position.into();
                    if pos
                        .map(|pos| pos.contains_point((mouse_x, mouse_y)))
                        .unwrap_or(false)
                    {
                        let point_contained_in_clipping_rect = match clipping_rect {
                            sdl2::render::ClippingRect::Some(rect) => {
                                rect.contains_point((mouse_x, mouse_y))
                            }
                            sdl2::render::ClippingRect::Zero => false,
                            sdl2::render::ClippingRect::None => true,
                        };
                        if !point_contained_in_clipping_rect {
                            return;
                        }
                        e.set_consumed_by_layout();
                        if self.scroll_x_enabled {
                            self.scroll_x.set(
                                self.scroll_x.get() - multiplier * x * self.mouse_wheel_sensitivity,
                            );
                        }
                        if self.scroll_y_enabled {
                            let desired_next_scroll_y =
                                self.scroll_y.get() - multiplier * y * self.mouse_wheel_sensitivity;
                            // println!("{}", desired_next_scroll_y);
                            // if let Some(scroll_restrictions) = scroll_restrictions {
                            //     println!("{:?}", scroll_restrictions);
                            //     //     if desired_next_scroll_y < scroll_restrictions.1 {
                            //     //         desired_next_scroll_y = scroll_restrictions.1;
                            //     //     }
                            // }
                            self.scroll_y.set(desired_next_scroll_y);
                        }
                    }
                }
                sdl2::event::Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => match self.drag_state {
                    DragState::None => {}
                    _ => {
                        self.drag_state = DragState::None;
                        e.set_consumed_by_layout();
                    }
                },
                // on mouse down, log the position and wait for drag start
                sdl2::event::Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    let pos: Option<sdl2::rect::Rect> = event.position.into();
                    if pos
                        .map(|pos| pos.contains_point((x, y)))
                        .unwrap_or(false)
                    {
                        let point_contained_in_clipping_rect = match clipping_rect {
                            sdl2::render::ClippingRect::Some(rect) => rect.contains_point((x, y)),
                            sdl2::render::ClippingRect::Zero => false,
                            sdl2::render::ClippingRect::None => true,
                        };
                        if !point_contained_in_clipping_rect {
                            return;
                        }
                        e.set_consumed_by_layout();
                        if let DragState::None = self.drag_state {
                            self.drag_state = DragState::DragStart((x, y));
                        }
                    }
                }
                // on mouse motion apply mouse drag.
                sdl2::event::Event::MouseMotion {
                    x, y, mousestate, ..
                } => {
                    if !mousestate.left() {
                        self.drag_state = DragState::None;
                        // intentional fallthrough
                    }
                    if let DragState::None = self.drag_state {
                        return;
                    }
                    if let DragState::DragStart((start_x, start_y)) = self.drag_state {
                        let dragged_far_enough_x = (start_x - x).abs() as u32 > self.drag_deadzone;
                        let dragged_far_enough_y = (start_y - y).abs() as u32 > self.drag_deadzone;
                        let trigger_x = dragged_far_enough_x && self.scroll_x_enabled;
                        let trigger_y = dragged_far_enough_y && self.scroll_y_enabled;
                        if trigger_x || trigger_y {
                            self.drag_state = DragState::Dragging((
                                x - self.scroll_x.get(),
                                y - self.scroll_y.get(),
                            ));
                            // intentional fallthrough
                        }
                    }

                    if let DragState::Dragging((drag_x, drag_y)) = self.drag_state {
                        e.set_consumed_by_layout();
                        self.scroll_x.set(x - drag_x);
                        self.scroll_y.set(y - drag_y);
                    }
                }
                _ => {}
            });

        update_result
    }

    fn draw(&mut self, mut event: WidgetEvent) -> Result<(), String> {
        // translate events before sending to contained. then translate back again when done
        let scroll_x = self.scroll_x.get();
        let scroll_y = self.scroll_y.get();

        let previous_clipping_rect = event.canvas.clip_rect();
        let clipping_rect =
            clipping_rect_intersection(previous_clipping_rect, event.position.into());
        event.canvas.set_clip_rect(clipping_rect);
        event.position.x += scroll_x as f32;
        event.position.y += scroll_y as f32;

        let draw_result = match &self.sizing_policy {
            ScrollerSizingPolicy::Children => {
                // scroller exactly passes sizing information to parent in this
                // case, no need to place again
                self.contained.draw(event.dup())
            }
            ScrollerSizingPolicy::Custom(_) => {
                // whatever the sizing of the parent, properly place the
                // contained within it
                let position_for_contained =
                    place(self.contained, event.position, event.aspect_ratio_priority)?;
                self.contained.draw(event.sub_event(position_for_contained))
            }
        };
        event.canvas.set_clip_rect(previous_clipping_rect); // restore
        event.position.x -= scroll_x as f32;
        event.position.y -= scroll_y as f32;

        draw_result
    }
}
