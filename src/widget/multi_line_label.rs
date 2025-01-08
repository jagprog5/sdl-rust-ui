use sdl2::{pixels::Color, rect::Rect, render::TextureCreator, video::WindowContext};

use crate::util::{
    font::MultiLineFontStyle,
    length::{MaxLenFailPolicy, MinLenFailPolicy, PreferredPortion},
};

use super::widget::{Widget, WidgetEvent};

pub trait MultiLineLabelState {
    /// produce a string from whatever data is being viewed
    fn get(&self) -> String;
}

impl MultiLineLabelState for String {
    fn get(&self) -> String {
        self.clone()
    }
}

struct MultiLineLabelCache<'sdl> {
    pub text_rendered: String,
    pub point_size: u16,
    pub wrap_width: u32,
    pub color: Color,
    pub texture: sdl2::render::Texture<'sdl>,
}

/// a multiline label's sizing is flexible - it can be any size. if the
/// width is too small, then it will wrap text. however, if the height is
/// too large, what should happen?
pub enum MultiLineMinHeightFailPolicy {
    /// cut off the text, to ensure it does not expand over the parent. contains
    /// a value from 0 to 1 inclusively, indicating if the text should be cut
    /// off from the negative or positive direction, respectively
    CutOff(f32),
    /// allow the text to be drawn downward past the parent's boundary
    AllowRunOff(MinLenFailPolicy),
}

impl Default for MultiLineMinHeightFailPolicy {
    fn default() -> Self {
        MultiLineMinHeightFailPolicy::AllowRunOff(MinLenFailPolicy::POSITIVE)
    }
}

/// a widget that contains multiline text.
/// the font object and rendered font is cached - rendering only occurs when the
/// text / style or dimensions change
pub struct MultiLineLabel<'sdl, 'state> {
    pub text: &'state dyn MultiLineLabelState,
    /// a single line label infers an appropriate point size from the available
    /// height. this doesn't make sense for multiline text, so it's instead
    /// stated literally
    pub point_size: u16,
    pub color: Color,

    font_interface: Box<dyn MultiLineFontStyle<'sdl> + 'sdl>,

    pub max_h_policy: MaxLenFailPolicy,
    pub min_h_policy: MultiLineMinHeightFailPolicy,

    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,

    creator: &'sdl TextureCreator<WindowContext>,
    cache: Option<MultiLineLabelCache<'sdl>>,
}

impl<'sdl, 'state> MultiLineLabel<'sdl, 'state> {
    pub fn new(
        text: &'state dyn MultiLineLabelState,
        point_size: u16,
        color: Color,
        font_interface: Box<dyn MultiLineFontStyle<'sdl> + 'sdl>,
        creator: &'sdl TextureCreator<WindowContext>,
    ) -> Self {
        Self {
            text,
            point_size,
            color,
            font_interface,
            preferred_w: Default::default(),
            preferred_h: Default::default(),
            creator,
            cache: Default::default(),
            min_h_policy: Default::default(),
            max_h_policy: Default::default(),
        }
    }
}

impl<'sdl, 'state> Widget for MultiLineLabel<'sdl, 'state> {
    fn draw(&mut self, event: WidgetEvent) -> Result<(), String> {
        let position: sdl2::rect::Rect = match event.position.into() {
            Some(v) => v,
            None => return Ok(()), // no input handling
        };

        let cache = match self.cache.take().filter(|cache| {
            cache.text_rendered == self.text.get().as_str()
                && cache.color == self.color
                && cache.point_size == self.point_size
                && cache.wrap_width == position.width()
        }) {
            Some(cache) => cache,
            None => {
                // if the text of the render properties have changed, then the
                // text needs to be re-rendered
                let text = self.text.get();
                let texture = self.font_interface.render(
                    text.as_str(),
                    self.color,
                    self.point_size,
                    position.width(),
                    &self.creator,
                )?;
                MultiLineLabelCache {
                    text_rendered: text,
                    point_size: self.point_size,
                    wrap_width: position.width(),
                    color: self.color,
                    texture,
                }
            }
        };

        let txt = &cache.texture;

        let query = txt.query();

        if query.height <= position.height() {
            let excess = position.height() - query.height;
            let excess = excess as f32;
            let excess = excess * self.max_h_policy.0;
            let excess = excess.round() as i32;
            event.canvas.copy(
                txt,
                None,
                Some(Rect::new(
                    position.x,
                    position.y + excess,
                    query.width,
                    query.height,
                )),
            )?;
        } else {
            let excess = query.height - position.height();
            let excess = excess as f32;
            match self.min_h_policy {
                MultiLineMinHeightFailPolicy::CutOff(v) => {
                    let excess = excess * (1. - v);
                    let excess = excess.round() as i32;
                    event.canvas.copy(
                        txt,
                        Some(Rect::new(0, excess, query.width, position.height())),
                        Some(Rect::new(position.x, position.y, query.width, position.height())),
                    )?
                }
                MultiLineMinHeightFailPolicy::AllowRunOff(v) => {
                    let excess = excess * (v.0 - 1.);
                    let excess = excess.round() as i32;
                    event.canvas.copy(
                        txt,
                        None,
                        Some(Rect::new(
                            position.x,
                            position.y + excess,
                            query.width,
                            query.height,
                        )),
                    )?;
                }
            }
        }

        self.cache = Some(cache);
        Ok(())
    }
}
