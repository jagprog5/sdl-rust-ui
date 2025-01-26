pub mod debug;
pub mod strut;
pub mod texture;

pub mod border;

pub mod multi_line_label;
pub mod single_line_label;
pub mod single_line_text_input;

pub mod background;
pub mod checkbox;

pub mod button;

use sdl2::render::{ClippingRect, WindowCanvas};

use crate::util::{
    focus::FocusManager,
    length::{
        clamp, AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy,
        PreferredPortion,
    },
    rect::FRect,
    rust::reborrow,
};

/// two purposes:
///  - used to indicate which events were not used by the UI and should be
///    passed down to the rest of the application
///  - used to ensure that a single widget uses an event
#[derive(Debug, Clone, Copy)]
pub enum ConsumedStatus {
    /// this event has not been consumed by any widget
    None,

    /// this event has been consumed by a non-layout widget. for the most part,
    /// it should be considered consumed, but it might still be used by layouts
    /// (e.g. scroller). this distinction was required for nested scroll widgets
    /// to work (a scroller's contained widget is given the opportunity to
    /// consume events first. that way an inner scroller can consume some scroll
    /// amount before the outer scroller. but if the child is instead something
    /// else which would consume events and prevent a scroll, then it is
    /// ignored)
    ConsumedByWidget,

    /// this event has been consumed by a layout, and should not be used by
    /// anything else
    ConsumedByLayout,
}

#[derive(Debug)]
pub struct SDLEvent {
    pub e: sdl2::event::Event,
    consumed_status: ConsumedStatus,
}

impl SDLEvent {
    pub fn consumed(&self) -> bool {
        match self.consumed_status {
            ConsumedStatus::None => false,
            _ => true,
        }
    }

    pub fn available(&self) -> bool {
        !self.consumed()
    }

    pub fn consumed_status(&self) -> ConsumedStatus {
        self.consumed_status
    }

    pub fn set_consumed(&mut self) {
        // shouldn't be consumed twice
        debug_assert!(matches!(self.consumed_status, ConsumedStatus::None));
        self.consumed_status = ConsumedStatus::ConsumedByWidget;
    }

    pub fn set_consumed_by_layout(&mut self) {
        debug_assert!(match self.consumed_status {
            ConsumedStatus::ConsumedByLayout => false,
            _ => true,
        });
        self.consumed_status = ConsumedStatus::ConsumedByLayout;
    }

    pub fn new(e: sdl2::event::Event) -> Self {
        Self {
            e,
            consumed_status: ConsumedStatus::None,
        }
    }
}

pub struct WidgetUpdateEvent<'sdl> {
    /// stores state indicating which widget has focus  
    /// none if this widget isn't inserted in a context which is focusable. for
    /// example, a label contained in a button is not focusable (the parent
    /// button is instead).  
    /// or alternatively, the focus manager is None if None is passed to
    /// update_gui (because the user of this lib doesn't care about focus)
    pub focus_manager: Option<&'sdl mut FocusManager>,
    /// the position that this widget is at. this is NOT an sdl2::rect::FRect
    // it's important to keep the sizing as floats as the sizing is being
    // computed.
    // - otherwise there's a lot of casting to and from integer. best to keep it
    //   as floating point until just before use
    // - started running into issues where a one pixel difference leads to a
    //   visible jump. specifically, when a label font size changes in
    //   horizontal layout (a one pixel in height leading to a larger difference
    //   in width due to aspect ratio)
    // - sdl2 has an f32 API
    pub position: FRect,
    /// although the object is updated at a position, give also the clipping rect
    /// that will be in effect once the widget is drawn
    pub clipping_rect: ClippingRect,
    /// which window is being update
    pub window_id: u32,
    /// in the context of where this widget is in the GUI, does the width or the
    /// height have priority in regard to enforcing an aspect ratio
    pub aspect_ratio_priority: AspectRatioPreferredDirection,
    /// handle all events from sdl. contains events in order of occurrence
    pub events: &'sdl mut [SDLEvent],
}

impl<'sdl> WidgetUpdateEvent<'sdl> {
    /// create a new event, same as self, but with a different position.
    /// intended to be passed to a layout's children
    pub fn sub_event(&mut self, position: FRect) -> WidgetUpdateEvent<'_> {
        WidgetUpdateEvent {
            // do a re-borrow. create a mutable borrow of the mutable borrow
            // output lifetime is elided - it's the re-borrowed lifetime
            focus_manager: self.focus_manager.as_mut().map(|f| reborrow(*f)),
            position,
            clipping_rect: self.clipping_rect,
            window_id: self.window_id,
            aspect_ratio_priority: self.aspect_ratio_priority,
            events: reborrow(self.events),
        }
    }

    pub fn dup(&mut self) -> WidgetUpdateEvent<'_> {
        self.sub_event(self.position)
    }
}

pub trait Widget {
    /// the widget will never have a width or height smaller than this width or
    /// height, respectively.
    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        Ok((MinLen::LAX, MinLen::LAX))
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        MinLenFailPolicy::CENTERED
    }
    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        MinLenFailPolicy::CENTERED
    }

    /// the widget will never have a width or height greater than this width or
    /// height, respectively, unless it would conflict with the minimum width or
    /// height, respectively.
    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        Ok((MaxLen::LAX, MaxLen::LAX))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        MaxLenFailPolicy::CENTERED
    }
    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        MaxLenFailPolicy::CENTERED
    }

    /// portion of parent. sometimes used as a weight between competing components
    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (PreferredPortion::FULL, PreferredPortion::FULL)
    }

    /// implementors should use this to request an aspect ratio (additionally,
    /// the min and max should have the same ratio)
    fn preferred_width_from_height(&mut self, _pref_h: f32) -> Option<Result<f32, String>> {
        None
    }

    /// implementors should use this to request an aspect ratio (additionally,
    /// the min and max should have the same ratio)
    fn preferred_height_from_width(&mut self, _pref_w: f32) -> Option<Result<f32, String>> {
        None
    }

    /// generally this shouldn't be changed from the default implementation.
    ///
    /// this effects the behavior of preferred_width_from_height and
    /// preferred_height_from_width.
    ///
    /// if true is returned, the output from those function is not restricted to
    /// be within the preferred portion of the parent (unless this would
    /// conflict with the min len)
    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        false
    }

    /// called for all widgets each frame before any call to draw
    fn update(&mut self, _event: WidgetUpdateEvent) -> Result<(), String> {
        Ok(())
    }

    /// might be called after update
    ///
    /// this occurs in a context where the position of the widget can change
    /// this frame. e.g. inside a scroller, the initial position can change once
    /// scrolling has occurred (but after the contained widget was updated)
    ///
    /// it is recommended, but not required, that implementors of widget use
    /// this function; the new position will be available in the next call to
    /// update anyway but this makes the position change this frame and not next
    /// frame
    fn update_adjust_position(&mut self, _pos_delta: (i32, i32)) {}

    /// draw. called after all widgets are update each frame
    fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        focus_manager: Option<&FocusManager>,
    ) -> Result<(), String>;
}

/// update the gui. returns a rect that should be passed to draw.
/// between update and draw, the canvas's size should not change
///
/// each frame after update_gui, the widget should be drawn with widget.draw()
pub fn update_gui(
    widget: &mut dyn Widget,
    events: &mut [SDLEvent],
    focus_manager: Option<&mut FocusManager>,
    canvas: &WindowCanvas,
) -> Result<(), String> {
    let (w, h) = match canvas.output_size() {
        Ok(v) => v,
        Err(msg) => {
            debug_assert!(false, "{}", msg); // infallible in prod
            (320, 320)
        }
    };

    let aspect_ratio_priority = AspectRatioPreferredDirection::default();

    let position = place(
        widget,
        FRect {
            x: 0.,
            y: 0.,
            w: w as f32,
            h: h as f32,
        },
        aspect_ratio_priority,
    )?;

    let widget_event = WidgetUpdateEvent {
        position,
        events,
        aspect_ratio_priority: AspectRatioPreferredDirection::default(),
        focus_manager,
        clipping_rect: ClippingRect::None,
        window_id: canvas.window().id(),
    };
    widget.update(widget_event)?;
    Ok(())
}

/// given a widget's min, max lengths and fail policies, what's the widget's
/// lengths and offset within the parent.
///
/// note that sdl2 rect, by definition, cannot have a zero length. if this would
/// occur, returns None
pub fn place(
    widget: &mut dyn Widget,
    parent: FRect,
    ratio_priority: AspectRatioPreferredDirection,
) -> Result<FRect, String> {
    let (max_w, max_h) = widget.max()?;
    let (min_w, min_h) = widget.min()?;
    let (preferred_portion_w, preferred_portion_h) = widget.preferred_portion();
    let pre_clamp_w = preferred_portion_w.get(parent.w);
    let pre_clamp_h = preferred_portion_h.get(parent.h);
    let mut w = clamp(pre_clamp_w, min_w, max_w);
    let mut h = clamp(pre_clamp_h, min_h, max_h);

    match ratio_priority {
        AspectRatioPreferredDirection::WidthFromHeight => {
            if let Some(new_w) = widget.preferred_width_from_height(h) {
                let new_w = new_w?;
                let new_w_max_clamp = if widget.preferred_link_allowed_exceed_portion() {
                    max_w
                } else {
                    max_w.strictest(MaxLen(pre_clamp_w))
                };
                w = clamp(new_w, min_w, max_w.strictest(new_w_max_clamp));
            }
        }
        AspectRatioPreferredDirection::HeightFromWidth => {
            if let Some(new_h) = widget.preferred_height_from_width(w) {
                let new_h = new_h?;
                let new_h_max_clamp = if widget.preferred_link_allowed_exceed_portion() {
                    max_h
                } else {
                    max_h.strictest(MaxLen(pre_clamp_h))
                };
                h = clamp(new_h, min_h, max_h.strictest(new_h_max_clamp));
            }
        }
    }

    let x_offset = crate::util::length::place(
        w,
        parent.w,
        widget.min_w_fail_policy(),
        widget.max_w_fail_policy(),
    );
    let y_offset = crate::util::length::place(
        h,
        parent.h,
        widget.min_h_fail_policy(),
        widget.max_h_fail_policy(),
    );

    Ok(FRect {
        x: parent.x + x_offset,
        y: parent.y + y_offset,
        w,
        h,
    })
}
