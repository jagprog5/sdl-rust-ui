use sdl2::{render::TextureCreator, video::WindowContext};

use crate::util::focus::FocusManager;
use crate::util::font::{SingleLineFontStyle, SingleLineTextRenderType, TextRenderProperties};
use crate::util::length::{
    AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen,
    MinLenFailPolicy, MinLenPolicy, PreferredPortion,
};

use crate::util::rust::CellRefOrCell;
use crate::widget::texture::AspectRatioFailPolicy;

use super::texture::texture_draw;
use super::{Widget, WidgetUpdateEvent};

/// caches the texture and what was used to create the texture
pub(crate) struct SingleLineLabelCache<'sdl> {
    pub text_rendered: String,
    pub properties_rendered: TextRenderProperties,
    pub texture: sdl2::render::Texture<'sdl>,
}

/// caches size of the rendered text
pub(crate) struct SingleLineLabelSizeCacheData {
    /// if this changes the width needs to be recalculated
    pub point_size_used: u16,
    /// if this changes the width needs to be recalculated
    pub text_used: String,
    /// the cached value
    pub size: (u32, u32),
}

/// caches the text width instead of recalculating every frame.
///
/// this cache is used for
/// - min or max when LenPolicy::Children is used
/// - preferred_length
pub(crate) struct SingleLineLabelSizeCache<'sdl> {
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
                text_used: text.to_owned(),
                size: self.font_interface.render_dimensions(text, point_size)?,
            },
        };

        Ok(self.cache.insert(cache).size)
    }
}

/// a widget that contains a single line of text.
/// the font object and rendered font is cached - rendering only occurs when the
/// text / style or dimensions change
pub struct SingleLineLabel<'sdl, 'state> {
    pub text: CellRefOrCell<'state, String>,
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
    pub min_w_policy: MinLenPolicy,
    pub max_w_policy: MaxLenPolicy,
    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,

    creator: &'sdl TextureCreator<WindowContext>,
    cache: Option<SingleLineLabelCache<'sdl>>,
    ratio_cache: SingleLineLabelSizeCache<'sdl>,

    /// state stored for draw from update
    draw_pos: crate::util::rect::FRect,
}

impl<'sdl, 'state> SingleLineLabel<'sdl, 'state> {
    pub fn new(
        text: CellRefOrCell<'state, String>,
        text_properties: SingleLineTextRenderType,
        font_interface: Box<dyn SingleLineFontStyle<'sdl> + 'sdl>,
        creator: &'sdl TextureCreator<WindowContext>,
    ) -> Self {
        let font_interface_dup_for_preferred_len = font_interface.dup();
        Self {
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
            min_w_policy: Default::default(),
            max_w_policy: Default::default(),
            ratio_cache: SingleLineLabelSizeCache {
                cache: None,
                font_interface: font_interface_dup_for_preferred_len,
            },
            min_h: Default::default(),
            max_h: Default::default(),
            preferred_w: Default::default(),
            preferred_h: Default::default(),
            draw_pos: Default::default(),
        }
    }
}

impl<'sdl, 'state> Widget for SingleLineLabel<'sdl, 'state> {
    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        let text = self.text.scope_take();
        let size = self.ratio_cache.get_size(u16::MAX, text.as_str())?;
        let ratio = size.0 as f32 / size.1 as f32;
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
        let text = self.text.take();
        let size = match self.ratio_cache.get_size(u16::MAX, text.as_str()) {
            Ok(size) => size,
            Err(err) => {
                self.text.set(text);
                return Err(err);
            }
        };
        self.text.set(text);
        let ratio = size.0 as f32 / size.1 as f32;
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

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        if !self.request_aspect_ratio {
            return None;
        }
        let text = self.text.scope_take();
        let pref_size = match self
            .ratio_cache
            .get_size(u16::MAX, text.as_str())
        {
            Ok(v) => v,
            Err(err) => return Some(Err(err)),
        };
        let ratio = pref_size.0 as f32 / pref_size.1 as f32;
        Some(Ok(AspectRatioPreferredDirection::width_from_height(
            ratio, pref_h,
        )))
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        if !self.request_aspect_ratio {
            return None;
        }
        let text = self.text.scope_take();
        let pref_size = match self
            .ratio_cache
            .get_size(u16::MAX, text.as_str())
        {
            Ok(v) => v,
            Err(err) => return Some(Err(err)),
        };

        let ratio = pref_size.0 as f32 / pref_size.1 as f32;
        Some(Ok(AspectRatioPreferredDirection::height_from_width(
            ratio, pref_w,
        )))
    }

    fn update(&mut self, event: WidgetUpdateEvent) -> Result<(), String> {
        self.draw_pos = event.position;
        Ok(())
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        self.draw_pos.x += pos_delta.0 as f32;
        self.draw_pos.y += pos_delta.1 as f32;
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        _focus_manager: &FocusManager,
    ) -> Result<(), String> {
        let position: sdl2::rect::Rect = match self.draw_pos.into() {
            Some(v) => v,
            None => return Ok(()), // no input handling
        };

        // the point size to render isn't just the height. it's also influenced by the aspect ratio as it get crammed into the available space

        let height_option_1 = position.height();

        let text = self.text.scope_take();
        let height_option_2 = {
            let pref_size = match self
                .ratio_cache
                .get_size(u16::MAX, text.as_str())
            {
                Ok(v) => v,
                Err(err) => return Err(err),
            };
            let ratio = pref_size.0 as f32 / pref_size.1 as f32;
            let height_from_width =
                AspectRatioPreferredDirection::height_from_width(ratio, position.width() as f32);
            height_from_width.ceil() as u32
        };

        let height_to_use = height_option_1.min(height_option_2);

        let point_size: u16 = match height_to_use.try_into() {
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
            canvas.set_draw_color(bg);
            canvas.fill_rect(position)?;
        }

        let cache = match self.cache.take().filter(|cache| {
            cache.text_rendered == text.as_str()
                && cache.properties_rendered == properties
        }) {
            Some(cache) => cache,
            None => {
                // if the text of the render properties have changed, then the
                // text needs to be re-rendered
                let texture =
                    self.font_interface
                        .render(text.as_str(), &properties, self.creator)?;
                SingleLineLabelCache {
                    text_rendered: text.to_string(),
                    texture,
                    properties_rendered: properties,
                }
            }
        };

        let txt = &cache.texture;
        let r = texture_draw(
            txt,
            &self.aspect_ratio_fail_policy,
            canvas,
            None,
            self.draw_pos,
        );

        self.cache = Some(cache);
        r?;

        Ok(())
    }
}
