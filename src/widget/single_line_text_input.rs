use std::cell::Cell;

use compact_str::CompactString;
use sdl2::{
    keyboard::{Keycode, Mod},
    pixels::{Color, PixelFormatEnum},
    rect::Point,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};

use crate::util::{
    focus::{FocusManager, RefCircularUIDCell, WidgetEventFocusSubset},
    font::{SingleLineFontStyle, SingleLineTextRenderType, TextRenderProperties},
    length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion},
};

use super::{single_line_label::SingleLineLabelCache, Widget, WidgetUpdateEvent};

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

pub trait SingleLineTextEditStyle {
    /// The texture will be redrawn only if the target dimensions change.
    ///
    /// This is drawn underneath of the underlying text
    fn draw(
        &mut self,
        focused: bool,
        text: &str,
        canvas: &mut Canvas<Window>,
        caret_position: f32,
    ) -> Result<(), String>;
}

/// a default provided single line text edit style
#[derive(Default)]
pub struct DefaultSingleLineEditStyle {}


impl SingleLineTextEditStyle for DefaultSingleLineEditStyle {
    fn draw(
        &mut self,
        focused: bool,
        text: &str,
        canvas: &mut Canvas<Window>,
        caret_position: f32,
    ) -> Result<(), String> {
        let _text = text; // todo!

        let size = canvas.output_size().map_err(|e| e.to_string())?;

        let amount_inward = 5i32;

        if size.0 <= amount_inward as u32 || size.1 <= amount_inward as u32 {
            return Ok(()); // too small to draw properly
        }

        let color = if focused {
            Color::RGB(118, 73, 206)
        } else {
            Color::RGB(50, 50, 50)
        };

        canvas.set_draw_color(color);

        let top_left_points = [
            Point::new(amount_inward, 0),
            Point::new(0, 0),
            Point::new(0, amount_inward),
        ];

        let bottom_left_points = [
            Point::new(amount_inward, size.1 as i32 - 1),
            Point::new(0, size.1 as i32 - 1),
            Point::new(0, size.1 as i32 - 1 - amount_inward),
        ];

        let top_right_points = [
            Point::new(size.0 as i32 - 1 - amount_inward, 0),
            Point::new(size.0 as i32 - 1, 0),
            Point::new(size.0 as i32 - 1, amount_inward),
        ];

        let bottom_right_points = [
            Point::new(size.0 as i32 - 1 - amount_inward, size.1 as i32 - 1),
            Point::new(size.0 as i32 - 1, size.1 as i32 - 1),
            Point::new(size.0 as i32 - 1, size.1 as i32 - 1 - amount_inward),
        ];

        let all_points = [
            top_left_points,
            top_right_points,
            bottom_left_points,
            bottom_right_points,
        ];

        for points in all_points {
            canvas.draw_lines(points.as_ref())?;
        }

        let caret_position = caret_position as i32;
        let caret_horizontal_spacing = 2;
        if caret_position > amount_inward + caret_horizontal_spacing
            && caret_position < size.0 as i32 - 1 - amount_inward - caret_horizontal_spacing
        {
            // big caret not at beginning or end
            canvas.draw_line(
                Point::new(caret_position, 0),
                Point::new(caret_position, size.1 as i32),
            )?;
        } else {
            // small caret
            let caret_vertical_spacing = 5;
            canvas.draw_line(
                Point::new(
                    caret_position,
                    amount_inward + 2 + caret_vertical_spacing,
                ),
                Point::new(
                    caret_position,
                    size.1 as i32 - (amount_inward + 3 + caret_vertical_spacing),
                ),
            )?;
        }

        Ok(())
    }
}

/// A cache for managing and reusing textures based on size and text
struct TextureVariantSizeCache<'sdl> {
    pub cache: Option<sdl2::render::Texture<'sdl>>,
    /// if this changes, the cache needs to be recomputed
    pub text_used: CompactString,
}

impl<'sdl> Default for TextureVariantSizeCache<'sdl> {
    fn default() -> Self {
        Self {
            cache: None,
            text_used: "".into(),
        }
    }
}

impl<'sdl> TextureVariantSizeCache<'sdl> {
    /// render txt or use the cache.  
    /// style is the style used to render the texture, with size.  
    /// creator is the texture creator for the canvas.  
    /// canvas is the window canvas.
    pub fn render(
        &mut self,
        style: &mut dyn SingleLineTextEditStyle,
        focused: bool,
        size: (u32, u32),
        text: CompactString,
        creator: &'sdl TextureCreator<WindowContext>,
        canvas: &mut Canvas<Window>,
        caret_position: f32,
    ) -> Result<&'_ Texture<'sdl>, String> {
        let cache = match self.cache.take().filter(|cache| {
            let q = cache.query();
            (q.width, q.height) == size && self.text_used == text
        }) {
            Some(cache) => cache, // reuse cache
            None => {
                // the size has changed or this is the first time calling.
                // either way, needs re-render
                let mut texture = creator
                    .create_texture_target(PixelFormatEnum::ARGB8888, size.0, size.1)
                    .map_err(|e| e.to_string())?;
                texture.set_blend_mode(sdl2::render::BlendMode::Blend);

                let mut e_out: Option<String> = None;
                canvas
                    .with_texture_canvas(&mut texture, |canvas| {
                        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                        canvas.clear(); // required to prevent flickering

                        e_out = style.draw(focused, &text, canvas, caret_position).err();
                    })
                    .map_err(|e| e.to_string())?;

                if let Some(e) = e_out {
                    return Err(e);
                }
                self.text_used = text;
                texture
            }
        };

        Ok(self.cache.insert(cache))
    }
}

pub enum SingleLineTextInputSoundVariant {
    Focus,
    TextAdded,
    TextRemoved,
    Enter,
}

/// a style which does not play any sounds and is not reliant on sdl2-mixer being enabled
#[derive(Clone, Copy)]
pub struct EmptySingleLineTextInputSoundStyle {}

impl SingleLineTextInputSoundStyle for EmptySingleLineTextInputSoundStyle {
    fn play_sound(&mut self, _which: SingleLineTextInputSoundVariant) -> Result<(), String> {
        // nothing
        Ok(())
    }
}

pub trait SingleLineTextInputSoundStyle {
    fn play_sound(&mut self, which: SingleLineTextInputSoundVariant) -> Result<(), String>;
}

#[cfg(feature = "sdl2-mixer")]
#[derive(Clone, Copy)]
pub struct DefaultSingleLineTextInputSoundStyle<'sdl> {
    pub sound_manager: &'sdl Cell<Option<crate::util::audio::SoundManager>>,
    pub focus_sound_path: Option<&'sdl std::path::Path>,
    pub text_added_sound_path: Option<&'sdl std::path::Path>,
    pub text_removed_sound_path: Option<&'sdl std::path::Path>,
    pub enter_sound_path: Option<&'sdl std::path::Path>,
}

#[cfg(feature = "sdl2-mixer")]
impl<'sdl> SingleLineTextInputSoundStyle for DefaultSingleLineTextInputSoundStyle<'sdl> {
    fn play_sound(&mut self, which: SingleLineTextInputSoundVariant) -> Result<(), String> {
        let maybe_sound_path: Option<&std::path::Path> = match which {
            SingleLineTextInputSoundVariant::Focus => self.focus_sound_path,
            SingleLineTextInputSoundVariant::TextAdded => self.text_added_sound_path,
            SingleLineTextInputSoundVariant::TextRemoved => self.text_removed_sound_path,
            SingleLineTextInputSoundVariant::Enter => self.enter_sound_path,
        };
        let sound_path = match maybe_sound_path {
            Some(v) => v,
            None => return Ok(()),
        };

        let mut maybe_manager = self.sound_manager.take();
        let manager = match maybe_manager.as_mut() {
            Some(v) => v,
            // should never error, as it will always be returned to the cell
            None => return Err("couldn't reference sound manager".to_owned()),
        };
        let maybe_r = manager.get(sound_path);
        self.sound_manager.set(maybe_manager);
        let r = maybe_r?;
        // do not handle err here (e.g. not enough channels)
        let _channel = sdl2::mixer::Channel::all().play(&r, 0);
        Ok(())
    }
}

/// contains a single line label which is editable
pub struct SingleLineTextInput<'sdl, 'state> {
    /// what happens when return key pressed
    pub functionality: Box<dyn FnMut() -> Result<(), String> + 'state>,

    pub focus_id: RefCircularUIDCell<'sdl>,
    /// internal state for sound
    focused_previous_frame: bool,
    /// internal state for sound - limit with many type sounds at once
    previous_text_input_timestamp: u32,

    /// how does the text input look
    style: Box<dyn SingleLineTextEditStyle + 'sdl>,
    /// what sounds should be played when the text bos is interacted with
    sounds: Box<dyn SingleLineTextInputSoundStyle + 'sdl>,

    focused: TextureVariantSizeCache<'sdl>,
    not_focused: TextureVariantSizeCache<'sdl>,

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
    /// state stored for draw from update
    draw_pos: crate::util::rect::FRect,
}

impl<'sdl, 'state> SingleLineTextInput<'sdl, 'state> {
    pub fn new(
        functionality: Box<dyn FnMut() -> Result<(), String> + 'state>,
        style: Box<dyn SingleLineTextEditStyle + 'sdl>,
        sounds: Box<dyn SingleLineTextInputSoundStyle + 'sdl>,
        focus_id: RefCircularUIDCell<'sdl>,
        text: &'state dyn SingleLineTextEditState,
        text_properties: SingleLineTextRenderType,
        font_interface: Box<dyn SingleLineFontStyle<'sdl> + 'sdl>,
        creator: &'sdl TextureCreator<WindowContext>,
    ) -> Self {
        Self {
            functionality,
            style,
            sounds,
            focused: Default::default(),
            not_focused: Default::default(),
            focus_id,
            focused_previous_frame: false,
            previous_text_input_timestamp: 0,
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
            draw_pos: Default::default(),
        }
    }
}

impl<'sdl, 'state> Widget for SingleLineTextInput<'sdl, 'state> {
    fn min(
        &mut self,
    ) -> Result<(crate::util::length::MinLen, crate::util::length::MinLen), String> {
        Ok((MinLen::LAX, self.min_h))
    }

    fn min_h_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        self.min_h_fail_policy
    }

    fn max(
        &mut self,
    ) -> Result<(crate::util::length::MaxLen, crate::util::length::MaxLen), String> {
        Ok((MaxLen::LAX, self.max_h))
    }

    fn max_h_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        self.max_h_fail_policy
    }

    fn preferred_portion(
        &self,
    ) -> (
        crate::util::length::PreferredPortion,
        crate::util::length::PreferredPortion,
    ) {
        (self.preferred_w, self.preferred_h)
    }

    fn update(&mut self, mut event: WidgetUpdateEvent) -> Result<(), String> {
        self.draw_pos = event.position;

        // keys:
        // - only applicable if currently focused
        // - consume key event once used

        let focus_manager = match &mut event.focus_manager {
            Some(v) => v,
            None => {
                // a single line text input simply cannot function properly
                // without a focus manager. this is unlike a button or checkbox,
                // which still can be pressed and hovered with the mouse and
                // while not focused
                debug_assert!(false);
                return Ok(());
            }
        };

        // detect rising edge of focus, for sound playing
        let mut previously_focused = focus_manager.is_focused(self.focus_id.uid());

        if previously_focused && !self.focused_previous_frame {
            // detect if focus was sent to this widget for any reason by
            // something else since the last time it was updated
            self.sounds
                .play_sound(SingleLineTextInputSoundVariant::Focus)?;
        }

        for sdl_event in event.events.iter_mut().filter(|event| event.available()) {
            FocusManager::default_widget_focus_behavior(
                self.focus_id.0.get(),
                WidgetEventFocusSubset {
                    focus_manager,
                    position: event.position,
                    event: sdl_event,
                    clipping_rect: event.clipping_rect,
                    window_id: event.window_id,
                },
            );

            if !focus_manager.is_focused(self.focus_id.uid()) {
                // keys:
                // - only applicable if currently focused
                // - consume key event once used
                continue;
            }

            if !previously_focused {
                previously_focused = true;
                self.sounds
                    .play_sound(SingleLineTextInputSoundVariant::Focus)?;
            }

            if sdl_event.consumed() {
                continue; // consumed as a result of default_widget_focus_behavior
            }

            static SOUND_LIMITER: u32 = 50; // too frequent sounds bad

            // fix repeat logic
            // fix consume_event logic

            let (consume_event, maybe_err): (bool, Option<String>) = (|| {
                match &mut sdl_event.e {
                    // if enter key is released and this widget has focus then trigger the functionality
                    sdl2::event::Event::KeyUp {
                        repeat,
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        if *repeat {
                            return (true, None);
                        }
                        // generally, try to play the sound before the
                        // functionality happens
                        if let Err(err) = self
                            .sounds
                            .play_sound(SingleLineTextInputSoundVariant::Enter)
                        {
                            return (true, Some(err));
                        }

                        match (self.functionality)() {
                            Ok(()) => (true, None),
                            Err(e) => (true, Some(e)),
                        }
                    }
                    // if backspace is pressed then pop the last character
                    sdl2::event::Event::KeyDown {
                        keycode: Some(Keycode::Backspace),
                        keymod,
                        timestamp,
                        ..
                    } => {
                        let mut content = self.text.get();
                        if !content.is_empty()
                            && timestamp
                                .checked_sub(self.previous_text_input_timestamp)
                                .unwrap_or(SOUND_LIMITER)
                                >= SOUND_LIMITER
                        {
                            self.previous_text_input_timestamp = *timestamp;
                            if let Err(err) = self
                                .sounds
                                .play_sound(SingleLineTextInputSoundVariant::TextRemoved)
                            {
                                return (true, Some(err));
                            }
                        }
                        if keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD) {
                            content.clear();
                        } else {
                            content.pop();
                        }
                        self.text.set(content);
                        (true, None)
                    }
                    // if text is typed then append it to the text. a text input
                    // event is NOT a key down event. it handles utf8 typing
                    sdl2::event::Event::TextInput {
                        text, timestamp, ..
                    } => {
                        if timestamp
                            .checked_sub(self.previous_text_input_timestamp)
                            .unwrap_or(SOUND_LIMITER)
                            >= SOUND_LIMITER
                        {
                            self.previous_text_input_timestamp = *timestamp;
                            if let Err(err) = self
                                .sounds
                                .play_sound(SingleLineTextInputSoundVariant::TextAdded)
                            {
                                return (true, Some(err));
                            }
                        }

                        let mut content = self.text.get();
                        content += text;
                        self.text.set(content);
                        (true, None)
                    }
                    _ => {
                        (false, None)
                    }
                }
            })();

            // still consume the event first even if the consumption of the
            // event resulted in an error
            if consume_event {
                sdl_event.set_consumed();
            }

            if let Some(err) = maybe_err {
                return Err(err);
            }
        }

        self.focused_previous_frame = focus_manager.is_focused(self.focus_id.uid());

        Ok(())
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        self.draw_pos.x += pos_delta.0 as f32;
        self.draw_pos.y += pos_delta.1 as f32;
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        focus_manager: Option<&FocusManager>,
    ) -> Result<(), String> {
        let position: sdl2::rect::Rect = match self.draw_pos.into() {
            Some(v) => v,
            None => return Ok(()),
        };

        let point_size: u16 = match position.height().try_into() {
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
            cache.text_rendered == self.text.get().as_str()
                && cache.properties_rendered == properties
        }) {
            Some(cache) => cache,
            None => {
                // if the text of the render properties have changed, then the
                // text needs to be re-rendered
                let text = self.text.get();
                let texture =
                    self.font_interface
                        .render(text.as_str(), &properties, self.creator)?;
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

        #[derive(Debug)]
        enum CaretPosition {
            Left,
            Right,
            Other(f32),
        }

        // the implementation of SingleLineFontStyle typically gives a 1x1
        // replacement texture for rendering text of zero length
        let caret_position = if !cache.text_rendered.is_empty() && query.height != 0 {
            let new_height = position.height() as f32;

            let scaler = new_height / query.height as f32; // div is guarded
            let new_width = query.width as f32 * scaler;

            

            if new_width < position.width() as f32 {
                // the text input's width is smaller than where it wants to be drawn
                // left align the content

                // requires copy_f to preserve exact ratio, or else position
                // will flicker a bit while typing
                canvas.copy_f(
                    txt,
                    None,
                    sdl2::rect::FRect::new(
                        position.x as f32,
                        position.y as f32,
                        new_width,
                        new_height,
                    ),
                )?;
                CaretPosition::Other(new_width)
            } else {
                let width_portion = if new_width == 0. {
                    debug_assert!(false); // can't occur but just in case
                    0.
                } else {
                    position.width() as f32 / new_width
                };
                let width_amount = (query.width as f32 * width_portion) as u32;

                // the text input's width is greater than where it wants to be drawn
                // cut off and only show the rightmost part of it
                canvas.copy(
                    txt,
                    sdl2::rect::Rect::new(
                        (query.width - width_amount) as i32,
                        0,
                        width_amount,
                        query.height,
                    ),
                    position,
                )?;
                CaretPosition::Right
            }
        } else {
            CaretPosition::Left
        };

        self.cache = Some(cache);

        // apply the style
        let focused = focus_manager
            .map(|f| f.is_focused(self.focus_id.uid()))
            .unwrap_or(false);

        let cache = if focused {
            &mut self.focused
        } else {
            &mut self.not_focused
        };

        let txt = cache.render(
            self.style.as_mut(),
            focused,
            (position.width(), position.height()),
            self.text.get(),
            self.creator,
            canvas,
            match caret_position {
                CaretPosition::Left => 0.,
                CaretPosition::Right => position.width().saturating_sub(1) as f32,
                CaretPosition::Other(v) => v,
            },
        )?;

        canvas.copy(txt, None, Some(position))?;

        Ok(())
    }
}
