use std::ops::Not;

use sdl2::rect::{FRect, Rect};

use crate::util::length::{
    frect_to_rect, AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen,
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
        let position = match event.position {
            Some(v) => v,
            None => return Ok(()), // has no other functionality other than drawing
        };
        texture_draw(
            &self.texture,
            &self.aspect_ratio_fail_policy,
            event.canvas,
            None,
            frect_to_rect(position),
        )
    }
}

pub fn texture_draw(
    texture: &sdl2::render::Texture,
    aspect_ratio_policy: &AspectRatioFailPolicy,
    canvas: &mut sdl2::render::WindowCanvas,
    src: Option<sdl2::rect::Rect>,
    dst: sdl2::rect::Rect,
) -> Result<(), String> {
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

    match aspect_ratio_policy {
        AspectRatioFailPolicy::Stretch => canvas.copy(texture, None, Some(dst)),
        AspectRatioFailPolicy::ZoomOut((zoom_x, zoom_y)) => {
            let src_aspect_ratio = src_w as f32 / src_h as f32;
            let dst_aspect_ratio = dst.width() as f32 / dst.height() as f32;

            if src_aspect_ratio > dst_aspect_ratio {
                // padding at the top and bottom; scale down the size of the
                // src so the width matches the destination
                let scale_down = dst.width() as f32 / src_w as f32;
                let dst_width = (src_w as f32 * scale_down).round() as u32;
                let dst_height = (src_h as f32 * scale_down).round() as u32;
                if dst_width == 0 || dst_height == 0 {
                    return Ok(());
                }

                let dst_y_offset = ((dst.height() - dst_height) as f32 * zoom_y) as i32;
                canvas.copy(
                    texture,
                    Rect::new(src_x, src_y, src_w, src_h),
                    Some(Rect::new(
                        dst.x(),
                        dst.y() + dst_y_offset,
                        dst_width,
                        dst_height,
                    )),
                )
            } else {
                // padding at the left and right; scale down the size of the
                // src so the height matches the destination
                let scale_down = dst.height() as f32 / src_h as f32;
                let dst_width = (src_w as f32 * scale_down).round() as u32;
                let dst_height = (src_h as f32 * scale_down).round() as u32;
                if dst_width == 0 || dst_height == 0 {
                    return Ok(());
                }

                let dst_x_offset = ((dst.width() - dst_width) as f32 * zoom_x) as i32;
                canvas.copy(
                    texture,
                    Rect::new(src_x, src_y, src_w, src_h),
                    Some(Rect::new(
                        dst.x() + dst_x_offset,
                        dst.y(),
                        dst_width,
                        dst_height,
                    )),
                )
            }
        }
        AspectRatioFailPolicy::ZoomIn((zoom_x, zoom_y)) => {
            let src_aspect_ratio = src_w as f32 / src_h as f32;
            let dst_aspect_ratio = dst.width() as f32 / dst.height() as f32;

            if src_aspect_ratio > dst_aspect_ratio {
                let width =
                    ((dst.width() as f32 / dst.height() as f32) * src_h as f32).round() as u32;
                let x = (((src_w as i32 - width as i32) as f32) * zoom_x) as i32;
                canvas.copy(
                    texture,
                    Some(Rect::new(src_x + x, src_y, width, src_h)),
                    Some(dst),
                )
            } else {
                let height =
                    ((src_w as f32 / dst.width() as f32) * dst.height() as f32).round() as u32;
                let y = ((src_h as i32 - height as i32) as f32 * zoom_y) as i32;
                canvas.copy(
                    texture,
                    Some(Rect::new(src_x, src_y + y, src_w, height)),
                    Some(dst),
                )
            }
        }
    }
}

pub fn texture_draw_f(
    texture: &sdl2::render::Texture,
    aspect_ratio_policy: &AspectRatioFailPolicy,
    canvas: &mut sdl2::render::WindowCanvas,
    src: Option<sdl2::rect::Rect>,
    dst: sdl2::rect::FRect,
) -> Result<(), String> {
    let (src_x, src_y, src_w, src_h) = match src {
        None => {
            let query = texture.query();
            (0, 0, query.width, query.height)
        }
        Some(v) => (v.x(), v.y(), v.width(), v.height()),
    };
    let src_rect = Rect::new(src_x, src_y, src_w, src_h);
    let (src_x, src_y, src_w, src_h) = (src_x as f32, src_y as f32, src_w as f32, src_h as f32);

    if src_w == 0. || src_h == 0. {
        return Ok(()); // can't draw empty. also guards against div by 0
    }

    match aspect_ratio_policy {
        AspectRatioFailPolicy::Stretch => canvas.copy(texture, None, Some(frect_to_rect(dst))),
        AspectRatioFailPolicy::ZoomOut((zoom_x, zoom_y)) => {
            let src_aspect_ratio = src_w / src_h;
            let dst_aspect_ratio = dst.width() / dst.height();

            if src_aspect_ratio > dst_aspect_ratio {
                // padding at the top and bottom; scale down the size of the
                // src so the width matches the destination
                let scale_down = dst.width() / src_w;
                let dst_width = src_w * scale_down;
                let dst_height = src_h * scale_down;
                if dst_width == 0. || dst_height == 0. {
                    return Ok(());
                }

                let dst_y_offset = (dst.height() - dst_height) * zoom_y;
                canvas.copy_f(
                    texture,
                    src_rect,
                    Some(FRect::new(
                        dst.x(),
                        dst.y() + dst_y_offset,
                        dst_width,
                        dst_height,
                    )),
                )
            } else {
                // padding at the left and right; scale down the size of the
                // src so the height matches the destination
                let scale_down = dst.height() / src_h;
                let dst_width = src_w * scale_down;
                let dst_height = src_h * scale_down;
                if dst_width == 0. || dst_height == 0. {
                    return Ok(());
                }

                let dst_x_offset = (dst.width() - dst_width) as f32 * zoom_x;
                canvas.copy_f(
                    texture,
                    src_rect,
                    Some(FRect::new(
                        dst.x() + dst_x_offset,
                        dst.y(),
                        dst_width,
                        dst_height,
                    )),
                )
            }
        }
        AspectRatioFailPolicy::ZoomIn((zoom_x, zoom_y)) => {
            let src_aspect_ratio = src_w / src_h;
            let dst_aspect_ratio = dst.width() / dst.height();

            if src_aspect_ratio > dst_aspect_ratio {
                let width = (dst.width() / dst.height()) * src_h;
                let x = (src_w - width) * zoom_x;
                canvas.copy(
                    texture,
                    Some(frect_to_rect(FRect::new(src_x + x, src_y, width, src_h))),
                    Some(frect_to_rect(dst)),
                )
            } else {
                let height = (src_w / dst.width()) * dst.height();
                let y = (src_h - height) * zoom_y;
                canvas.copy_f(
                    texture,
                    Some(frect_to_rect(FRect::new(src_x, src_y + y, src_w, height))),
                    Some(dst),
                )
            }
        }
    }
}

