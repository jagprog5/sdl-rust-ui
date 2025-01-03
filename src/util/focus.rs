use sdl2::keyboard::{Keycode, Mod};

use crate::widget::widget::{SDLEvent, WidgetEvent};

use super::length::frect_to_rect;

#[derive(Clone, Copy)]
pub struct FocusID(u64);

/// a widget can be the current focus. how a widget handles that means is up to
/// it. only zero or one widgets can be focused at a time.
pub struct FocusManager {
    /// increments upward to get new unique ids for each widget
    next_available: u64,
    /// the id of the widget that is currently focused
    current_focus: Option<u64>,
}

impl FocusManager {
    /// clear entire state of the focus manager
    pub fn clear(&mut self) {
        self.next_available = 0;
        self.current_focus = None;
    }

    pub fn next_available_id(&mut self) -> FocusID {
        let ret = FocusID(self.next_available);
        self.next_available += 1;
        ret
    }

    pub fn is_focused(&self, id: FocusID) -> bool {
        match self.current_focus {
            Some(v) => id.0 == v,
            None => false,
        }
    }

    pub fn set_focus(&mut self, id: FocusID) {
        self.current_focus = Some(id.0);
    }

    /// only if this widget has focus then remove focus
    pub fn unfocus(&mut self, id: FocusID) {
        if self.is_focused(id) {
            self.current_focus = None;
        }
    }

    pub fn set_next_focused(&mut self) {
        self.current_focus = Some(match self.current_focus {
            None => 0, // start at first
            Some(mut v) => {
                // go to next, wrapping back to 0
                v += 1;
                if v >= self.next_available {
                    v = 0;
                }
                v
            }
        });
    }

    pub fn set_previous_focused(&mut self) {
        let last = self.next_available.checked_sub(1).unwrap_or(0);
        self.current_focus = Some(match self.current_focus {
            None => last,
            Some(v) => v.checked_sub(1).unwrap_or(last),
        });
    }

    /// handle default behavior for how focus should change given the events:
    /// - tab goes to next, shift + tab goes to previous
    /// - mouse moved on/off widget gains/loses focus
    ///
    /// this function may consume tab events
    pub fn default_widget_focus_behavior(my_focus_id: FocusID, event: &mut WidgetEvent) {
        let focus_manager = match &mut event.focus_manager {
            Some(v) => v,
            None => return,
        };
        for sdl_input in event.events.iter_mut() {
            match sdl_input.e {
                sdl2::event::Event::MouseMotion { x, y, .. } => {
                    let mut has_point = false;

                    // if event is consumed, it's considered not over this widget for focus purposes
                    if sdl_input.available() { 
                        if let Some(position) = frect_to_rect(event.position) {
                            let point_contained_in_clipping_rect = match event.canvas.clip_rect() {
                                sdl2::render::ClippingRect::Some(rect) => rect.contains_point((x, y)),
                                sdl2::render::ClippingRect::Zero => false,
                                sdl2::render::ClippingRect::None => true,
                            };
                            // ignore mouse events out of position bounds and
                            // out of scroll area clipping rect
                            if position.contains_point((x, y)) && point_contained_in_clipping_rect {
                                has_point = true;
                            }
                        }
                    }

                    if has_point {
                        focus_manager.set_focus(my_focus_id);
                    } else {
                        focus_manager.unfocus(my_focus_id);
                    }
                }
                sdl2::event::Event::KeyDown {
                    repeat: false,
                    keycode: Some(Keycode::Tab),
                    keymod,
                    ..
                } => {
                    if sdl_input.consumed() {
                        continue;
                    }
                    if !focus_manager.is_focused(my_focus_id) {
                        continue; // only process tab if I am focused
                    }
                    sdl_input.set_consumed();
                    if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                        // shift tab was pressed
                        focus_manager.set_previous_focused();
                    } else {
                        // tab was pressed
                        focus_manager.set_next_focused();
                    }
                }
                _ => {}
            }
        }
    }

    /// how should the focus manager itself handle focus, irrespective of any
    /// focusable widgets
    ///
    /// this will consume tab events
    pub fn default_start_focus_behavior(&mut self, events: &mut [SDLEvent]) {
        if let Some(_) = self.current_focus {
            return; // only applicable if nothing has the focus right now
        }
        for sdl_input in events.iter_mut().filter(|e| e.available()) {
            match sdl_input.e {
                sdl2::event::Event::KeyDown {
                    repeat: false,
                    keycode: Some(Keycode::Tab),
                    keymod,
                    ..
                } => {
                    sdl_input.set_consumed();
                    if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                        // shift tab was pressed
                        self.set_previous_focused();
                    } else {
                        // tab was pressed
                        self.set_next_focused();
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self {
            next_available: Default::default(),
            current_focus: Default::default(),
        }
    }
}
