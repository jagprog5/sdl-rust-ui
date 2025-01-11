use std::cell::Cell;

use compact_str::CompactString;
use sdl2::{keyboard::Keycode, render::TextureCreator, video::WindowContext};

use crate::util::{focus::{FocusID, FocusManager}, font::{SingleLineFontStyle, SingleLineTextRenderType, TextRenderProperties}, length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion}};

use super::{single_line_label::SingleLineLabelCache, widget::Widget};

pub trait SingleLineTextEditState {
    /// produce a string from whatever data is being viewed
    fn get(&self) -> CompactString;

    /// used to provide internal mutability on the contained data. could be used
    /// to append, etc
    fn set(&self, new: CompactString);
}

pub struct DefaultSingleLineTextEditState {
    pub inner: Cell<CompactString>,
}

impl SingleLineTextEditState for DefaultSingleLineTextEditState {
    fn get(&self) -> CompactString {
        let temp_v = self.inner.take();
        let ret = temp_v.clone();
        self.inner.set(temp_v);
        ret
    }
    
    fn set(&self, new: CompactString) {
        self.inner.set(new);
    }
}

/// contains a single line label which is editable
pub struct SingleLineTextInput<'sdl, 'state> {
    /// what happens when return key pressed
    pub functionality: Box<dyn FnMut() -> Result<(), String> + 'state>,

    pub focus_id: FocusID,

    pub text: &'state dyn SingleLineTextEditState,
    pub text_properties: SingleLineTextRenderType,
    font_interface: Box<dyn SingleLineFontStyle<'sdl> + 'sdl>,

    pub min_h: MinLen,
    pub max_h: MaxLen,
    pub min_h_fail_policy: MinLenFailPolicy,
    pub max_h_fail_policy: MaxLenFailPolicy,

    pub preferred_w: PreferredPortion,
    pub preferred_h: PreferredPortion,

    creator: &'sdl TextureCreator<WindowContext>,
    cache: Option<SingleLineLabelCache<'sdl>>,
}

impl<'sdl, 'state> SingleLineTextInput<'sdl, 'state> {
    pub fn new(
            functionality: Box<dyn FnMut() -> Result<(), String> + 'state>,
            focus_id: FocusID,
            text: &'state dyn SingleLineTextEditState,
            text_properties: SingleLineTextRenderType,
            font_interface: Box<dyn SingleLineFontStyle<'sdl> + 'sdl>,
            creator: &'sdl TextureCreator<WindowContext>,
        ) -> Self {
        Self {
            functionality,
            focus_id,
            text,
            text_properties,
            font_interface,
            creator,
            cache: None,
            min_h: Default::default(),
            max_h: Default::default(),
            preferred_w: Default::default(),
            preferred_h: Default::default(),
            min_h_fail_policy: Default::default(),
            max_h_fail_policy: Default::default(),
        }
    }
}

impl<'sdl, 'state> Widget for SingleLineTextInput<'sdl, 'state> {
    fn min(&mut self) -> Result<(crate::util::length::MinLen, crate::util::length::MinLen), String> {
        Ok((MinLen::LAX, self.min_h))
    }

    fn min_h_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        self.min_h_fail_policy
    }

    fn max(&mut self) -> Result<(crate::util::length::MaxLen, crate::util::length::MaxLen), String> {
        Ok((MaxLen::LAX, self.max_h))
    }

    fn max_h_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        self.max_h_fail_policy
    }

    fn preferred_portion(&self) -> (crate::util::length::PreferredPortion, crate::util::length::PreferredPortion) {
        (self.preferred_w, self.preferred_h)
    }

    fn update(&mut self, mut event: super::widget::WidgetEvent) -> Result<(), String> {
        FocusManager::default_widget_focus_behavior(self.focus_id, &mut event);

        // handle text input events (not to be confused with key down events)
        for sdl_event in event.events.iter_mut().filter(|event| event.available()) {
            match &sdl_event.e {
                // if enter key is released and this widget has focus then trigger the functionality
                sdl2::event::Event::KeyUp {
                    repeat: false,
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    if let Some(focus_manager) = &event.focus_manager {
                        if focus_manager.is_focused(self.focus_id) {
                            match (self.functionality)() {
                                Ok(()) => (),
                                Err(e) => return Err(e),
                            };
                            sdl_event.set_consumed();
                        }
                    }
                }
                // if backspace is pressed then pop the last character
                sdl2::event::Event::KeyDown { keycode: Some(Keycode::Backspace), ..} => {
                    if let Some(focus_manager) = &event.focus_manager {
                        if focus_manager.is_focused(self.focus_id) {
                            let mut content = self.text.get();
                            content.pop();
                            self.text.set(content);
                            sdl_event.set_consumed();
                        }
                    }
                }
                // if text is typed then append it to the text
                sdl2::event::Event::TextInput { text, .. } => {
                    if let Some(focus_manager) = &event.focus_manager {
                        if focus_manager.is_focused(self.focus_id) {
                            let mut content = self.text.get();
                            content += text;
                            self.text.set(content);
                        }
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }
    
    fn draw(&mut self, event: super::widget::WidgetEvent) -> Result<(), String> {
        let position: sdl2::rect::Rect = match event.position.into() {
            Some(v) => v,
            None => return Ok(()),
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
            event.canvas.fill_rect(position)?;
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

        // draw the texture to the position in such a way that only takes the
        // right most content that fits within the aspect ratio

        let query = txt.query();
        if query.height == 0 {
            return Ok(()); // guard div
        }

        let new_height = position.height() as f32;
        let scaled_height = new_height / query.height as f32;
        let new_width = query.width as f32 * scaled_height;

        let new_height = new_height as u32;
        // truncate, so it doesn't go one pixel off
        let new_width = new_width as u32;

        if new_width <= position.width() {
            // the text input's width is smaller than where it wants to be drawn
            // left align the content
            event.canvas.copy(txt, None, sdl2::rect::Rect::new(position.x, position.y, new_width, new_height))?;
        } else {
            // the text input's width is greater than where it wants to be drawn
            // cut off and only show the rightmost part of it
            event.canvas.copy(txt, sdl2::rect::Rect::new((new_width - position.width()) as i32, 0, query.width, query.height), position)?;
        }

        self.cache = Some(cache);

        Ok(())
    }
}