use crate::{
    util::{
        focus::FocusManager,
        length::{
            clamp, place, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen, MinLenFailPolicy,
            MinLenPolicy, PreferredPortion,
        },
    },
    widget::{Widget, WidgetUpdateEvent},
};

use super::horizontal_layout::RUN_OFF_SIZING_AMOUNT;

#[derive(Clone, Copy)]
pub enum MajorAxisMaxLenPolicy {
    /// the layout has an unbounded max length and extra space is divided
    /// equally between components
    Spread,

    // the layout's elements are grouped together
    Together(MaxLenPolicy),
}

pub(crate) fn direction_conditional_iter_mut<'a, T>(
    vec: &'a mut [T],
    reverse: bool,
) -> Box<dyn Iterator<Item = &'a mut T> + 'a> {
    if reverse {
        Box::new(vec.iter_mut().rev())
    } else {
        Box::new(vec.iter_mut())
    }
}

pub struct VerticalLayout<'sdl> {
    pub elems: Vec<&'sdl mut dyn Widget>,
    /// reverse the order IN TIME that elements are updated and drawn in. this
    /// does not affect the placement of elements in space
    pub reverse: bool,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,
    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,
    pub min_w_policy: MinLenPolicy,
    pub max_w_policy: MaxLenPolicy,
    pub min_h_policy: MinLenPolicy,
    pub max_h_policy: MajorAxisMaxLenPolicy,
}

impl<'sdl> Default for VerticalLayout<'sdl> {
    fn default() -> Self {
        Self {
            elems: Default::default(),
            reverse: Default::default(),
            preferred_w: Default::default(),
            preferred_h: Default::default(),
            min_w_fail_policy: Default::default(),
            max_w_fail_policy: Default::default(),
            min_h_fail_policy: Default::default(),
            max_h_fail_policy: Default::default(),
            min_w_policy: MinLenPolicy::Children,
            min_h_policy: MinLenPolicy::Children,
            max_w_policy: MaxLenPolicy::Literal(MaxLen::LAX),
            max_h_policy: MajorAxisMaxLenPolicy::Together(MaxLenPolicy::Children),
        }
    }
}

impl<'sdl> Widget for VerticalLayout<'sdl> {
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
            height_so_far = height_so_far.combined(elem_min_h);
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
        self.min_w_fail_policy
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.min_h_fail_policy
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        let w_view_children = match self.max_w_policy {
            MaxLenPolicy::Children => None,
            MaxLenPolicy::Literal(max_len) => Some(max_len),
        };

        let h_view_children = match self.max_h_policy {
            MajorAxisMaxLenPolicy::Spread => Some(MaxLen::LAX),
            MajorAxisMaxLenPolicy::Together(max_len_policy) => match max_len_policy {
                MaxLenPolicy::Children => None,
                MaxLenPolicy::Literal(max_len) => Some(max_len),
            },
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
            height_so_far = height_so_far.combined(elem_max_h);
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
        self.max_w_fail_policy
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_h_fail_policy
    }

    fn update(&mut self, mut event: WidgetUpdateEvent) -> Result<(), String> {
        if self.elems.is_empty() {
            return Ok(());
        }

        // collect various info from child components
        let mut sum_preferred_vertical = PreferredPortion(0.);
        let mut info: Vec<ChildInfo> = vec![ChildInfo::default(); self.elems.len()];
        for (i, elem) in direction_conditional_iter_mut(&mut self.elems, self.reverse).enumerate() {
            let (min_w, min_h) = elem.min()?;
            let (max_w, max_h) = elem.max()?;
            let (pref_w, pref_h) = elem.preferred_portion();

            info[i].max_horizontal = max_w;
            info[i].min_horizontal = min_w;
            info[i].preferred_horizontal = pref_w;

            info[i].max_vertical = max_h.0;
            info[i].min_vertical = min_h.0;
            info[i].preferred_vertical = pref_h;
            sum_preferred_vertical.0 += pref_h.0;
        }

        let mut amount_taken = 0f32;
        let mut amount_given = 0f32;
        for info in info.iter_mut() {
            info.height = info
                .preferred_vertical
                .weighted_portion(sum_preferred_vertical, event.position.h);

            let next_info_height = clamp(
                info.height,
                MinLen(info.min_vertical),
                MaxLen(info.max_vertical),
            );

            if info.height < next_info_height {
                // when clamped, it became larger
                // it wants to be larger than it currently is
                // take some len from the other components
                amount_taken += next_info_height - info.height;
            } else if info.height > next_info_height {
                // when clamped, it became smaller
                // it wants to be smaller than it currently is
                // give some len to the other components
                amount_given += info.height - next_info_height;
            }
            info.height = next_info_height;
        }

        if amount_given >= amount_taken {
            let excess = amount_given - amount_taken;
            distribute_excess(&mut info, excess);
        } else {
            let deficit = amount_taken - amount_given;
            take_deficit(&mut info, deficit);
        }

        if self.elems.len() == 1 {
            let position = crate::widget::place(
                self.elems[0],
                event.position,
                crate::util::length::AspectRatioPreferredDirection::WidthFromHeight,
            )?;
            let mut sub_event = event.sub_event(position);
            sub_event.aspect_ratio_priority =
                crate::util::length::AspectRatioPreferredDirection::WidthFromHeight;
            self.elems[0].update(sub_event)?;
            return Ok(());
        }

        let mut sum_display_height = 0f32;
        for info in info.iter() {
            sum_display_height += info.height;
        }

        let vertical_space = if sum_display_height < event.position.h {
            let extra_space = event.position.h - sum_display_height;
            debug_assert!(!self.elems.is_empty());
            let num_spaces = self.elems.len() as u32 - 1;

            // store as float -> extremely important. or else a divide could
            // truncate spaces and lead to weird positions over several elements
            debug_assert!(num_spaces != 0);
            
            extra_space / num_spaces as f32
        } else {
            0.
        };

        let mut y_pos = if self.reverse {
            event.position.y + event.position.h
        } else {
            event.position.y
        };

        // the position given to each child is snapped to an integer grid.
        // in doing this, it rounds down. this accumulates an error over
        // many elements, which would cause the overall layout to not fill
        // its entire parent. to fix this, it distributes the error and
        // instead rounds up sometimes
        //
        // the elements to round up must be chosen in a good way:
        // - it's monotonic. a increase or decrease in the parent will give
        // the same or no change in each of the children
        // - children at the minimum are kept as is to prevent some jitter
        //   (but will be rounded up as a last resort)
        // - maximums are respected
        // - it distributes the round-ups in a semi even way
        let mut e_err_accumulation = 0.;
        let mut indices_not_at_min: Vec<usize> = Vec::new();
        let mut indices_at_min: Vec<usize> = Vec::new();
        for (i, info) in info.iter_mut().enumerate() {
            e_err_accumulation += info.height - info.height.floor();
            info.height = info.height.floor();
            if info.height <= info.min_vertical {
                indices_at_min.push(i);
            } else {
                indices_not_at_min.push(i);
            }
        }

        e_err_accumulation = e_err_accumulation.round();
        let mut e_err_accumulation = e_err_accumulation as usize;

        crate::util::shuffle::shuffle(&mut indices_not_at_min, 1234);
        crate::util::shuffle::shuffle(&mut indices_at_min, 5678);
        indices_not_at_min.extend(indices_at_min);
        let visit_indices = indices_not_at_min;

        for visit_index in visit_indices.iter() {
            let info = &mut info[*visit_index];
            if e_err_accumulation < 1 {
                break;
            }
            if info.height + 1. < info.max_vertical {
                info.height += 1.;
                e_err_accumulation -= 1;
            }
        }

        for (elem, info) in
            direction_conditional_iter_mut(&mut self.elems, self.reverse).zip(info.iter_mut())
        {
            if self.reverse {
                y_pos -= info.height;
                y_pos -= vertical_space;
            }

            // calculate the width, and maybe the width from the height
            let pre_clamp_width = info.preferred_horizontal.get(event.position.w);
            let mut width = clamp(pre_clamp_width, info.min_horizontal, info.max_horizontal);
            if let Some(new_w) = elem.preferred_width_from_height(info.height) {
                let new_w = new_w?;
                let new_w_max_clamp = if elem.preferred_link_allowed_exceed_portion() {
                    info.max_horizontal
                } else {
                    info.max_horizontal.strictest(MaxLen(pre_clamp_width))
                };
                width = clamp(new_w, info.min_horizontal, new_w_max_clamp);
            }

            let x = place(
                width,
                event.position.w,
                elem.min_w_fail_policy(),
                elem.max_w_fail_policy(),
            ) + event.position.x;

            let mut sub_event = event.sub_event(crate::util::rect::FRect {
                x,
                y: y_pos,
                w: width,
                h: info.height,
            });
            sub_event.aspect_ratio_priority =
                crate::util::length::AspectRatioPreferredDirection::WidthFromHeight;
            elem.update(sub_event)?;
            if !self.reverse {
                y_pos += info.height;
                y_pos += vertical_space;
            }
        }
        Ok(())
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        self.elems
            .iter_mut()
            .for_each(|e| e.update_adjust_position(pos_delta));
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        focus_manager: Option<&FocusManager>,
    ) -> Result<(), String> {
        for e in self.elems.iter_mut() {
            e.draw(canvas, focus_manager)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
#[derive(Default)]
struct ChildInfo {
    preferred_vertical: PreferredPortion,
    max_vertical: f32,
    min_vertical: f32,

    // iterated upon by the layout
    height: f32,

    preferred_horizontal: PreferredPortion,
    max_horizontal: MaxLen,
    min_horizontal: MinLen,
}


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
            if info.max_vertical < info.min_vertical {
                continue;
            }
            if info.height < info.max_vertical {
                available_weight += info.preferred_vertical.0;
            }
        }

        for info in info.iter_mut() {
            if info.max_vertical < info.min_vertical {
                continue;
            }
            if info.height < info.max_vertical {
                let ideal_amount_to_give = (info.preferred_vertical.0 / available_weight) * excess;
                let max_amount_to_give = info.max_vertical - info.height;
                if ideal_amount_to_give > max_amount_to_give {
                    info.height = info.max_vertical;
                    excess_from_excess += ideal_amount_to_give - max_amount_to_give;
                } else {
                    info.height += ideal_amount_to_give;
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
            if info.max_vertical < info.min_vertical {
                // I don't think this case can happen, but just in case
                continue;
            }
            if info.height > info.min_vertical {
                available_weight += info.preferred_vertical.0;
            }
        }

        for info in info.iter_mut() {
            if info.max_vertical < info.min_vertical {
                continue;
            }
            if info.height > info.min_vertical {
                let ideal_amount_to_take = (info.preferred_vertical.0 / available_weight) * deficit;
                let max_amount_to_take = info.height - info.min_vertical;
                if ideal_amount_to_take > max_amount_to_take {
                    info.height = info.min_vertical;
                    deficit_from_deficit += ideal_amount_to_take - max_amount_to_take;
                } else {
                    info.height -= ideal_amount_to_take;
                }
            }
        }
        deficit = deficit_from_deficit;
        if deficit == 0. {
            return;
        }
    }
}
