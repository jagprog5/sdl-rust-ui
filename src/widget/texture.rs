use std::ops::Not;

use crate::util::length::{
    AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen,
    MinLenFailPolicy, MinLenPolicy, PreferredPortion,
};

use super::widget::{Widget, WidgetEvent};

/// how should an image's aspect ratio be treated if the available space does
/// not have the same ratio
pub enum AspectRatioFailPolicy {
    /// simply stretch the image to fit the available space, ignoring the aspect
    /// ratio
    Stretch,

    /// zoom out, adding blank space.
    ///
    /// contains two floats from 0-1 (inclusive), where 0 aligns the image in
    /// the negative direction (x, y respectively), and 1 aligns the image in
    /// the positive direction.
    ///
    /// a sane default is (0.5, 0.5)
    ZoomOut((f32, f32)),

    /// zoom in, cutting off excess length
    ///
    /// contains two floats from 0-1 (inclusive) where 0 aligns the image in the
    /// negative direction (x, y respectively), and 1 aligns the image in the
    /// positive direction.
    ///
    /// a sane default is (0.5, 0.5)
    ZoomIn((f32, f32)),
}

impl Default for AspectRatioFailPolicy {
    fn default() -> Self {
        AspectRatioFailPolicy::ZoomOut((0.5, 0.5))
    }
}

/// widget for a static sdl2 texture
pub struct Texture<'sdl> {
    pub texture: &'sdl sdl2::render::Texture<'sdl>,
    /// none means use the entire texture
    pub texture_src: Option<sdl2::rect::Rect>,

    /// how should the texture be stretched / sized if the aspect ratio is not
    /// respected
    pub aspect_ratio_fail_policy: AspectRatioFailPolicy,

    pub request_aspect_ratio: bool,

    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,
    pub min_w_policy: MinLenPolicy,
    pub max_w_policy: MaxLenPolicy,
    pub min_h_policy: MinLenPolicy,
    pub max_h_policy: MaxLenPolicy,
    pub pref_w: PreferredPortion,
    pub pref_h: PreferredPortion,
    pub preferred_link_allowed_exceed_portion: bool,
}

impl<'sdl> Texture<'sdl> {
    pub fn new(texture: &'sdl sdl2::render::Texture<'sdl>) -> Texture<'sdl> {
        Texture {
            texture: texture,
            texture_src: Default::default(),
            aspect_ratio_fail_policy: Default::default(),
            request_aspect_ratio: true,
            min_w_fail_policy: Default::default(),
            max_w_fail_policy: Default::default(),
            min_h_fail_policy: Default::default(),
            max_h_fail_policy: Default::default(),
            min_w_policy: Default::default(),
            max_w_policy: Default::default(),
            min_h_policy: Default::default(),
            max_h_policy: Default::default(),
            pref_w: Default::default(),
            pref_h: Default::default(),
            preferred_link_allowed_exceed_portion: Default::default(),
        }
    }
}

impl<'sdl> Widget for Texture<'sdl> {
    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.preferred_link_allowed_exceed_portion
    }

    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        if let MinLenPolicy::Literal(w) = self.min_w_policy {
            if let MinLenPolicy::Literal(h) = self.min_h_policy {
                return Ok((w, h)); // no need to query texture
            }
        }

        // texture querying is fast. just does a struct lookup
        let query = self.texture.query();
        Ok((
            match self.min_w_policy {
                MinLenPolicy::Children => MinLen(query.width as f32),
                MinLenPolicy::Literal(min_len) => min_len,
            },
            match self.min_h_policy {
                MinLenPolicy::Children => MinLen(query.height as f32),
                MinLenPolicy::Literal(min_len) => min_len,
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
        if let MaxLenPolicy::Literal(w) = self.max_w_policy {
            if let MaxLenPolicy::Literal(h) = self.max_h_policy {
                return Ok((w, h)); // no need to query texture
            }
        }

        // texture querying is fast. just does a struct lookup
        let query = self.texture.query();
        Ok((
            match self.max_w_policy {
                MaxLenPolicy::Children => MaxLen(query.width as f32),
                MaxLenPolicy::Literal(max_len) => max_len,
            },
            match self.max_h_policy {
                MaxLenPolicy::Children => MaxLen(query.height as f32),
                MaxLenPolicy::Literal(max_len) => max_len,
            },
        ))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_w_fail_policy
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_h_fail_policy
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.pref_w, self.pref_h)
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        if self.request_aspect_ratio.not() {
            return None;
        }
        
        let q = self.texture.query();
        let ratio = q.width as f32 / q.height as f32;
        Some(Ok(AspectRatioPreferredDirection::width_from_height(
            ratio, pref_h,
        )))
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        if self.request_aspect_ratio.not() {
            return None;
        }

        let q = self.texture.query();
        let ratio = q.width as f32 / q.height as f32;

        Some(Ok(AspectRatioPreferredDirection::height_from_width(
            ratio, pref_w,
        )))
    }

    fn draw(&mut self, event: WidgetEvent) -> Result<(), String> {
        texture_draw(
            self.texture,
            &self.aspect_ratio_fail_policy,
            event.canvas,
            self.texture_src,
            event.position,
        )
    }
}

pub(crate) fn texture_draw(
    texture: &sdl2::render::Texture,
    aspect_ratio_fail_policy: &AspectRatioFailPolicy,
    canvas: &mut sdl2::render::WindowCanvas,
    src: Option<sdl2::rect::Rect>,
    dst: crate::util::rect::FRect,
) -> Result<(), String> {
    // dst is kept as float form until just before canvas copy. needed or else
    // it is jumpy

    let (src_x, src_y, src_w, src_h) = match src {
        None => {
            let query = texture.query();
            (0, 0, query.width, query.height)
        }
        Some(v) => (v.x(), v.y(), v.width(), v.height()),
    };

    if src_w == 0 || src_h == 0 {
        return Ok(()); // can't draw empty. also guards against div by 0
    }

    match aspect_ratio_fail_policy {
        AspectRatioFailPolicy::Stretch => {
            let dst: sdl2::rect::Rect = match dst.into() {
                None => return Ok(()), // can't draw zero size
                Some(v) => v,
            };
            canvas.copy(texture, src, Some(dst))
        }
        AspectRatioFailPolicy::ZoomOut((zoom_x, zoom_y)) => {
            let src_w = src_w as f32;
            let src_h = src_h as f32;
            let src_aspect_ratio = src_w / src_h; // div guarded above
            if dst.h == 0. {
                return Ok(()); // guard div + can't drawn zero area texture
            }
            let dst_aspect_ratio = dst.w / dst.h;

            if src_aspect_ratio > dst_aspect_ratio {
                // padding at the top and bottom; scale down the size of the
                // src so the width matches the destination
                let scale_down = dst.w / src_w; // div guarded above
                let dst_width = (src_w * scale_down).round() as u32;
                let dst_height = (src_h * scale_down).round() as u32;
                if dst_width == 0 || dst_height == 0 {
                    return Ok(()); // zoomed out too much
                }

                let dst_y_offset = ((dst.h - dst_height as f32) * zoom_y).round() as i32;
                canvas.copy(
                    texture,
                    src,
                    Some(sdl2::rect::Rect::new(
                        dst.x.round() as i32,
                        dst.y.round() as i32 + dst_y_offset,
                        dst_width,
                        dst_height,
                    )),
                )
            } else {
                // padding at the left and right; scale down the size of the
                // src so the height matches the destination
                let scale_down = dst.h / src_h; // div guarded above
                let dst_width = (src_w * scale_down).round() as u32;
                let dst_height = (src_h * scale_down).round() as u32;
                if dst_width == 0 || dst_height == 0 {
                    return Ok(()); // zoomed out too much
                }

                let dst_x_offset = ((dst.w - dst_width as f32) * zoom_x) as i32;
                canvas.copy(
                    texture,
                    src,
                    Some(sdl2::rect::Rect::new(
                        dst.x.round() as i32 + dst_x_offset,
                        dst.y.round() as i32,
                        dst_width,
                        dst_height,
                    )),
                )
            }
        }
        AspectRatioFailPolicy::ZoomIn((zoom_x, zoom_y)) => {
            let dst_sdl2: sdl2::rect::Rect = match dst.into() {
                None => return Ok(()), // can't draw zero size
                Some(v) => v,
            };

            let src_w_f = src_w as f32;
            let src_h_f = src_h as f32;

            let src_aspect_ratio = src_w_f / src_h_f; // guarded above
            let dst_aspect_ratio = dst.w / dst.h; // guarded above by dst_sdl2 into

            if src_aspect_ratio > dst_aspect_ratio {
                let width = (dst_aspect_ratio * src_h_f).round() as u32;
                if width == 0 {
                    return Ok(()); // too extreme of a ratio
                }
                let x = ((src_w_f - width as f32) * zoom_x) as i32;
                canvas.copy(
                    texture,
                    Some(sdl2::rect::Rect::new(src_x + x, src_y, width, src_h)),
                    Some(dst_sdl2),
                )
            } else {
                //                     V guarded above by dst_sdl2 into
                let height = ((src_w_f / dst.w) * dst.h).round() as u32;
                if height == 0 {
                    return Ok(()); // too extreme of a ratio
                }
                let y = ((src_h_f - height as f32) * zoom_y) as i32;
                canvas.copy(
                    texture,
                    Some(sdl2::rect::Rect::new(src_x, src_y + y, src_w, height)),
                    Some(dst_sdl2),
                )
            }
        }
    }
}
