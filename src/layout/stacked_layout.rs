use crate::{
    util::length::{
        MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion,
    },
    widget::widget::{Widget, WidgetEvent},
};

pub struct StackedLayoutLiteralSizing {
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,
    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,
    pub min_w: MinLen,
    pub max_w: MaxLen,
    pub min_h: MinLen,
    pub max_h: MaxLen,
}

impl Default for StackedLayoutLiteralSizing {
    fn default() -> Self {
        Self {
            preferred_w: Default::default(),
            preferred_h: Default::default(),
            min_w_fail_policy: Default::default(),
            max_w_fail_policy: Default::default(),
            min_h_fail_policy: Default::default(),
            max_h_fail_policy: Default::default(),
            min_w: Default::default(),
            max_w: Default::default(),
            min_h: Default::default(),
            max_h: Default::default(),
        }
    }
}

pub enum StackedLayoutSizingPolicy {
    /// uses the sizing from the children in this layout.
    /// 
    /// the sizing is inherited from the children as follow:
    ///  - the maximum and minimum lengths are whichever is strictest from any
    ///    of the children (which is typically the top most child)
    ///  - all other properties are inherited from the top most child
    Children,
    /// states literally, ignoring the children
    Literal(StackedLayoutLiteralSizing),
}

impl Default for StackedLayoutSizingPolicy {
    fn default() -> Self {
        StackedLayoutSizingPolicy::Children
    }
}

/// draws several widgets over top of each other.
/// typically used for a background, and some element in the foreground
pub struct StackedLayout<'sdl> {
    pub elems: Vec<&'sdl mut dyn Widget>,
    pub sizing_policy: StackedLayoutSizingPolicy,
}

impl<'sdl> Default for StackedLayout<'sdl> {
    fn default() -> Self {
        Self {
            elems: Default::default(),
            sizing_policy: Default::default(),
        }
    }
}

// macro to reuse code for update vs draw
macro_rules! impl_widget_fn {
    ($fn_name:ident) => {
        fn $fn_name(&mut self, mut event: WidgetEvent) -> Result<(), String> {
            let position = match event.position {
                Some(v) => v,
                None => {
                    // even if there is no draw position, still always propagate all
                    // events to all children
                    for elem in self.elems.iter_mut() {
                        elem.$fn_name(event.sub_event(None))?;
                    }
                    return Ok(());
                }
            };

            for elem in self.elems.iter_mut() {
                let pos_for_child = crate::widget::widget::place(
                    &mut **elem,
                    position,
                    event.aspect_ratio_priority,
                )?;
                elem.$fn_name(event.sub_event(pos_for_child))?;
            }
            Ok(())
        }
    };
}

impl<'sdl> Widget for StackedLayout<'sdl> {
    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        let w_view_children = match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => None,
            StackedLayoutSizingPolicy::Literal(stacked_layout_literal_sizing) => Some(stacked_layout_literal_sizing.min_w),
        };

        let h_view_children = match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => None,
            StackedLayoutSizingPolicy::Literal(stacked_layout_literal_sizing) => Some(stacked_layout_literal_sizing.min_h),
        };

        if let Some(w) = w_view_children {
            if let Some(h) = h_view_children {
                return Ok((w, h)); // no need to iterate children in this case
            }
        }

        let mut height_so_far = MinLen::LAX;
        let mut width_so_far = MinLen::LAX;

        for elem in self.elems.iter_mut() {
            let (elem_min_w, elem_min_h) = elem.min()?;
            height_so_far = height_so_far.strictest(elem_min_h);
            width_so_far = width_so_far.strictest(elem_min_w);
        }

        Ok((
            match w_view_children {
                Some(w) => w,
                None => width_so_far,
            },
            match h_view_children {
                Some(h) => h,
                None => height_so_far,
            },
        ))
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => {
                match self.elems.last() {
                    Some(v) => v.min_w_fail_policy(),
                    None => MinLenFailPolicy::default(),
                }
            },
            StackedLayoutSizingPolicy::Literal(stacked_layout_literal_sizing) => {
                stacked_layout_literal_sizing.min_w_fail_policy
            }
        }
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => {
                match self.elems.last() {
                    Some(v) => v.min_h_fail_policy(),
                    None => MinLenFailPolicy::default(),
                }
            },
            StackedLayoutSizingPolicy::Literal(stacked_layout_literal_sizing) => {
                stacked_layout_literal_sizing.min_h_fail_policy
            }
        }
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        let w_view_children = match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => None,
            StackedLayoutSizingPolicy::Literal(stacked_layout_literal_sizing) => Some(stacked_layout_literal_sizing.max_w),
        };

        let h_view_children = match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => None,
            StackedLayoutSizingPolicy::Literal(stacked_layout_literal_sizing) => Some(stacked_layout_literal_sizing.max_h),
        };

        if let Some(w) = w_view_children {
            if let Some(h) = h_view_children {
                return Ok((w, h)); // no need to iterate children in this case
            }
        }

        let mut height_so_far = MaxLen::LAX;
        let mut width_so_far = MaxLen::LAX;

        for elem in self.elems.iter_mut() {
            let (elem_max_w, elem_max_h) = elem.max()?;
            height_so_far = height_so_far.strictest(elem_max_h);
            width_so_far = width_so_far.strictest(elem_max_w);
        }

        Ok((
            match w_view_children {
                Some(w) => w,
                None => width_so_far,
            },
            match h_view_children {
                Some(h) => h,
                None => height_so_far,
            },
        ))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => {
                match self.elems.last() {
                    Some(v) => v.max_w_fail_policy(),
                    None => MaxLenFailPolicy::default(),
                }
            },
            StackedLayoutSizingPolicy::Literal(stacked_layout_literal_sizing) => {
                stacked_layout_literal_sizing.max_w_fail_policy
            }
        }
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => {
                match self.elems.last() {
                    Some(v) => v.max_h_fail_policy(),
                    None => MaxLenFailPolicy::default(),
                }
            },
            StackedLayoutSizingPolicy::Literal(stacked_layout_literal_sizing) => {
                stacked_layout_literal_sizing.max_h_fail_policy
            }
        }
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => {
                match self.elems.last() {
                    Some(v) => v.preferred_portion(),
                    None => (PreferredPortion::default(), PreferredPortion::default()),
                }
            },
            StackedLayoutSizingPolicy::Literal(stacked_layout_literal_sizing) => {
                (stacked_layout_literal_sizing.preferred_w, stacked_layout_literal_sizing.preferred_h)
            }
        }
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        match &mut self.sizing_policy {
            StackedLayoutSizingPolicy::Children => {
                match self.elems.last_mut() {
                    Some(v) => v.preferred_width_from_height(pref_h),
                    None => None,
                }
            },
            StackedLayoutSizingPolicy::Literal(_) => {
                None
            },
        }
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        match &mut self.sizing_policy {
            StackedLayoutSizingPolicy::Children => {
                match self.elems.last_mut() {
                    Some(v) => v.preferred_height_from_width(pref_w),
                    None => None,
                }
            },
            StackedLayoutSizingPolicy::Literal(_) => {
                None
            },
        }
    }

    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        match &self.sizing_policy {
            StackedLayoutSizingPolicy::Children => {
                match self.elems.last() {
                    Some(v) => v.preferred_link_allowed_exceed_portion(),
                    None => Default::default(),
                }
            },
            StackedLayoutSizingPolicy::Literal(_) => {
                Default::default()
            },
        }
    }

    impl_widget_fn!(update);
    impl_widget_fn!(draw);
}
