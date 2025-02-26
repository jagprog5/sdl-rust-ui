use sdl2::{
    keyboard::{Keycode, Mod},
    render::ClippingRect,
};

use crate::widget::SDLEvent;


#[derive(Debug, PartialEq, Eq)]
pub struct FocusID {
    pub previous: String,
    pub me: String,
    pub next: String,
}

/// a widget can be the current focus. how a widget handles what that means is
/// up to it. only zero or one widgets should be focused at a time.
#[derive(Default)]
pub struct FocusManager(pub Option<String>);

pub(crate) fn point_in_position_and_clipping_rect(
    x: i32,
    y: i32,
    position: sdl2::rect::Rect,
    clipping_rect: ClippingRect,
) -> bool {
    if position.contains_point((x, y)) {
        // ignore mouse events out of scroll area and position
        let point_contained_in_clipping_rect = match clipping_rect {
            sdl2::render::ClippingRect::Some(rect) => rect.contains_point((x, y)),
            sdl2::render::ClippingRect::Zero => false,
            sdl2::render::ClippingRect::None => true,
        };

        if point_contained_in_clipping_rect {
            return true;
        }
    }
    false
}

// closely related to WidgetUpdateEvent
pub struct DefaultFocusBehaviorArg<'sdl> {
    pub focus_manager: &'sdl mut FocusManager,
    pub position: super::rect::FRect,
    pub clipping_rect: ClippingRect,
    pub window_id: u32,
    /// a single event. the intent is that this would be inline with the
    /// existing processing loop - for consistent order of operations each
    /// element should be fully processed before moving to the next element
    pub event: &'sdl mut SDLEvent,
}

impl FocusManager {
    pub fn is_focused(&self, other: &FocusID) -> bool {
        self.0.as_ref().map(|uid| uid == other.me.as_str()).unwrap_or(false)
    }

    /// handle default behavior for how focus should change given the events:
    /// - mouse moved over widget gains focus
    /// - if focused:
    ///     - tab goes to next, shift + tab goes to previous (consumes events)
    ///     - escape key causes unfocus (consumes event)
    pub fn default_widget_focus_behavior(my_focus_id: &FocusID, event: DefaultFocusBehaviorArg) {
        match event.event.e {
            // keys:
            // - only applicable if currently focused
            // - consume key event once used
            sdl2::event::Event::KeyDown {
                repeat,
                keycode: Some(Keycode::Tab),
                keymod,
                ..
            } => {
                if !event.focus_manager.is_focused(&my_focus_id) {
                    return; // only process tab if I am focused
                }
                event.event.set_consumed();
                if repeat {
                    return;
                }
                if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                    // shift tab was pressed
                    event.focus_manager.0 = Some(my_focus_id.previous.clone());
                } else {
                    // tab was pressed
                    event.focus_manager.0 = Some(my_focus_id.next.clone());
                }
            }
            sdl2::event::Event::KeyDown {
                repeat,
                keycode: Some(Keycode::ESCAPE),
                ..
            } => {
                if !event.focus_manager.is_focused(&my_focus_id) {
                    return; // only process escape if I am focused
                }
                event.event.set_consumed();
                if repeat {
                    return;
                }
                event.focus_manager.0 = None; // unfocus
            }
            sdl2::event::Event::MouseMotion {
                x, y, window_id, ..
            } => {
                if event.window_id != window_id {
                    return; // not for me!
                }
                let position: Option<sdl2::rect::Rect> = event.position.into();
                if let Some(position) = position {
                    if point_in_position_and_clipping_rect(x, y, position, event.clipping_rect) {
                        // even if not focused, if mouse is moved over
                        // widget then set focus to that widget
                        //
                        // generally never consume mouse motion events
                        event.focus_manager.0 = Some(my_focus_id.me.clone());
                    }
                }
            }
            _ => {}
        }
    }

    /// if tab or shift tab has not been consumed by any widget, then set the
    /// focus to the first or last widget, respectively
    pub fn default_start_focus_behavior(
        &mut self,
        events: &mut [SDLEvent],
        start_widget_focus_id: &str,
        end_widget_focus_id: &str,
    ) {
        for sdl_input in events.iter_mut().filter(|e| e.available()) {
            if let sdl2::event::Event::KeyDown {
                repeat,
                keycode: Some(Keycode::Tab),
                keymod,
                ..
            } = sdl_input.e
            {
                sdl_input.set_consumed();
                if repeat {
                    continue;
                }
                if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                    // shift tab was pressed
                    self.0 = Some(end_widget_focus_id.to_owned());
                } else {
                    // tab was pressed
                    self.0 = Some(start_widget_focus_id.to_owned());
                }
            }
        }
    }
}
