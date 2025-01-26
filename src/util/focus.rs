use sdl2::{
    keyboard::{Keycode, Mod},
    render::ClippingRect,
};

use crate::widget::SDLEvent;

use std::{
    cell::Cell,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

/// 8 pseudo-random bytes. provide a source of randomness
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PRNGBytes(pub [u8; 8]);

/// like a uuid, but not unique between machines; no form of mac or node id
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UID {
    /// monotonic in time
    inst: Instant,
    /// monotonic counter
    count: u64,
    prng_bytes: PRNGBytes,
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

impl UID {
    /// create a uid, with some supplied random bytes
    ///
    /// if lazy, provide zeros
    pub fn new(prng_bytes: PRNGBytes) -> Self {
        let now = Instant::now();
        let count = COUNTER.fetch_add(1, Ordering::Relaxed); // overflows
        Self {
            inst: now,
            count,
            prng_bytes,
        }
    }
}

/// the focus IDs form a circular doubly linked list via UIDs. if elements are
/// added or removed from the ui then the next and previous UIDs should be kept
/// in sync (if they are not, then it's treated like being unfocused when
/// transitioning to a non existent UID)
#[derive(Debug, Clone, Copy)]
pub struct CircularUID {
    uid: UID,
    /// the uid that comes after this one
    next_uid: Option<UID>,
    /// the uid that comes before this one
    previous_uid: Option<UID>,
}

impl CircularUID {
    pub fn new(uid: UID) -> Self {
        Self {
            uid,
            next_uid: None,
            previous_uid: None,
        }
    }

    pub fn uid(&self) -> UID {
        self.uid
    }

    pub fn next(&self) -> Option<UID> {
        self.next_uid
    }

    pub fn previous(&self) -> Option<UID> {
        self.previous_uid
    }

    /// set this to be after the other. also modifies other to be before this
    pub fn set_after(&mut self, other: &mut CircularUID) -> &mut CircularUID {
        self.previous_uid = Some(other.uid());
        other.next_uid = Some(self.uid());
        self
    }

    /// set this to be before the other. also modifies other to be after this
    pub fn set_before(&mut self, other: &mut CircularUID) -> &mut CircularUID {
        self.next_uid = Some(other.uid());
        other.previous_uid = Some(self.uid());
        self
    }

    /// sets this UID as before and after itself
    pub fn single_id_loop(&mut self) -> &mut CircularUID {
        self.previous_uid = Some(self.uid());
        self.next_uid = Some(self.uid());
        self
    }
}

/// exact same as CircularUID but an interior mutability reference wrapper
#[derive(Debug, Clone, Copy)]
pub struct RefCircularUIDCell<'a>(pub &'a Cell<CircularUID>);

impl<'a> RefCircularUIDCell<'a> {
    pub fn uid(&self) -> UID {
        self.0.get().uid()
    }

    pub fn next(&self) -> Option<UID> {
        self.0.get().next()
    }

    pub fn previous(&self) -> Option<UID> {
        self.0.get().previous()
    }

    pub fn set_after(&self, other: &RefCircularUIDCell) -> &Self {
        let mut me = self.0.get();
        let mut o = other.0.get();
        me.set_after(&mut o);
        self.0.set(me);
        other.0.set(o);
        self
    }

    pub fn set_before(&self, other: &RefCircularUIDCell) -> &Self {
        let mut me = self.0.get();
        let mut o = other.0.get();
        me.set_before(&mut o);
        self.0.set(me);
        other.0.set(o);
        self
    }

    pub fn single_id_loop(&self) -> &Self {
        let mut me = self.0.get();
        me.single_id_loop();
        self.0.set(me);
        self
    }
}

/// a widget can be the current focus. how a widget handles what that means is
/// up to it. only zero or one widgets should be focused at a time.
#[derive(Default)]
pub struct FocusManager(pub Option<UID>);


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

/// subset of WidgetUpdateEvent, where there is Some(focus_manager)
pub struct WidgetEventFocusSubset<'sdl> {
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
    pub fn is_focused(&self, other: UID) -> bool {
        self.0.map(|uid| uid == other).unwrap_or(false)
    }

    /// handle default behavior for how focus should change given the events:
    /// - mouse moved over widget gains focus
    /// - if focused:
    ///     - tab goes to next, shift + tab goes to previous (consumes events)
    ///     - escape key causes unfocus (consumes event)
    pub fn default_widget_focus_behavior(my_focus_id: CircularUID, event: WidgetEventFocusSubset) {
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
                if !event.focus_manager.is_focused(my_focus_id.uid()) {
                    return; // only process tab if I am focused
                }
                event.event.set_consumed();
                if repeat {
                    return;
                }
                if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                    // shift tab was pressed
                    event.focus_manager.0 = my_focus_id.previous_uid;
                } else {
                    // tab was pressed
                    event.focus_manager.0 = my_focus_id.next_uid;
                }
            }
            sdl2::event::Event::KeyDown {
                repeat,
                keycode: Some(Keycode::ESCAPE),
                ..
            } => {
                if !event.focus_manager.is_focused(my_focus_id.uid()) {
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
                        event.focus_manager.0 = Some(my_focus_id.uid());
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
        start_widget_focus_id: UID,
        end_widget_focus_id: UID,
    ) {
        for sdl_input in events.iter_mut().filter(|e| e.available()) {
            if let sdl2::event::Event::KeyDown {
                    repeat,
                    keycode: Some(Keycode::Tab),
                    keymod,
                    ..
                } = sdl_input.e {
                sdl_input.set_consumed();
                if repeat {
                    continue;
                }
                if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                    // shift tab was pressed
                    self.0 = Some(end_widget_focus_id);
                } else {
                    // tab was pressed
                    self.0 = Some(start_widget_focus_id);
                }
            }
        }
    }
}
