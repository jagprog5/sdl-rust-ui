use std::cell::Cell;
use std::u16;

use compact_str::CompactString;
use sdl2::{render::TextureCreator, video::WindowContext};

use crate::util::font::{SingleLineFontStyle, TextRenderProperties, SingleLineTextRenderType};
use crate::util::length::{
    AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen, MinLenFailPolicy, MinLenPolicy, PreferredPortion
};

use crate::widget::{
    texture::AspectRatioFailPolicy,
    widget::{Widget, WidgetEvent},
};

use super::texture::texture_draw_f;

/// caches the texture and what was used to create the texture
struct SingleLineLabelCache<'sdl> {
    pub text_rendered: CompactString,
    pub properties_rendered: TextRenderProperties,
    pub texture: sdl2::render::Texture<'sdl>,
}

/// caches size of the rendered text
struct SingleLineLabelSizeCacheData {
    /// if this changes the width needs to be recalculated
    pub point_size_used: u16,
    /// if this changes the width needs to be recalculated
    pub text_used: CompactString,
    /// the cached value
    pub size: (u32, u32),
}

/// caches the text width instead of recalculating every frame.
/// 
/// this cache is used for
/// - min or max when LenPolicy::Children is used
/// - preferred_length
struct SingleLineLabelSizeCache<'sdl> {
    pub cache: Option<SingleLineLabelSizeCacheData>,
    /// dup of the font_interface used by the Label, except this one is used for
    /// the min / max point size (since font interface caches based on point
    /// size, it makes sense to have a different cache for each)
    pub font_interface: Box<dyn SingleLineFontStyle<'sdl> + 'sdl>,
}

impl<'sdl> SingleLineLabelSizeCache<'sdl> {
    /// might take a copy of label_font_interface it this cache doesn't already have one
    pub fn get_size(&mut self, point_size: u16, text: &str) -> Result<(u32, u32), String> {
        let cache = match self
            .cache
            .take()
            .filter(|cache| cache.text_used == text && cache.point_size_used == point_size)
        {
            Some(cache) => cache, // cache is ok
            None => SingleLineLabelSizeCacheData {
                point_size_used: point_size,
                text_used: CompactString::from(text),
                size: self.font_interface.render_dimensions(text, point_size)?,
            },
        };

        Ok(self.cache.insert(cache).size)
    }
}

/// hides internals
pub struct SingleLineLabelSizeCachePub<'sdl> {
    cache: SingleLineLabelSizeCache<'sdl>,
}

impl<'sdl> SingleLineLabelSizeCachePub<'sdl> {
    /// might take a copy of label_font_interface it this cache doesn't already have one
    pub fn get_size(&mut self, point_size: u16, text: &str) -> Result<(u32, u32), String> {
        self.cache.get_size(point_size, text)
    }

    pub fn new(font_interface: Box<dyn SingleLineFontStyle<'sdl> + 'sdl>) -> Self {
        Self {
            cache: SingleLineLabelSizeCache {
                cache: None,
                font_interface,
            },
        }
    }
}

pub enum SingleLineLabelMinWidthPolicy<'sdl> {
    /// stated literally, ignoring the underlying text
    Literal(MinLen),
    /// infer from the minimum text height
    Infer(SingleLineLabelSizeCachePub<'sdl>),
}

impl<'sdl> SingleLineLabelMinWidthPolicy<'sdl> {
    pub fn get_size(&mut self, height: MinLen, text: &str) -> Result<(MinLen, MinLen), String> {
        let point_size: u16 = match (height.0 as u32).try_into() {
            Ok(v) => v,
            Err(_) => u16::MAX,
        };
        let r = match self {
            SingleLineLabelMinWidthPolicy::Literal(v) => (v.0, height.0),
            SingleLineLabelMinWidthPolicy::Infer(label_width_cache) => {
                let v = label_width_cache.get_size(point_size, text)?;
                (v.0 as f32, v.1 as f32)
            }
        };
        Ok((MinLen(r.0), MinLen(r.1)))
    }

    pub fn new(label: &SingleLineLabel<'sdl, '_>, policy: MinLenPolicy) -> Self {
        match policy {
            MinLenPolicy::Children => {
                SingleLineLabelMinWidthPolicy::Infer(SingleLineLabelSizeCachePub::new(label.font_interface.dup()))
            }
            MinLenPolicy::Literal(min_len) => SingleLineLabelMinWidthPolicy::Literal(min_len),
        }
    }
}

pub enum SingleLineLabelMaxWidthPolicy<'sdl> {
    /// stated literally, ignoring the underlying text
    Literal(MaxLen),
    /// infer from the maximum text height
    Infer(SingleLineLabelSizeCachePub<'sdl>),
}

impl<'sdl> SingleLineLabelMaxWidthPolicy<'sdl> {
    pub fn get_size(&mut self, height: MaxLen, text: &str) -> Result<(MaxLen, MaxLen), String> {
        let point_size: u16 = match (height.0 as u32).try_into() {
            Ok(v) => v,
            Err(_) => u16::MAX,
        };
        let r = match self {
            SingleLineLabelMaxWidthPolicy::Literal(v) => (v.0, height.0),
            SingleLineLabelMaxWidthPolicy::Infer(label_width_cache) => {
                let v = label_width_cache.get_size(point_size, text)?;
                (v.0 as f32, v.1 as f32)
            }
        };
        Ok((MaxLen(r.0), MaxLen(r.1)))
    }

    pub fn new(label: &SingleLineLabel<'sdl, '_>, policy: MaxLenPolicy) -> Self {
        match policy {
            MaxLenPolicy::Children => {
                SingleLineLabelMaxWidthPolicy::Infer(SingleLineLabelSizeCachePub::new(label.font_interface.dup()))
            }
            MaxLenPolicy::Literal(min_len) => SingleLineLabelMaxWidthPolicy::Literal(min_len),
        }
    }
}

pub trait SingleLineLabelState {
    /// produce a string from whatever data is being viewed
    fn get(&self) -> CompactString;
}

impl SingleLineLabelState for CompactString {
    fn get(&self) -> CompactString {
        self.clone()
    }
}

pub struct DefaultSingleLineLabelState {
    pub inner: Cell<CompactString>,
}

impl SingleLineLabelState for DefaultSingleLineLabelState {
    fn get(&self) -> CompactString {
        let temp_v = self.inner.take();
        let ret = temp_v.clone();
        self.inner.set(temp_v);
        ret
    }
}

/// a widget that contains a single line of text.
/// the font object and rendered font is cached - rendering only occurs when the
/// text / style or dimensions change
pub struct SingleLineLabel<'sdl, 'state> {
    pub text: &'state dyn SingleLineLabelState,
    pub text_properties: SingleLineTextRenderType,
    font_interface: Box<dyn SingleLineFontStyle<'sdl> + 'sdl>,

    pub aspect_ratio_fail_policy: AspectRatioFailPolicy,
    pub request_aspect_ratio: bool,

    pub min_w_fail_policy: MinLenFailPolicy,
    pub max_w_fail_policy: MaxLenFailPolicy,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,

    // a label does it's sizing by receiving a height, and deriving what the
    // corresponding width would be for that height
    pub min_h: MinLen,
    pub max_h: MaxLen,
    pub min_w: SingleLineLabelMinWidthPolicy<'sdl>,
    pub max_w: SingleLineLabelMaxWidthPolicy<'sdl>,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,

    creator: &'sdl TextureCreator<WindowContext>,
    cache: Option<SingleLineLabelCache<'sdl>>,
    ratio_cache: SingleLineLabelSizeCache<'sdl>,
}

impl<'sdl, 'state> SingleLineLabel<'sdl, 'state> {
    pub fn new(
        text: &'state dyn SingleLineLabelState,
        text_properties: SingleLineTextRenderType,
        font_interface: Box<dyn SingleLineFontStyle<'sdl> + 'sdl>,
        creator: &'sdl TextureCreator<WindowContext>,
    ) -> Self {
        let font_interface_dup_for_preferred_len = font_interface.dup();

        let mut r = Self {
            text,
            text_properties,
            font_interface,
            creator,
            request_aspect_ratio: true,
            cache: Default::default(),
            aspect_ratio_fail_policy: Default::default(),
            min_w_fail_policy: Default::default(),
            max_w_fail_policy: Default::default(),
            min_h_fail_policy: Default::default(),
            max_h_fail_policy: Default::default(),
            min_w: SingleLineLabelMinWidthPolicy::Literal(MinLen::LAX), // replace below
            max_w: SingleLineLabelMaxWidthPolicy::Literal(MaxLen::LAX),
            ratio_cache: SingleLineLabelSizeCache {
                cache: None,
                font_interface: font_interface_dup_for_preferred_len,
            },
            min_h: Default::default(),
            max_h: Default::default(),
            preferred_w: Default::default(),
            preferred_h: Default::default(),
        };
        let min_w = SingleLineLabelMinWidthPolicy::new(&r, MinLenPolicy::Children);
        let max_w = SingleLineLabelMaxWidthPolicy::new(&r, MaxLenPolicy::Children);
        r.min_w = min_w;
        r.max_w = max_w;
        r
    }
}

impl<'sdl, 'state> Widget for SingleLineLabel<'sdl, 'state> {
    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        let size = self.min_w.get_size(self.min_h, self.text.get().as_str())?;

        let ratio = size.0.0 as f32 / size.1.0 as f32;
        let min_w = AspectRatioPreferredDirection::width_from_height(ratio, self.min_h.0);
        Ok((MinLen(min_w), self.min_h))
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.min_w_fail_policy
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.min_h_fail_policy
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        let size = self.max_w.get_size(self.max_h, self.text.get().as_str())?;

        let ratio = size.0.0 as f32 / size.1.0 as f32;
        let max_w = AspectRatioPreferredDirection::width_from_height(ratio, self.max_h.0);
        Ok((MaxLen(max_w), self.max_h))
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_w_fail_policy
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.max_h_fail_policy
    }

    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.preferred_w, self.preferred_h)
    }

    fn preferred_width_from_height(
        &mut self,
        pref_h: f32,
    ) -> Option<Result<f32, String>> {
        if !self.request_aspect_ratio {
            return None;
        }
        let pref_size = match self.ratio_cache.get_size(u16::MAX, self.text.get().as_str()) {
            Ok(v) => v,
            Err(err) => return Some(Err(err)),
        };
        let ratio = pref_size.0 as f32 / pref_size.1 as f32;
        Some(Ok(AspectRatioPreferredDirection::width_from_height(
            ratio,
            pref_h,
        )))
    }

    fn preferred_height_from_width(
        &mut self,
        pref_w: f32,
    ) -> Option<Result<f32, String>> {
        if !self.request_aspect_ratio {
            return None;
        }
        let pref_size = match self.ratio_cache.get_size(u16::MAX, self.text.get().as_str()) {
            Ok(v) => v,
            Err(err) => return Some(Err(err)),
        };

        let ratio = pref_size.0 as f32 / pref_size.1 as f32;
        Some(Ok(AspectRatioPreferredDirection::height_from_width(
            ratio,
            pref_w,
        )))
    }

    fn draw(&mut self, event: WidgetEvent) -> Result<(), String> {
        let position = match event.position {
            Some(v) => v,
            None => return Ok(()), // no input handling
        };

        let point_size: u16 = match (position.height() as u32).try_into() {
            Ok(v) => v,
            Err(_) => u16::MAX,
        };

        let properties = TextRenderProperties {
            point_size,
            render_type: self.text_properties,
        };

        if let SingleLineTextRenderType::Shaded(_fg, bg) = properties.render_type {
            // more consistent; regardless of what the aspect ratio fail policy
            // (padding bars), give a background over the entirety of the label
            event.canvas.set_draw_color(bg);
            event.canvas.fill_frect(position)?;
        }

        let cache = match self.cache.take().filter(|cache| {
            cache.text_rendered == self.text.get().as_str() && cache.properties_rendered == properties
        }) {
            Some(cache) => cache,
            None => {
                // if the text of the render properties have changed, then the
                // text needs to be re-rendered
                let text = self.text.get();
                let texture =
                    self.font_interface
                        .render(text.as_str(), &properties, &self.creator)?;
                SingleLineLabelCache {
                    text_rendered: text,
                    texture,
                    properties_rendered: properties,
                }
            }
        };

        let txt = &cache.texture;
        texture_draw_f(
            txt,
            &self.aspect_ratio_fail_policy,
            event.canvas,
            None,
            position,
        )?;

        self.cache = Some(cache);
        Ok(())
    }
}