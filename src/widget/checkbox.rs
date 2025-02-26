use std::cell::Cell;

use sdl2::{
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
    pixels::{Color, PixelFormatEnum},
    rect::Point,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};

use crate::util::{
    focus::{
        point_in_position_and_clipping_rect, DefaultFocusBehaviorArg, FocusID, FocusManager
    },
    length::{MaxLen, MinLen},
};

use super::{Widget, WidgetUpdateEvent};

/// a different texture is rendered for each of the displayed states that a
/// checkbox can have
#[derive(Clone, Copy)]
pub enum CheckBoxTextureVariant {
    Idle,
    Focused,
    // Pressed <- impossible to be pressed yet not focused
    FocusedPressed,
    FocusChecked,
    FocusedPressedChecked,
    Checked,
    CheckedPressed,
}

impl CheckBoxTextureVariant {
    fn focused(&self) -> bool {
        match self {
            CheckBoxTextureVariant::Focused
            | CheckBoxTextureVariant::FocusedPressed
            | CheckBoxTextureVariant::FocusChecked
            | CheckBoxTextureVariant::FocusedPressedChecked => true,
            _ => false,
        }
    }

    fn pressed(&self) -> bool {
        match self {
            CheckBoxTextureVariant::FocusedPressed
            | CheckBoxTextureVariant::FocusedPressedChecked
            | CheckBoxTextureVariant::CheckedPressed => true,
            _ => false,
        }
    }

    fn checked(&self) -> bool {
        match self {
            CheckBoxTextureVariant::FocusChecked
            | CheckBoxTextureVariant::FocusedPressedChecked
            | CheckBoxTextureVariant::Checked
            | CheckBoxTextureVariant::CheckedPressed => true,
            _ => false,
        }
    }
}

/// indicates how a size cache should be drawn for a given variant
pub trait TextureVariantStyle<TVariant> {
    /// The texture will be redrawn only if the target dimensions change.
    fn draw(&mut self, variant: TVariant, canvas: &mut Canvas<Window>) -> Result<(), String>;
}

/// a default provided check box style
#[derive(Default)]
pub struct DefaultCheckBoxStyle {}

impl TextureVariantStyle<CheckBoxTextureVariant> for DefaultCheckBoxStyle {
    fn draw(
        &mut self,
        variant: CheckBoxTextureVariant,
        canvas: &mut Canvas<Window>,
    ) -> Result<(), String> {
        let size = canvas.output_size().map_err(|e| e.to_string())?;

        let amount_inward = 5i32;

        if size.0 <= amount_inward as u32 || size.1 <= amount_inward as u32 {
            return Ok(()); // too small to draw properly
        }

        let focused = variant.focused();
        let pressed = variant.pressed();
        let checked = variant.checked();

        let color = if focused {
            if pressed {
                Color::RGB(200, 200, 200)
            } else {
                Color::RGB(118, 73, 206)
            }
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

        // ============================ foreground =============================

        let check_size = 10i32;

        if size.0 <= check_size as u32 || size.1 <= check_size as u32 {
            return Ok(()); // too small to draw properly
        }

        let color = if checked {
            if pressed {
                Color::RGB(50, 0, 20) // falling
            } else {
                Color::RGB(0, 160, 0)
            }
        } else if pressed {
            Color::RGB(100, 200, 100) // rising
        } else {
            Color::RGB(50, 50, 50)
        };
        canvas.set_draw_color(color);

        let first_points = [
            Point::new(
                size.0 as i32 / 2 - check_size / 2,
                size.1 as i32 / 2 - check_size / 2,
            ),
            Point::new(
                size.0 as i32 / 2 + check_size / 2,
                size.1 as i32 / 2 + check_size / 2,
            ),
        ];

        let second_points = [
            Point::new(
                size.0 as i32 / 2 - check_size / 2,
                size.1 as i32 / 2 + check_size / 2,
            ),
            Point::new(
                size.0 as i32 / 2 + check_size / 2,
                size.1 as i32 / 2 - check_size / 2,
            ),
        ];

        let all_points = [first_points, second_points];

        for points in all_points {
            canvas.draw_lines(points.as_ref())?;
        }

        Ok(())
    }
}

/// A cache for managing and reusing textures based on some style variant and size.
pub(crate) struct TextureVariantSizeCache<'sdl, TVariant> {
    pub cache: Option<sdl2::render::Texture<'sdl>>,
    _marker: std::marker::PhantomData<TVariant>,
}

impl<'sdl, TVariant> Default for TextureVariantSizeCache<'sdl, TVariant> {
    fn default() -> Self {
        Self {
            cache: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'sdl, TVariant> TextureVariantSizeCache<'sdl, TVariant> {
    /// render txt or use the cache.  
    /// style is the style used to render the texture, with size.  
    /// creator is the texture creator for the canvas.  
    /// canvas is the window canvas.
    pub fn render(
        &mut self,
        style: &mut dyn TextureVariantStyle<TVariant>,
        variant: TVariant,
        size: (u32, u32),
        creator: &'sdl TextureCreator<WindowContext>,
        canvas: &mut Canvas<Window>,
    ) -> Result<&'_ Texture<'sdl>, String> {
        let cache = match self.cache.take().filter(|cache| {
            let q = cache.query();
            (q.width, q.height) == size
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

                        e_out = style.draw(variant, canvas).err();
                    })
                    .map_err(|e| e.to_string())?;

                if let Some(e) = e_out {
                    return Err(e);
                }
                texture
            }
        };

        Ok(self.cache.insert(cache))
    }
}

/// for which sound should be played, for a widget that is focusable and
/// press-able (like a checkbox or a button)
pub enum FocusPressWidgetSoundVariant {
    /// focused or hovered
    Focus,
    Press,
    Release,
}

pub trait FocusPressWidgetSoundStyle {
    fn play_sound(&mut self, which: FocusPressWidgetSoundVariant) -> Result<(), String>;
}

/// a style which does not play any sounds and is not reliant on sdl2-mixer being enabled
#[derive(Clone, Copy)]
pub struct EmptyFocusPressWidgetSoundStyle {}

impl FocusPressWidgetSoundStyle for EmptyFocusPressWidgetSoundStyle {
    fn play_sound(&mut self, _which: FocusPressWidgetSoundVariant) -> Result<(), String> {
        // nothing
        Ok(())
    }
}

#[cfg(feature = "sdl2-mixer")]
#[derive(Clone, Copy)]
pub struct DefaultFocusPressWidgetSoundStyle<'sdl> {
    pub sound_manager: &'sdl Cell<Option<crate::util::audio::SoundManager>>,
    pub focus_sound_path: Option<&'sdl std::path::Path>,
    pub press_sound_path: Option<&'sdl std::path::Path>,
    pub release_sound_path: Option<&'sdl std::path::Path>,
}

#[cfg(feature = "sdl2-mixer")]
impl<'sdl> FocusPressWidgetSoundStyle for DefaultFocusPressWidgetSoundStyle<'sdl> {
    fn play_sound(&mut self, which: FocusPressWidgetSoundVariant) -> Result<(), String> {
        let maybe_sound_path: Option<&std::path::Path> = match which {
            FocusPressWidgetSoundVariant::Focus => self.focus_sound_path,
            FocusPressWidgetSoundVariant::Press => self.press_sound_path,
            FocusPressWidgetSoundVariant::Release => self.release_sound_path,
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

pub struct CheckBox<'sdl, 'state> {
    pub checked: &'state Cell<bool>,
    pub focus_id: FocusID,
    /// internal state for drawing
    pressed: bool,
    /// hovered is only used if no focus manager is available
    hovered: bool,

    /// internal state for sound
    focused_previous_frame: bool,

    pub size: f32,
    creator: &'sdl TextureCreator<WindowContext>,

    /// state stored for draw from update
    draw_pos: crate::util::rect::FRect,

    /// how does the checkbox look
    style: Box<dyn TextureVariantStyle<CheckBoxTextureVariant> + 'sdl>,
    /// what sounds should be played when the checkbox is interacted with
    sounds: Box<dyn FocusPressWidgetSoundStyle + 'sdl>,

    idle: TextureVariantSizeCache<'sdl, CheckBoxTextureVariant>,
    focused: TextureVariantSizeCache<'sdl, CheckBoxTextureVariant>,
    focused_pressed: TextureVariantSizeCache<'sdl, CheckBoxTextureVariant>,
    focused_checked: TextureVariantSizeCache<'sdl, CheckBoxTextureVariant>,
    focused_checked_pressed: TextureVariantSizeCache<'sdl, CheckBoxTextureVariant>,
    idle_checked: TextureVariantSizeCache<'sdl, CheckBoxTextureVariant>,
    checked_pressed: TextureVariantSizeCache<'sdl, CheckBoxTextureVariant>,
}

impl<'sdl, 'state> CheckBox<'sdl, 'state> {
    pub fn new(
        checked: &'state Cell<bool>,
        focus_id: FocusID,
        style: Box<dyn TextureVariantStyle<CheckBoxTextureVariant> + 'sdl>,
        sounds: Box<dyn FocusPressWidgetSoundStyle + 'sdl>,
        creator: &'sdl TextureCreator<WindowContext>,
    ) -> Self {
        Self {
            checked,
            focus_id,
            pressed: false,
            hovered: false,
            focused_previous_frame: false,
            style,
            sounds,
            size: 30.,
            creator,
            draw_pos: Default::default(),
            idle: Default::default(),
            idle_checked: Default::default(),
            checked_pressed: Default::default(),
            focused: Default::default(),
            focused_checked: Default::default(),
            focused_checked_pressed: Default::default(),
            focused_pressed: Default::default(),
        }
    }
}

/// update implementation for something which can be focused and pressed
pub(crate) fn focus_press_update_implementation<T>(
    hovered: &mut bool,
    pressed: &mut bool,
    focused_previous_frame: &mut bool,
    focus_id: &FocusID,
    mut event: WidgetUpdateEvent,
    functionality: &mut T,
    sounds: &mut dyn FocusPressWidgetSoundStyle,
) -> Result<(), String>
where
    T: FnMut() -> Result<(), String> + ?Sized,
{
    let has_focus_at_beginning = event.focus_manager.is_focused(focus_id);

    // detect if focus was sent to this widget for any reason by something else
    // since the last time it was updated
    if has_focus_at_beginning && !*focused_previous_frame {
        sounds.play_sound(FocusPressWidgetSoundVariant::Focus)?;
    }

    // used to detect rising edge, for when the focus or hover is gained on the
    // widget. at that point, play sound
    //
    // this part got a bit messy, but here's how it's used:
    // - not touched in key events, as key events are only applicable when the
    //   widget is already focused (can't be rising edge if already positive)
    // - any mouse stuff over the widget - set to true
    // - if mouse moved and was previously false, play sound and set to true
    let mut focus_sound_state = *hovered || has_focus_at_beginning;

    // value updated each frame
    *hovered = false;
    *pressed = false;

    for sdl_event in event.events.iter_mut().filter(|e| e.available()) {
        FocusManager::default_widget_focus_behavior(
            focus_id,
            DefaultFocusBehaviorArg {
                focus_manager: &mut event.focus_manager,
                position: event.position,
                event: sdl_event,
                clipping_rect: event.clipping_rect,
                window_id: event.window_id,
            },
        );
        if sdl_event.consumed() {
            continue; // consumed as a result of default_widget_focus_behavior
        }

        match sdl_event.e {
            // keys:
            // - only applicable if currently focused
            // - consume key event once used
            sdl2::event::Event::KeyDown {
                repeat,
                keycode: Some(Keycode::Tab),
                keymod,
                ..
            } => {
                // tab and shift tab go to next, previous widget respectively.
                // only if this current widget is focused. and consume the event
                if event.focus_manager.is_focused(&focus_id) {
                    sdl_event.set_consumed();
                    if repeat {
                        continue;
                    }
                    if keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD) {
                        event.focus_manager.0 = Some(focus_id.previous.clone());
                    } else {
                        event.focus_manager.0 = Some(focus_id.next.clone());
                    }
                }
            }
            sdl2::event::Event::KeyDown {
                repeat,
                keycode: Some(Keycode::Return),
                ..
            } => {
                // enter key pressed down. only if currently focused
                if event.focus_manager.is_focused(&focus_id) {
                    sdl_event.set_consumed();
                    if repeat {
                        continue;
                    }
                    *pressed = true;
                    sounds.play_sound(FocusPressWidgetSoundVariant::Press)?;
                }
            }
            sdl2::event::Event::KeyUp {
                repeat,
                keycode: Some(Keycode::Return),
                ..
            } => {
                // enter key released. only if currently focused.
                if event.focus_manager.is_focused(focus_id) {
                    sdl_event.set_consumed(); // consume before trying functionality
                    if repeat {
                        continue;
                    }
                    sounds.play_sound(FocusPressWidgetSoundVariant::Release)?;
                    match functionality() {
                        Ok(()) => (),
                        Err(e) => return Err(e),
                    };
                }
            }
            // mouse:
            // - consume mouse down and up (but not mouse motion)
            // - doesn't check if currently focused (mouse over widget + events
            //   haven't been consumed is good enough)
            // - sets focus to current widget when consumed
            sdl2::event::Event::MouseMotion {
                mousestate,
                x,
                y,
                window_id,
                ..
            } => {
                if window_id != event.window_id {
                    continue; // not for me!
                }
                let position: Option<sdl2::rect::Rect> = event.position.into();
                if let Some(position) = position {
                    if point_in_position_and_clipping_rect(x, y, position, event.clipping_rect) {
                        *hovered = true;
                        if !mousestate.left() {
                            if !focus_sound_state {
                                focus_sound_state = true;
                                sounds.play_sound(FocusPressWidgetSoundVariant::Focus)?;
                            }
                            continue;
                        }
                        if !focus_sound_state {
                            focus_sound_state = true;
                            sounds.play_sound(FocusPressWidgetSoundVariant::Press)?;
                        }

                        // the mouse was moved over the widget AND the left
                        // button is pressed
                        //
                        // generally never consume mouse motion events
                        *pressed = true;
                        event.focus_manager.0 = Some(focus_id.me.clone());
                    }
                }
            }
            sdl2::event::Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x,
                y,
                window_id,
                ..
            } => {
                if window_id != event.window_id {
                    continue; // not for me!
                }
                let position: Option<sdl2::rect::Rect> = event.position.into();
                if let Some(position) = position {
                    if point_in_position_and_clipping_rect(x, y, position, event.clipping_rect) {
                        sounds.play_sound(FocusPressWidgetSoundVariant::Press)?;
                        // the left mouse button was pressed on this widget
                        *pressed = true;
                        *hovered = true;
                        focus_sound_state = true;
                        sdl_event.set_consumed();
                        event.focus_manager.0 = Some(focus_id.me.clone());
                    }
                }
            }
            sdl2::event::Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                x,
                y,
                window_id,
                ..
            } => {
                if window_id != event.window_id {
                    continue; // not for me!
                }
                // ok even if not focused (button click works even if no
                // focus manager is used at all)
                let position: Option<sdl2::rect::Rect> = event.position.into();
                if let Some(position) = position {
                    if point_in_position_and_clipping_rect(x, y, position, event.clipping_rect) {
                        *pressed = false;
                        *hovered = true;
                        focus_sound_state = true;
                        sdl_event.set_consumed();
                        event.focus_manager.0 = Some(focus_id.me.clone());
                        sounds.play_sound(FocusPressWidgetSoundVariant::Release)?;
                        match functionality() {
                            Ok(()) => (),
                            Err(e) => return Err(e),
                        };
                    }
                }
            }
            _ => {}
        }
    }

    *focused_previous_frame = event
        .focus_manager.is_focused(focus_id);

    Ok(())
}

impl<'sdl, 'state> Widget for CheckBox<'sdl, 'state> {
    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        Ok((MinLen(self.size), MinLen(self.size)))
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        Ok((MaxLen(self.size), MaxLen(self.size)))
    }

    fn update(&mut self, event: WidgetUpdateEvent) -> Result<(), String> {
        self.draw_pos = event.position;
        focus_press_update_implementation(
            &mut self.hovered,
            &mut self.pressed,
            &mut self.focused_previous_frame,
            &self.focus_id,
            event,
            &mut || {
                let v = self.checked.get();
                let v = !v;
                self.checked.set(v);
                Ok(())
            },
            self.sounds.as_mut(),
        )
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        self.draw_pos.x += pos_delta.0 as f32;
        self.draw_pos.y += pos_delta.1 as f32;
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        focus_manager: &FocusManager,
    ) -> Result<(), String> {
        let position: sdl2::rect::Rect = match self.draw_pos.into() {
            Some(v) => v,
            // the rest of this is just for drawing or being clicked, both
            // require non-zero area position
            None => return Ok(()),
        };

        let focused = focus_manager.is_focused(&self.focus_id);
        let checked = self.checked.get();
        let variant = if focused || self.hovered {
            if self.pressed {
                if checked {
                    CheckBoxTextureVariant::FocusedPressedChecked
                } else {
                    CheckBoxTextureVariant::FocusedPressed
                }
            } else if checked {
                CheckBoxTextureVariant::FocusChecked
            } else {
                CheckBoxTextureVariant::Focused
            }
        } else if checked {
            if self.pressed {
                CheckBoxTextureVariant::CheckedPressed
            } else {
                CheckBoxTextureVariant::Checked
            }
        } else {
            CheckBoxTextureVariant::Idle
        };

        let cache = match variant {
            CheckBoxTextureVariant::Idle => &mut self.idle,
            CheckBoxTextureVariant::Focused => &mut self.focused,
            CheckBoxTextureVariant::FocusedPressed => &mut self.focused_pressed,
            CheckBoxTextureVariant::FocusChecked => &mut self.focused_checked,
            CheckBoxTextureVariant::FocusedPressedChecked => &mut self.focused_checked_pressed,
            CheckBoxTextureVariant::Checked => &mut self.idle_checked,
            CheckBoxTextureVariant::CheckedPressed => &mut self.checked_pressed,
        };

        let txt = cache.render(
            self.style.as_mut(),
            variant,
            (position.width(), position.height()),
            self.creator,
            canvas,
        )?;

        canvas.copy(txt, None, Some(position))?;
        Ok(())
    }
}
