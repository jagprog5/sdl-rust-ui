use sdl2::render::WindowCanvas;

use crate::util::{
    focus::FocusManager,
    length::{
        clamp, AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy,
        PreferredPortion,
    },
};

pub struct SDLEvent {
    pub e: sdl2::event::Event,
    /// indicates if this event was consumed by a widget (indicating it should
    /// not be used by other things)
    consumed_status: bool,
}

impl SDLEvent {
    pub fn set_consumed(&mut self) {
        self.consumed_status = true;
    }

    pub fn consumed(&self) -> bool {
        self.consumed_status
    }

    pub fn available(&self) -> bool {
        !self.consumed_status
    }

    pub fn new(e: sdl2::event::Event) -> Self {
        Self {
            e,
            consumed_status: false,
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
    /// the position that this widget is at. None when zero area
    // one might wonder - why is FRect being used instead of Rect?
    // - started running into issues where a one pixel difference leads to a
    //   visible jump. specifically, when a label font size changes in
    //   horizontal layout (a one pixel in height leading to a larger difference
    //   in width due to aspect ratio)
    // - rect uses i32 for pos and u32 for length. simpler to always use f32
    // - sdl2 has an f32 API 
    pub position: Option<sdl2::rect::FRect>,
    /// in the context of where this widget is in the GUI, does the width or the
    /// height have priority in regard to enforcing an aspect ratio
    pub aspect_ratio_priority: AspectRatioPreferredDirection,
    /// handle all events from sdl. contains events in order of receival
    pub events: &'sdl mut [SDLEvent],
    /// draw the widget and children
    pub canvas: &'sdl mut sdl2::render::WindowCanvas,
}

impl<'sdl> WidgetEvent<'sdl> {
    /// create a new event, same as self, but with a different position.
    /// intended to be passed to a layout's children
    pub fn sub_event(
        &mut self,
        position: Option<sdl2::rect::FRect>,
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

    /// implementors should use this to request an aspect ratio
    fn preferred_width_from_height(&mut self, _pref_h: f32) -> Option<Result<f32, String>> {
        None
    }

    /// implementors should use this to request an aspect ratio
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

    /// draw thyself
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
                sdl2::rect::FRect::new(0., 0., w as f32, h as f32),
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
    parent: sdl2::rect::FRect,
    ratio_priority: AspectRatioPreferredDirection,
) -> Result<Option<sdl2::rect::FRect>, String> {
    let (max_w, max_h) = widget.max()?;
    let (min_w, min_h) = widget.min()?;
    let (preferred_portion_w, preferred_portion_h) = widget.preferred_portion();
    let pre_clamp_w = preferred_portion_w.get(parent.width());
    let pre_clamp_h = preferred_portion_h.get(parent.height());
    let mut w = clamp(pre_clamp_w, min_w, max_w);
    let mut h = clamp(pre_clamp_h, min_h, max_h);

    match ratio_priority {
        AspectRatioPreferredDirection::WidthFromHeight => {
            if let Some(new_w) = widget.preferred_width_from_height(h) {
                let new_w = new_w?;
                w = clamp(new_w, min_w, max_w.strictest(MaxLen(pre_clamp_w)));
            }
        }
        AspectRatioPreferredDirection::HeightFromWidth => {
            if let Some(new_h) = widget.preferred_height_from_width(w) {
                let new_h = new_h?;
                h = clamp(new_h, min_h, max_h.strictest(MaxLen(pre_clamp_h)));
            }
        }
    }

    if w == 0. || h == 0. {
        return Ok(None);
    }

    let x_offset = crate::util::length::place(
        w,
        parent.width(),
        widget.min_w_fail_policy(),
        widget.max_w_fail_policy(),
    );
    let y_offset = crate::util::length::place(
        h,
        parent.height(),
        widget.min_h_fail_policy(),
        widget.max_h_fail_policy(),
    );
    Ok(Some(sdl2::rect::FRect::new(
        parent.x() + x_offset,
        parent.y() + y_offset,
        w,
        h,
    )))
}
