use std::cell::Cell;

use sdl2::{
    event::WindowEvent,
    mouse::{MouseButton, SystemCursor},
    render::ClippingRect,
};

use crate::{
    util::{focus::FocusManager, length::AspectRatioPreferredDirection, rect::FRect},
    widget::{
        debug::CustomSizingControl,
        {place, ConsumedStatus, Widget, WidgetUpdateEvent},
    },
};

use super::clipper::clipping_rect_intersection;

#[derive(Debug)]
enum DragState {
    None,
    /// waiting for mouse to move far enough before beginning dragging
    DragStart((i32, i32)),
    /// contains drag diff
    Dragging((i32, i32)),
}

#[derive(Default)]
pub enum ScrollAspectRatioDirectionPolicy {
    #[default]
    Inherit,
    Literal(AspectRatioPreferredDirection),
}

pub enum ScrollerSizingPolicy {
    /// inherit sizing from the contained widget
    Children,
    /// states literally, ignoring the contained widget. the contained widget
    /// will then be placed within the scroll widget's bounds
    ///
    /// give an aspect ratio direction to be given to the contained widget
    Custom(CustomSizingControl, ScrollAspectRatioDirectionPolicy),
}

#[derive(Default)]
struct ScrollerCursorCache {
    /// this type is:
    ///  - outer optional, is the cache set or not
    ///  - inner optional, the cache is set, but None if sdl call failed (this
    ///    api in infallible - should not err on sdl2 cursor set failure)
    ///
    /// when freed it clears the cursor if it is currently set
    cursor: Option<Option<sdl2::mouse::Cursor>>,
    /// cursor loaded is appropriate for this
    scroll_x_enabled: bool,
    /// cursor loaded is appropriate for this
    scroll_y_enabled: bool,
}

impl ScrollerCursorCache {
    pub fn clear(&mut self) {
        self.cursor = None;
    }

    pub fn set_or_use_cache(&mut self, scroll_x_enabled: bool, scroll_y_enabled: bool) {
        if !scroll_x_enabled && !scroll_y_enabled {
            self.cursor = None;
            return;
        }

        if self.cursor.is_none()
            || self.scroll_x_enabled != scroll_x_enabled
            || self.scroll_y_enabled != scroll_y_enabled
        {
            self.scroll_x_enabled = scroll_x_enabled;
            self.scroll_y_enabled = scroll_y_enabled;

            let cursor_to_request = if scroll_x_enabled && scroll_y_enabled {
                SystemCursor::SizeAll
            } else if scroll_x_enabled {
                SystemCursor::SizeWE
            } else {
                SystemCursor::SizeNS
            };

            let cursor_result = sdl2::mouse::Cursor::from_system(cursor_to_request);
            debug_assert!(cursor_result.is_ok());
            let cursor_optional = cursor_result.ok();
            if let Some(cursor) = cursor_optional.as_ref() {
                cursor.set()
            }
            self.cursor = Some(cursor_optional);
        }
    }
}

/// translates its content - facilitates scrolling. also applies clipping rect
/// to contained content
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
    /// true restricts the scrolling to keep the contained in frame
    pub restrict_scroll: bool,

    /// calculated during update, stored for draw.
    /// used for clipping rect calculations
    previous_clipping_rect_from_update: ClippingRect,
    position_from_update: FRect,

    cursor_cache: ScrollerCursorCache,
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
            cursor_cache: Default::default(),
            previous_clipping_rect_from_update: ClippingRect::None,
            position_from_update: Default::default(),
        }
    }
}

/// apply even if scroll is not enabled (as what if it was enabled previously
/// and content was moved off screen)
fn apply_scroll_restrictions(
    mut position_for_contained: crate::util::rect::FRect,
    event_position: crate::util::rect::FRect,
    scroll_y: &mut i32,
    scroll_x: &mut i32,
) {
    position_for_contained.x += *scroll_x as f32;
    position_for_contained.y += *scroll_y as f32;

    if position_for_contained.h < event_position.h {
        // the contained thing is smaller than the parent
        let violating_top = position_for_contained.y < event_position.y;
        let violating_bottom = position_for_contained.y + position_for_contained.h
            > event_position.y + event_position.h;

        if violating_top {
            *scroll_y += (event_position.y - position_for_contained.y) as i32;
        } else if violating_bottom {
            *scroll_y -= ((position_for_contained.y + position_for_contained.h)
                - (event_position.y + event_position.h)) as i32;
        }
    } else {
        let down_from_top = position_for_contained.y > event_position.y;

        let up_from_bottom = position_for_contained.y + position_for_contained.h
            < event_position.y + event_position.h;

        if down_from_top {
            *scroll_y += (event_position.y - position_for_contained.y) as i32;
        } else if up_from_bottom {
            *scroll_y -= ((position_for_contained.y + position_for_contained.h)
                - (event_position.y + event_position.h)) as i32;
        }
    }

    if position_for_contained.w < event_position.w {
        // the contained thing is smaller than the parent
        let violating_left = position_for_contained.x < event_position.x;
        let violating_right = position_for_contained.x + position_for_contained.w
            > event_position.x + event_position.w;

        if violating_left {
            *scroll_x += (event_position.x - position_for_contained.x) as i32;
        } else if violating_right {
            *scroll_x -= ((position_for_contained.x + position_for_contained.w)
                - (event_position.x + event_position.w)) as i32;
        }
    } else {
        let left_from_right = position_for_contained.x > event_position.x;

        let right_from_left = position_for_contained.x + position_for_contained.w
            < event_position.x + event_position.w;

        if left_from_right {
            *scroll_x += (event_position.x - position_for_contained.x) as i32;
        } else if right_from_left {
            *scroll_x -= ((position_for_contained.x + position_for_contained.w)
                - (event_position.x + event_position.w)) as i32;
        }
    }
}

impl<'sdl, 'state> Widget for Scroller<'sdl, 'state> {
    fn min(
        &mut self,
    ) -> Result<(crate::util::length::MinLen, crate::util::length::MinLen), String> {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.min(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => {
                Ok((scroller_literal_sizing.min_w, scroller_literal_sizing.min_h))
            }
        }
    }

    fn min_w_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.min_w_fail_policy(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => {
                scroller_literal_sizing.min_w_fail_policy
            }
        }
    }

    fn min_h_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.min_h_fail_policy(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => {
                scroller_literal_sizing.min_h_fail_policy
            }
        }
    }

    fn max(
        &mut self,
    ) -> Result<(crate::util::length::MaxLen, crate::util::length::MaxLen), String> {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.max(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => {
                Ok((scroller_literal_sizing.max_w, scroller_literal_sizing.max_h))
            }
        }
    }

    fn max_w_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.max_w_fail_policy(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => {
                scroller_literal_sizing.max_w_fail_policy
            }
        }
    }

    fn max_h_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        match &self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.max_h_fail_policy(),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => {
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
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => (
                scroller_literal_sizing.preferred_w,
                scroller_literal_sizing.preferred_h,
            ),
        }
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        match &mut self.sizing_policy {
            ScrollerSizingPolicy::Children => self.contained.preferred_width_from_height(pref_h),
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => {
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
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => {
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
            ScrollerSizingPolicy::Custom(scroller_literal_sizing, _) => {
                scroller_literal_sizing.preferred_link_allowed_exceed_portion
            }
        }
    }

    fn update(&mut self, mut event: WidgetUpdateEvent) -> Result<(), String> {
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
        let mut scroll_x = self.scroll_x.get();
        let mut scroll_y = self.scroll_y.get();

        self.previous_clipping_rect_from_update = event.clipping_rect;
        self.position_from_update = event.position;

        let clip_rect_for_contained = clipping_rect_intersection(
            self.previous_clipping_rect_from_update,
            self.position_from_update.into(),
        );

        let position_for_contained = match &self.sizing_policy {
            ScrollerSizingPolicy::Children => {
                // scroller exactly passes sizing information to parent in this
                // case, no need to place again
                event.position
            }
            ScrollerSizingPolicy::Custom(_, dir) => {
                let dir = match dir {
                    ScrollAspectRatioDirectionPolicy::Inherit => event.aspect_ratio_priority,
                    ScrollAspectRatioDirectionPolicy::Literal(dir) => *dir,
                };
                place(self.contained, event.position, dir)?
            }
        };

        if self.restrict_scroll {
            // restrict here to catch all from previous frame or previous within
            // this frame. e.g. if the window is resized to be smaller so it's
            // no longer within bounds
            apply_scroll_restrictions(
                position_for_contained,
                event.position,
                &mut scroll_y,
                &mut scroll_x,
            );
        }

        // shift all positions based on the scroll, and update the container
        let position_for_contained_shifted = FRect {
            x: position_for_contained.x + scroll_x as f32,
            y: position_for_contained.y + scroll_y as f32,
            w: position_for_contained.w,
            h: position_for_contained.h,
        };
        let mut event_for_contained = event.sub_event(position_for_contained_shifted);
        // set clipping rect in dup as to not affect any widgets that might come
        // after this one
        event_for_contained.clipping_rect = clip_rect_for_contained;

        let before_update_scroll_pos = (scroll_x, scroll_y);

        self.contained.update(event_for_contained)?;

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
                    window_id,
                    ..
                } => {
                    if event.window_id != window_id {
                        return; // not for me!
                    }
                    let mut multiplier: i32 = match direction {
                        sdl2::mouse::MouseWheelDirection::Flipped => -1,
                        _ => 1,
                    };
                    if position_for_contained.h > event.position.h {
                        multiplier *= -1;
                    }
                    // only look at wheel when mouse over scroll area
                    let pos: Option<sdl2::rect::Rect> = event.position.into();
                    if pos
                        .map(|pos| pos.contains_point((mouse_x, mouse_y)))
                        .unwrap_or(false)
                    {
                        let point_contained_in_clipping_rect = match clip_rect_for_contained {
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
                            scroll_x -= multiplier * x * self.mouse_wheel_sensitivity;
                        }
                        if self.scroll_y_enabled {
                            scroll_y -= multiplier * y * self.mouse_wheel_sensitivity;
                        }
                        if self.restrict_scroll {
                            apply_scroll_restrictions(
                                position_for_contained,
                                event.position,
                                &mut scroll_y,
                                &mut scroll_x,
                            );
                        }
                    }
                }
                sdl2::event::Event::Window {
                    win_event:
                        WindowEvent::Hidden
                        | WindowEvent::Minimized
                        | WindowEvent::Leave
                        | WindowEvent::FocusLost
                        | WindowEvent::Close,
                    ..
                } => {
                    // same functionality as below for mouse button up,
                    // but don't consume the event
                    self.drag_state = DragState::None;
                    if self.restrict_scroll {
                        apply_scroll_restrictions(
                            position_for_contained,
                            event.position,
                            &mut scroll_y,
                            &mut scroll_x,
                        );
                    }
                }
                sdl2::event::Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => match self.drag_state {
                    DragState::None => {}
                    _ => {
                        // reset, regardless mouse position
                        self.drag_state = DragState::None;
                        e.set_consumed_by_layout();
                        if self.restrict_scroll {
                            apply_scroll_restrictions(
                                position_for_contained,
                                event.position,
                                &mut scroll_y,
                                &mut scroll_x,
                            );
                        }
                    }
                },
                // on mouse down, log the position and wait for drag start
                sdl2::event::Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    window_id,
                    ..
                } => {
                    if event.window_id != window_id {
                        return; // not for me!
                    }
                    let pos: Option<sdl2::rect::Rect> = event.position.into();
                    if pos.map(|pos| pos.contains_point((x, y))).unwrap_or(false) {
                        let point_contained_in_clipping_rect = match clip_rect_for_contained {
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
                    x,
                    y,
                    mousestate,
                    window_id,
                    ..
                } => {
                    if !mousestate.left() {
                        self.drag_state = DragState::None;
                        // if mouse motion is detected and the left mouse button
                        // isn't pressed down, regardless of position or window,
                        // then clear the drag state
                        //
                        // intentional fallthrough.
                    }
                    if let DragState::None = self.drag_state {
                        return;
                    }
                    if event.window_id != window_id {
                        // ignore drag through windows. this would only make
                        // sense if there was some relative coordinate system,
                        // which I don't plan on doing
                        return;
                    }
                    e.set_consumed_by_layout();
                    if let DragState::DragStart((start_x, start_y)) = self.drag_state {
                        let dragged_far_enough_x =
                            (start_x - x).unsigned_abs() > self.drag_deadzone;
                        let dragged_far_enough_y =
                            (start_y - y).unsigned_abs() > self.drag_deadzone;
                        let trigger_x = dragged_far_enough_x && self.scroll_x_enabled;
                        let trigger_y = dragged_far_enough_y && self.scroll_y_enabled;
                        if trigger_x || trigger_y {
                            self.drag_state = DragState::Dragging((x - scroll_x, y - scroll_y));
                            // intentional fallthrough
                        }
                    }

                    if let DragState::Dragging((drag_x, drag_y)) = self.drag_state {
                        if self.scroll_x_enabled {
                            scroll_x = x - drag_x;
                        }
                        if self.scroll_y_enabled {
                            scroll_y = y - drag_y;
                        }
                    }
                }
                _ => {}
            });

        // sync changes. the scroll_x and scroll_y local vars should not have
        // been changed if the scroll wasn't enabled, with the exception of
        // scroll restrictions (and e.g. changing window size)
        self.scroll_x.set(scroll_x);
        self.scroll_y.set(scroll_y);

        // update cursor based on drag state
        match self.drag_state {
            DragState::Dragging(_) => {
                self.cursor_cache
                    .set_or_use_cache(self.scroll_x_enabled, self.scroll_y_enabled);
            }
            _ => {
                self.cursor_cache.clear();
            }
        }

        // account for changes between when update was called and the events were consumed
        self.contained.update_adjust_position((
            scroll_x - before_update_scroll_pos.0,
            scroll_y - before_update_scroll_pos.1,
        ));
        Ok(())
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        self.position_from_update.x += pos_delta.0 as f32;
        self.position_from_update.y += pos_delta.1 as f32;
        self.contained.update_adjust_position(pos_delta);
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        focus_manager: Option<&FocusManager>,
    ) -> Result<(), String> {
        debug_assert!(canvas.clip_rect() == self.previous_clipping_rect_from_update);
        canvas.set_clip_rect(clipping_rect_intersection(
            self.previous_clipping_rect_from_update,
            self.position_from_update.into(),
        ));
        let draw_result = self.contained.draw(canvas, focus_manager);
        canvas.set_clip_rect(self.previous_clipping_rect_from_update); // restore
        draw_result
    }
}
