use crate::{
    util::length::{
        clamp, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen, MinLenFailPolicy, MinLenPolicy,
        PreferredPortion,
    },
    widget::widget::{Widget, WidgetEvent},
};

use super::vertical_layout::MajorAxisMaxLenPolicy;

pub struct HorizontalLayout<'sdl> {
    pub elems: Vec<&'sdl mut dyn Widget>,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,
    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,
    pub min_w_policy: MinLenPolicy,
    pub max_w_policy: MajorAxisMaxLenPolicy,
    pub min_h_policy: MinLenPolicy,
    pub max_h_policy: MaxLenPolicy,
}

impl<'sdl> Default for HorizontalLayout<'sdl> {
    fn default() -> Self {
        Self {
            elems: Default::default(),
            preferred_w: Default::default(),
            preferred_h: Default::default(),
            min_w_fail_policy: Default::default(),
            max_w_fail_policy: Default::default(),
            min_h_fail_policy: Default::default(),
            max_h_fail_policy: Default::default(),
            min_w_policy: MinLenPolicy::Children,
            min_h_policy: MinLenPolicy::Literal(MinLen::LAX),
            max_w_policy: MajorAxisMaxLenPolicy::Together(MaxLenPolicy::Children),
            max_h_policy: MaxLenPolicy::Literal(MaxLen::LAX),
        }
    }
}

// macro to reuse code for update vs draw
macro_rules! impl_widget_fn {
    ($fn_name:ident) => {
        fn $fn_name(&mut self, mut event: WidgetEvent) -> Result<(), String> {
            let is_pos_non_empty: Option<sdl2::rect::Rect> = event.position.into();
            if let None = is_pos_non_empty {
                // even if there is no draw position, still always propagate all
                // events to all children. consistency
                for elem in self.elems.iter_mut() {
                    let mut sub_event = event.sub_event(event.position);
                    sub_event.aspect_ratio_priority =
                        crate::util::length::AspectRatioPreferredDirection::HeightFromWidth;
                    elem.$fn_name(sub_event)?;
                }
                return Ok(());
            }

            // collect info from child components
            let mut info: Vec<ChildInfo> = vec![ChildInfo::default(); self.elems.len()];
            let mut sum_preferred_horizontal = PreferredPortion::EMPTY;
            for (i, elem) in self.elems.iter_mut().enumerate() {
                let (min_w, min_h) = elem.min()?;
                let (max_w, max_h) = elem.max()?;
                let (pref_w, pref_h) = elem.preferred_portion();

                info[i].max_vertical = max_h;
                info[i].min_vertical = min_h;
                info[i].preferred_vertical = pref_h;

                info[i].max_horizontal = max_w.0;
                info[i].min_horizontal = min_w.0;
                info[i].preferred_horizontal = pref_w;

                sum_preferred_horizontal.0 += pref_w.0;
            }

            let mut amount_taken = 0f32;
            let mut amount_given = 0f32;
            for info in info.iter_mut() {
                info.width = info.preferred_horizontal.weighted_portion(
                    sum_preferred_horizontal,
                    self.elems.len(),
                    event.position.w,
                );
                if info.width < info.min_horizontal {
                    // it is being made larger than it would prefer.
                    // take some len from the other components
                    amount_taken += info.min_horizontal - info.width;
                    info.width = info.min_horizontal;
                } else if info.width > info.max_horizontal {
                    // it is being made smaller than it would prefer.
                    // give some len to the other components
                    amount_given += info.width - info.max_horizontal;
                    info.width = info.max_horizontal;
                }
            }

            if amount_given >= amount_taken {
                let excess = amount_given - amount_taken;
                distribute_excess(&mut info, excess);
            } else {
                let deficit = amount_taken - amount_given;
                take_deficit(&mut info, deficit);
            }

            let mut sum_display_width = 0f32;
            for info in info.iter() {
                sum_display_width += info.width;
            }

            let horizontal_space = if sum_display_width < event.position.w {
                if self.elems.len() == 0 {
                    return Ok(());
                }

                if self.elems.len() == 1 {
                    let position = crate::widget::widget::place(
                        self.elems[0],
                        event.position,
                        crate::util::length::AspectRatioPreferredDirection::HeightFromWidth,
                    )?;
                    let mut sub_event = event.sub_event(position);
                    sub_event.aspect_ratio_priority =
                        crate::util::length::AspectRatioPreferredDirection::HeightFromWidth;
                    self.elems[0].$fn_name(sub_event)?;
                    return Ok(());
                }

                let extra_space = event.position.w - sum_display_width;
                debug_assert!(self.elems.len() > 0);
                let num_spaces = self.elems.len() as u32 - 1;

                debug_assert!(num_spaces != 0);
                let extra_space_per_elem = extra_space / num_spaces as f32;
                extra_space_per_elem
            } else {
                0.
            };

            let mut x_pos = event.position.x;
            let mut e_err_accumulation = 0f32;
            for (elem, info) in self.elems.iter_mut().zip(info.iter_mut()) {
                // handle accumulation of errors. this is needed for things to look pixel perfect with many children
                e_err_accumulation += info.width - info.width.floor();
                info.width = info.width.floor();
                if e_err_accumulation >= 1. {
                    info.width += 1.;
                    e_err_accumulation -= 1.;
                }
                let pre_clamp_height = info.preferred_vertical.get(event.position.h);
                let mut height = clamp(pre_clamp_height, info.min_vertical, info.max_vertical);
                if let Some(new_h) = elem.preferred_height_from_width(info.width) {
                    let new_h = new_h?;
                    let new_h_max_clamp = if elem.preferred_link_allowed_exceed_portion() {
                        info.max_vertical
                    } else {
                        info.max_vertical.strictest(MaxLen(pre_clamp_height))
                    };
                    height = clamp(new_h, info.min_vertical, new_h_max_clamp);
                }

                let y = crate::util::length::place(
                    height,
                    event.position.h,
                    elem.min_h_fail_policy(),
                    elem.max_h_fail_policy(),
                ) + event.position.y;

                let mut sub_event = event.sub_event(crate::util::rect::FRect {x: x_pos, y, w: info.width, h: height});
                sub_event.aspect_ratio_priority =
                    crate::util::length::AspectRatioPreferredDirection::HeightFromWidth;
                elem.$fn_name(sub_event)?;
                x_pos += info.width;
                x_pos += horizontal_space as f32;
            }
            Ok(())
        }
    };
}

impl<'sdl> Widget for HorizontalLayout<'sdl> {
    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.preferred_w, self.preferred_h)
    }

    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        let w_view_children = match self.min_w_policy {
            MinLenPolicy::Children => None,
            MinLenPolicy::Literal(min_len) => Some(min_len),
        };

        let h_view_children = match self.min_h_policy {
            MinLenPolicy::Children => None,
            MinLenPolicy::Literal(min_len) => Some(min_len),
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
            width_so_far = width_so_far.combined(elem_min_w);
            height_so_far = height_so_far.strictest(elem_min_h);
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
        self.min_w_fail_policy
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.min_h_fail_policy
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        let w_view_children = match self.max_w_policy {
            MajorAxisMaxLenPolicy::Spread => Some(MaxLen::LAX),
            MajorAxisMaxLenPolicy::Together(max_len_policy) => match max_len_policy {
                MaxLenPolicy::Children => None,
                MaxLenPolicy::Literal(max_len) => Some(max_len),
            },
        };

        let h_view_children = match self.max_h_policy {
            MaxLenPolicy::Children => None,
            MaxLenPolicy::Literal(max_len) => Some(max_len),
        };

        if let Some(w) = w_view_children {
            if let Some(h) = h_view_children {
                return Ok((w, h)); // no need to iterate children in this case
            }
        }

        let mut height_so_far = MaxLen(0.);
        let mut width_so_far = MaxLen::LAX;

        for elem in self.elems.iter_mut() {
            let (elem_max_w, elem_max_h) = elem.max()?;
            width_so_far = width_so_far.combined(elem_max_w);
            height_so_far = height_so_far.strictest(elem_max_h);
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
        self.max_w_fail_policy
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_h_fail_policy
    }

    impl_widget_fn!(update);
    impl_widget_fn!(draw);
}

#[derive(Clone, Copy)]
struct ChildInfo {
    preferred_horizontal: PreferredPortion,
    max_horizontal: f32,
    min_horizontal: f32,

    // iterated upon by the layout
    width: f32,

    preferred_vertical: PreferredPortion,
    max_vertical: MaxLen,
    min_vertical: MinLen,
}

impl Default for ChildInfo {
    fn default() -> Self {
        Self {
            preferred_horizontal: Default::default(),
            max_horizontal: Default::default(),
            min_horizontal: Default::default(),
            width: Default::default(),
            preferred_vertical: Default::default(),
            max_vertical: Default::default(),
            min_vertical: Default::default(),
        }
    }
}

/// effects the behavior of sizing for vertical layout and horizontal layout.
///
/// regardless of the chosen value, sizing nearly always completes in 1-3
/// iterations.
///
/// if set to None, this will always give the correct result, but sizing has
/// time complexity O(n^2); a max of # children iterations will be done.
///
/// if set to Some(v), then a max of v iterations will be done. this will nearly
/// always give correct results except for pathologically complex layouts.
/// incorrect layout may have small gaps or overlaps between components
///
/// recommended Some(15)
pub(crate) const RUN_OFF_SIZING_AMOUNT: Option<usize> = Some(15);

/// given some amount of excess length, distributed to all components in a way
/// that respects the minimum and distributes the length equally by component
/// weight
fn distribute_excess(info: &mut [ChildInfo], mut excess: f32) {
    let num_iters = match RUN_OFF_SIZING_AMOUNT {
        Some(v) => v,
        None => info.len(),
    };

    for _ in 0..num_iters {
        if excess == 0. {
            return;
        }
        let mut excess_from_excess = 0f32;

        let mut available_weight = 0f32;
        for info in info.iter() {
            if info.width < info.max_horizontal {
                available_weight += info.preferred_horizontal.0;
            }
        }

        for info in info.iter_mut() {
            if info.width < info.max_horizontal {
                let ideal_amount_to_give =
                    (info.preferred_horizontal.0 / available_weight) * excess;
                let max_amount_to_give = info.max_horizontal - info.width;
                if ideal_amount_to_give > max_amount_to_give {
                    info.width = info.max_horizontal;
                    excess_from_excess += ideal_amount_to_give - max_amount_to_give;
                } else {
                    info.width += ideal_amount_to_give;
                }
            }
        }
        excess = excess_from_excess;
    }
}

/// given some amount of length that needs to be sourced by other components,
/// source it in a way that distributes the loss equally by component weight,
/// and respects the minimums and maximums
fn take_deficit(info: &mut [ChildInfo], mut deficit: f32) {
    let num_iters = match RUN_OFF_SIZING_AMOUNT {
        Some(v) => v,
        None => info.len(),
    };

    for _ in 0..num_iters {
        let mut deficit_from_deficit = 0f32;

        let mut available_weight = 0f32;
        for info in info.iter() {
            if info.width > info.min_horizontal {
                available_weight += info.preferred_horizontal.0;
            }
        }

        for info in info.iter_mut() {
            if info.width > info.min_horizontal {
                let ideal_amount_to_take =
                    (info.preferred_horizontal.0 / available_weight) * deficit;
                let max_amount_to_take = info.width - info.min_horizontal;
                if ideal_amount_to_take > max_amount_to_take {
                    info.width = info.min_horizontal;
                    deficit_from_deficit += ideal_amount_to_take - max_amount_to_take;
                } else {
                    info.width -= ideal_amount_to_take;
                }
            }
        }
        deficit = deficit_from_deficit;
        if deficit == 0. {
            return;
        }
    }
}
