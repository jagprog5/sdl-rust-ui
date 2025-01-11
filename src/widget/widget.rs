use sdl2::render::WindowCanvas;

use crate::util::{
    focus::FocusManager,
    length::{
        clamp, AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy,
        PreferredPortion,
    }, rect::FRect,
};

#[derive(Debug, Clone, Copy)]
pub enum ConsumedStatus {
    /// this event has not been consumed by any widget
    None,

    /// this event has been consumed by a non-layout widget. for the most part,
    /// it should be considered consumed, but it might still be used by layouts
    /// (e.g. scroller)
    ConsumedByWidget,

    /// this event has been consumed by a layout, and should not be used by
    /// anything else
    ConsumedByLayout,
}

pub struct SDLEvent {
    pub e: sdl2::event::Event,
    consumed_status: ConsumedStatus,
}

impl SDLEvent {
    pub fn consumed(&self) -> bool {
        match self.consumed_status {
            ConsumedStatus::None => false,
            _ => true
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
        debug_assert!(match self.consumed_status {
            ConsumedStatus::None => true,
            _ => false,
        });
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

/// a widget interacts with sdl by receiving events and drawing itself
pub struct WidgetEvent<'sdl> {
    /// stores state indicating which widget has focus  
    /// none if this widget isn't inserted in a context which is focusable. for
    /// example, a label contained in a button is not focusable (the parent
    /// button is instead).  
    /// or alternatively, the focus manager is None if None is passed to
    /// update_gui
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
    /// in the context of where this widget is in the GUI, does the width or the
    /// height have priority in regard to enforcing an aspect ratio
    pub aspect_ratio_priority: AspectRatioPreferredDirection,
    /// handle all events from sdl. contains events in order of occurrence
    pub events: &'sdl mut [SDLEvent],
    /// draw the widget and children
    pub canvas: &'sdl mut sdl2::render::WindowCanvas,
}

impl<'sdl> WidgetEvent<'sdl> {
    /// create a new event, same as self, but with a different position.
    /// intended to be passed to a layout's children
    pub fn sub_event(
        &mut self,
        position: FRect,
    ) -> WidgetEvent<'_> {
        WidgetEvent {
            // do a re-borrow. create a mutable borrow of the mutable borrow
            // output lifetime is elided - it's the re-borrowed lifetime
            focus_manager: match &mut self.focus_manager {
                Some(v) => Some(&mut *v),
                None => None,
            },
            position,
            aspect_ratio_priority: self.aspect_ratio_priority,
            events: &mut *self.events,
            canvas: &mut *self.canvas,
        }
    }

    pub fn dup(&mut self) -> WidgetEvent<'_> {
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

    /// receive input and change any state which could be viewed by other
    /// widgets. happens before draw for all widgets for each frame
    fn update(&mut self, _event: WidgetEvent) -> Result<(), String> {
        Ok(())
    }


    /// draw self and update state. called each frame after update
    /// 
    /// state which might be viewed between widgets should instead be updated in
    /// update
    fn draw(&mut self, event: WidgetEvent) -> Result<(), String>;
}

macro_rules! generate_gui_function {
    ($fn_name:ident, $widget_action:ident) => {
        pub fn $fn_name(
            widget: &mut dyn Widget,
            canvas: &mut WindowCanvas,
            events: &mut [SDLEvent],
            focus_manager: Option<&mut FocusManager>,
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

            let widget_event = WidgetEvent {
                position,
                events,
                canvas,
                aspect_ratio_priority: AspectRatioPreferredDirection::default(),
                focus_manager,
            };
            widget.$widget_action(widget_event)
        }
    };
}

generate_gui_function!(update_gui, update);
generate_gui_function!(draw_gui, draw);

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
