use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

#[cfg(feature = "sdl2-ttf")]
use sdl2::{
    pixels::Color,
    render::TextureCreator,
    rwops::RWops,
    surface::Surface,
    ttf::{Font, Sdl2TtfContext},
    video::WindowContext,
};
#[cfg(feature = "sdl2-ttf")]
use weak_table::WeakValueHashMap;

/// manages a font. use this to get a font object with a certain point size
#[cfg(feature = "sdl2-ttf")]
pub struct FontManager<'sdl> {
    ttf_context: &'sdl Sdl2TtfContext,
    /// refs ttf data
    font_data: &'sdl [u8],
    /// associates point size with the font
    fonts: WeakValueHashMap<u16, Weak<Font<'sdl, 'sdl>>>,
}

#[cfg(feature = "sdl2-ttf")]
impl<'sdl> FontManager<'sdl> {
    /// font_data is the contents of a ttf file read to the end
    pub fn new(ttf_context: &'sdl Sdl2TtfContext, font_data: &'sdl [u8]) -> Self {
        Self {
            ttf_context,
            font_data,
            fonts: Default::default(),
        }
    }
}

#[cfg(feature = "sdl2-ttf")]
impl<'sdl> FontManager<'sdl> {
    pub fn get(&mut self, point_size: u16) -> Result<Rc<Font<'sdl, 'sdl>>, String> {
        match self.fonts.get(&point_size) {
            Some(v) => return Ok(v),
            None => {}
        };

        let rwops = RWops::from_bytes(&self.font_data)?;
        let font = Rc::new(self.ttf_context.load_font_from_rwops(rwops, point_size)?);
        self.fonts.insert(point_size, font.clone());
        Ok(font)
    }
}

// =============================================================================

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SingleLineTextRenderType {
    #[deprecated(note="looks like sh**")]
    Solid(Color),
    /// foreground, background, respectively
    Shaded(Color, Color),
    Blended(Color),
}

impl Default for SingleLineTextRenderType {
    fn default() -> Self {
        SingleLineTextRenderType::Blended(Color::WHITE)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TextRenderProperties {
    pub point_size: u16,
    pub render_type: SingleLineTextRenderType,
}

// =============================================================================

/// tells the gui how to render text
pub trait SingleLineFontStyle<'sdl> {
    /// render text - produce a texture
    ///
    /// it is likely that subsequent calls to this FontStyle instance will
    /// request the same point size (including between member functions); the
    /// most recent font object for that point size should be cached  
    /// caller note: if this isn't the case, then this object should be cloned
    /// for each instance where a different point size is used
    ///
    /// shouldn't give err on empty text input (just give background texture)
    fn render(
        &mut self,
        text: &str,
        properties: &TextRenderProperties,
        texture_creator: &'sdl TextureCreator<WindowContext>,
    ) -> Result<sdl2::render::Texture<'sdl>, String>;

    /// get the width, height of some text if it were to be rendered
    ///
    /// all of the doc string for render applies here as well
    fn render_dimensions(&mut self, text: &str, point_size: u16) -> Result<(u32, u32), String>;

    /// object safe clone
    fn dup(&self) -> Box<dyn SingleLineFontStyle<'sdl> + 'sdl>;
}

/// tells the gui how to render text
pub trait MultiLineFontStyle<'sdl> {
    /// render wrapped text
    ///
    /// the doc string for SingleLineFontStyle::render applies here as well
    fn render(
        &mut self,
        text: &str,
        color: Color,
        point_size: u16,
        wrap_width: u32,
        texture_creator: &'sdl TextureCreator<WindowContext>,
    ) -> Result<sdl2::render::Texture<'sdl>, String>;
}

#[cfg(feature = "sdl2-ttf")]
#[derive(Clone)]
struct TextRendererFontCache<'sdl> {
    /// the cached object
    pub font: Rc<Font<'sdl, 'sdl>>,
    /// if this changes, a new font is needed
    pub font_point_size: u16,
}

#[cfg(feature = "sdl2-ttf")]
#[derive(Clone)]
pub struct TextRenderer<'sdl> {
    font_manager: &'sdl Cell<Option<FontManager<'sdl>>>,
    cache: Option<TextRendererFontCache<'sdl>>,
}

#[cfg(feature = "sdl2-ttf")]
impl<'sdl> TextRenderer<'sdl> {
    pub fn new(font_manager: &'sdl Cell<Option<FontManager<'sdl>>>) -> Self {
        Self {
            font_manager,
            cache: None,
        }
    }
}

#[cfg(feature = "sdl2-ttf")]
impl<'sdl> SingleLineFontStyle<'sdl> for TextRenderer<'sdl> {
    fn render(
        &mut self,
        text: &str,
        properties: &TextRenderProperties,
        texture_creator: &'sdl TextureCreator<WindowContext>,
    ) -> Result<sdl2::render::Texture<'sdl>, String> {
        let surface = if text.len() == 0 {
            // handle SdlError("Text has zero width")
            // create a 1x1 replacement
            let mut surface = Surface::new(1, 1, sdl2::pixels::PixelFormatEnum::ARGB8888)
                .map_err(|e| e.to_string())?;
            surface.with_lock_mut(|buffer| match properties.render_type {
                SingleLineTextRenderType::Shaded(_, background) => {
                    buffer[3] = background.a;
                    buffer[2] = background.r;
                    buffer[1] = background.g;
                    buffer[0] = background.b;
                }
                _ => {
                    buffer[0] = 0;
                    buffer[1] = 0;
                    buffer[2] = 0;
                    buffer[3] = 0;
                }
            });
            surface
        } else {
            let font = match self
                .cache
                .take()
                .filter(|cache| cache.font_point_size == properties.point_size)
            {
                Some(cache) => &self.cache.insert(cache).font,
                None => {
                    let mut maybe_manager = self.font_manager.take();
                    let manager = match maybe_manager.as_mut() {
                        Some(v) => v,
                        // should never error, as it will always be returned to the cell
                        None => return Err("couldn't reference font manager".to_owned()),
                    };
                    let maybe_r = manager.get(properties.point_size);
                    self.font_manager.set(maybe_manager);
                    let r = maybe_r?;
                    &self
                        .cache
                        .insert(TextRendererFontCache {
                            font: r.clone(),
                            font_point_size: properties.point_size,
                        })
                        .font
                }
            };

            let partial_render = font.render(text);
            let surface = match properties.render_type {
                #[allow(deprecated)]
                SingleLineTextRenderType::Solid(color) => partial_render.solid(color),
                SingleLineTextRenderType::Shaded(color, background) => {
                    partial_render.shaded(color, background)
                }
                SingleLineTextRenderType::Blended(color) => partial_render.blended(color),
            }
            .map_err(|e| e.to_string())?;
            surface
        };

        let mut texture = texture_creator
            .create_texture_from_surface(surface)
            .map_err(|e| e.to_string())?;

        // I made this binding :)
        texture.set_scale_mode(sdl2::render::ScaleMode::Linear);

        Ok(texture)
    }

    fn render_dimensions(&mut self, text: &str, point_size: u16) -> Result<(u32, u32), String> {
        let font = match self
            .cache
            .take()
            .filter(|cache| cache.font_point_size == point_size)
        {
            Some(cache) => &self.cache.insert(cache).font,
            None => {
                let mut maybe_manager = self.font_manager.take();
                let manager = match maybe_manager.as_mut() {
                    Some(v) => v,
                    // should never error, as it will always be returned to the cell
                    None => return Err("couldn't reference font manager".to_owned()),
                };
                let maybe_r = manager.get(point_size);
                self.font_manager.set(maybe_manager);
                let r = maybe_r?;
                &self
                    .cache
                    .insert(TextRendererFontCache {
                        font: r.clone(),
                        font_point_size: point_size,
                    })
                    .font
            }
        };

        let (w, h) = font.size_of(text).map_err(|e| e.to_string())?;
        Ok((w, h))
    }

    fn dup(&self) -> Box<dyn SingleLineFontStyle<'sdl> + 'sdl> {
        Box::new(TextRenderer {
            font_manager: self.font_manager,
            cache: None,
        })
    }
}

#[cfg(feature = "sdl2-ttf")]
impl<'sdl> MultiLineFontStyle<'sdl> for TextRenderer<'sdl> {
    fn render(
        &mut self,
        text: &str,
        color: Color,
        point_size: u16,
        wrap_width: u32,
        texture_creator: &'sdl TextureCreator<WindowContext>,
    ) -> Result<sdl2::render::Texture<'sdl>, String> {
        // closely follows SingleLineFontStyle::render implementation
        let surface = if text.len() == 0 {
            // handle SdlError("Text has zero width")
            // create a 1x1 replacement
            let mut surface = Surface::new(1, 1, sdl2::pixels::PixelFormatEnum::ARGB8888)
                .map_err(|e| e.to_string())?;
            surface.with_lock_mut(|buffer| {
                buffer[0] = 0;
                buffer[1] = 0;
                buffer[2] = 0;
                buffer[3] = 0;
            });
            surface
        } else {
            let font = match self
                .cache
                .take()
                .filter(|cache| cache.font_point_size == point_size)
            {
                Some(cache) => &self.cache.insert(cache).font,
                None => {
                    let mut maybe_manager = self.font_manager.take();
                    let manager = match maybe_manager.as_mut() {
                        Some(v) => v,
                        // should never error, as it will always be returned to the cell
                        None => return Err("couldn't reference font manager".to_owned()),
                    };
                    let maybe_r = manager.get(point_size);
                    self.font_manager.set(maybe_manager);
                    let r = maybe_r?;
                    &self
                        .cache
                        .insert(TextRendererFontCache {
                            font: r.clone(),
                            font_point_size: point_size,
                        })
                        .font
                }
            };

            let partial_render = font.render(text);
            let surface = partial_render
                .blended_wrapped(color, wrap_width)
                .map_err(|e| e.to_string())?;
            surface
        };
        let mut texture = texture_creator
            .create_texture_from_surface(surface)
            .map_err(|e| e.to_string())?;
        texture.set_scale_mode(sdl2::render::ScaleMode::Linear);
        Ok(texture)
    }
}
