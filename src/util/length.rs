/// if a minimum length can't be respected, should excess length be pushed in the
/// positive or negative direction past the parent's boundary.
///
/// a minimum length has a higher priority compare to a maximum length
#[derive(Debug, Clone, Copy)]
pub struct MinLenFailPolicy(pub f32);

impl MinLenFailPolicy {
    /// expand excess in the positive direction
    pub const POSITIVE: MinLenFailPolicy = MinLenFailPolicy(1.);
    /// expand excess in the negative direction
    pub const NEGATIVE: MinLenFailPolicy = MinLenFailPolicy(0.);
    /// expand excess equally in positive and negative direction
    pub const CENTERED: MinLenFailPolicy = MinLenFailPolicy(0.5);
}

impl Default for MinLenFailPolicy {
    fn default() -> Self {
        MinLenFailPolicy::CENTERED
    }
}

/// if a maximum length can't be respected, where in the parent should this
/// length be placed. should it hug the positive or negative edge of the parent
#[derive(Debug, Clone, Copy)]
pub struct MaxLenFailPolicy(pub f32);

impl MaxLenFailPolicy {
    /// position in the most positive direction in the parent
    pub const POSITIVE: MaxLenFailPolicy = MaxLenFailPolicy(1.);
    /// position in the most negative direction in the parent
    pub const NEGATIVE: MaxLenFailPolicy = MaxLenFailPolicy(0.);
    /// position centered within the parent, with excess space given equally in
    /// the positive and negative direction
    pub const CENTERED: MaxLenFailPolicy = MaxLenFailPolicy(0.5);
}

impl Default for MaxLenFailPolicy {
    fn default() -> Self {
        MaxLenFailPolicy::CENTERED
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MinLen(pub f32);

impl From<f32> for MinLen {
    fn from(value: f32) -> Self {
        MinLen(value)
    }
}

impl MinLen {
    /// returns the strictest of two minimum lengths
    pub fn strictest(self, other: MinLen) -> MinLen {
        if self.0 > other.0 {
            self
        } else {
            other
        }
    }

    pub fn combined(self, other: MinLen) -> MinLen {
        MinLen(self.0 + other.0)
    }

    /// the least strict value possible
    pub const LAX: MinLen = MinLen(0.);
}

impl Default for MinLen {
    fn default() -> Self {
        MinLen::LAX
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MaxLen(pub f32);

impl From<f32> for MaxLen {
    fn from(value: f32) -> Self {
        MaxLen(value)
    }
}

impl MaxLen {
    /// returns the strictest of two maximum lengths
    pub fn strictest(self, other: MaxLen) -> MaxLen {
        if self.0 < other.0 {
            self
        } else {
            other
        }
    }

    pub fn combined(self, other: MaxLen) -> MaxLen {
        let v = if self.0 == f32::MAX || other.0 == f32::MAX {
            f32::MAX
        } else {
            self.0 + other.0
        };
        MaxLen(v)
    }

    /// the least strict value possible
    pub const LAX: MaxLen = MaxLen(f32::MAX);
}

impl Default for MaxLen {
    fn default() -> Self {
        MaxLen::LAX
    }
}

pub fn clamp(len: f32, min: MinLen, max: MaxLen) -> f32 {
    if min.0 >= max.0 {
        return min.0;
    }
    
    if len < min.0 {
        return min.0
    }
    
    if len > max.0 {
        return max.0;
    }

    len
}

pub fn place(
    len: f32,
    parent: f32,
    min_fail_policy: MinLenFailPolicy,
    max_fail_policy: MaxLenFailPolicy,
) -> f32 {
    if len < parent {
        return (parent - len) * max_fail_policy.0;
    }
    if len > parent {
        return (parent - len) * (1. - min_fail_policy.0);
    }
    return 0.;
}

/// what is the preferred portion of the parent's length that this length should
/// take up. in cases where multiple portions are competing, a weighted portion
/// is used (and as a convention, should add up to 1).
#[derive(Clone, Copy)]
pub struct PreferredPortion(pub f32);

impl From<f32> for PreferredPortion {
    fn from(value: f32) -> Self {
        PreferredPortion(value)
    }
}

impl PreferredPortion {
    pub const FULL: PreferredPortion = PreferredPortion(1.);
    pub const EMPTY: PreferredPortion = PreferredPortion(0.);

    /// this is a portion of the parent. given the parent, get the actual length to use
    pub fn get(&self, parent_len: f32) -> f32 {
        self.0 * parent_len
    }

    /// suppose multiple portions are sharing the same length. give the number
    /// of portions, the total portion together (ideally should add to 1 but
    /// doesn't have to), how long is this portion of the parent
    pub fn weighted_portion(
        &self,
        sum_portions: PreferredPortion,
        mut num_portions: usize,
        parent_len: f32,
    ) -> f32 {
        if num_portions == 0 {
            debug_assert!(false);
            num_portions = 1;
        }

        let p = if sum_portions.0 == 0. {
            // entirely possible that each component is zero preferred portion.
            // this is handled by giving each portion an equal amount
            PreferredPortion(1. / num_portions as f32)
        } else {
            PreferredPortion(self.0 / sum_portions.0)
        };

        p.get(parent_len)
    }
}

impl Default for PreferredPortion {
    fn default() -> Self {
        PreferredPortion::FULL
    }
}

#[derive(Clone, Copy)]
pub enum MinLenPolicy {
    /// inherit the dimensions of the contained thing
    Children,
    /// min len is plainly stated, ignoring the underlying thing's dimensions
    Literal(MinLen),
}

impl Default for MinLenPolicy {
    fn default() -> Self {
        MinLenPolicy::Children
    }
}

#[derive(Clone, Copy)]
pub enum MaxLenPolicy {
    /// inherit the dimensions of the contained thing
    Children,
    /// max len is plainly stated, ignoring the underlying thing's dimensions
    Literal(MaxLen),
}

impl Default for MaxLenPolicy {
    fn default() -> Self {
        MaxLenPolicy::Children
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AspectRatioPreferredDirection {
    WidthFromHeight,
    HeightFromWidth,
}

impl Default for AspectRatioPreferredDirection {
    fn default() -> Self {
        AspectRatioPreferredDirection::WidthFromHeight
    }
}

impl AspectRatioPreferredDirection {
    pub fn width_from_height(ratio: f32, h: f32) -> f32 {
        h * ratio
    }

    pub fn height_from_width(ratio: f32, w: f32) -> f32 {
        w / ratio
    }
}

/// positioning and sizing stays as floats, until just before rendering /
/// updating, only then is it converted to integers
pub fn frect_to_rect(rect: sdl2::rect::FRect) -> sdl2::rect::Rect {
    sdl2::rect::Rect::new(rect.x().round() as i32, rect.y().round() as i32, rect.width().round() as u32, rect.height().round() as u32)
}
