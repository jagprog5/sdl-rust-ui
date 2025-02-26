use sdl2::{pixels::Color, rect::Rect, render::TextureCreator, video::WindowContext};

use crate::util::{
    focus::FocusManager,
    font::MultiLineFontStyle,
    length::{MaxLenFailPolicy, MinLenFailPolicy, PreferredPortion},
    rect::rect_len_round, rust::CellRefOrCell,
};

use super::{Widget, WidgetUpdateEvent};

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
    /// allow the text to be drawn past the parent's boundary in a direction.
    /// indicate the direction
    AllowRunOff(MinLenFailPolicy),
    /// request an appropriate height, deduced from the width and text
    None(MinLenFailPolicy, MaxLenFailPolicy),
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
    pub text: CellRefOrCell<'state, String>,
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

    /// state stored for draw from update
    draw_pos: crate::util::rect::FRect,

    creator: &'sdl TextureCreator<WindowContext>,
    cache: Option<MultiLineLabelCache<'sdl>>,
}

impl<'sdl, 'state> MultiLineLabel<'sdl, 'state> {
    pub fn new(
        text: CellRefOrCell<'state, String>,
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
            draw_pos: Default::default(),
        }
    }
}

impl<'sdl, 'state> Widget for MultiLineLabel<'sdl, 'state> {
    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        (self.preferred_w, self.preferred_h)
    }

    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        match self.min_h_policy {
            MultiLineMinHeightFailPolicy::None(_, _) => true,
            _ => false,
        }
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        match self.min_h_policy {
            MultiLineMinHeightFailPolicy::None(min_len_fail_policy, _) => min_len_fail_policy,
            _ => Default::default(), // doesn't matter
        }
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        match self.min_h_policy {
            MultiLineMinHeightFailPolicy::None(_, max_len_fail_policy) => max_len_fail_policy,
            _ => Default::default(), // doesn't matter
        }
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        match self.min_h_policy {
            MultiLineMinHeightFailPolicy::None(_, _) => {
                // match logic from draw, so that the same cache is used
                let pref_w = match rect_len_round(pref_w) {
                    Some(v) => v,
                    None => return Some(Ok(0.)), // doesn't matter
                };
                let text = self.text.scope_take();
                // ok to use the same cache as draw, as once the pref_w is
                // figured out, then that same one is used at draw as well
                let cache = match self.cache.take().filter(|cache| {
                    cache.text_rendered == text.as_str()
                        && cache.color == self.color
                        && cache.point_size == self.point_size
                        && cache.wrap_width == pref_w
                }) {
                    Some(cache) => cache,
                    None => {
                        // if the text of the render properties have changed, then the
                        // text needs to be re-rendered
                        let texture = match self.font_interface.render(
                            text.as_str(),
                            self.color,
                            self.point_size,
                            pref_w,
                            self.creator,
                        ) {
                            Ok(v) => v,
                            Err(e) => return Some(Err(e)),
                        };
                        MultiLineLabelCache {
                            text_rendered: text.to_string(),
                            point_size: self.point_size,
                            wrap_width: pref_w,
                            color: self.color,
                            texture,
                        }
                    }
                };

                let txt = &cache.texture;

                let query = txt.query();

                self.cache = Some(cache);
                Some(Ok(query.height as f32))
            }
            _ => None,
        }
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

        let text = self.text.scope_take();

        let cache = match self.cache.take().filter(|cache| {
            cache.text_rendered == text.as_str()
                && cache.color == self.color
                && cache.point_size == self.point_size
                && cache.wrap_width == position.width()
        }) {
            Some(cache) => cache,
            None => {
                // if the text of the render properties have changed, then the
                // text needs to be re-rendered
                let texture = self.font_interface.render(
                    text.as_str(),
                    self.color,
                    self.point_size,
                    position.width(),
                    self.creator,
                )?;
                MultiLineLabelCache {
                    text_rendered: text.to_string(),
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
            canvas.copy(
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
                    canvas.copy(
                        txt,
                        Some(Rect::new(0, excess, query.width, position.height())),
                        Some(Rect::new(
                            position.x,
                            position.y,
                            query.width,
                            position.height(),
                        )),
                    )?
                }
                MultiLineMinHeightFailPolicy::AllowRunOff(v) => {
                    let excess = excess * (v.0 - 1.);
                    let excess = excess.round() as i32;
                    canvas.copy(
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
                MultiLineMinHeightFailPolicy::None(_, _) => {
                    canvas.copy(txt, None, self.draw_pos)?;
                }
            }
        }

        self.cache = Some(cache);
        Ok(())
    }
}
