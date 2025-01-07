use std::cell::Cell;

use sdl2::{
    keyboard::Keycode, mouse::MouseButton, pixels::{Color, PixelFormatEnum}, rect::Point, render::{Canvas, Texture, TextureCreator}, video::{Window, WindowContext}
};

use crate::util::{
    focus::{FocusID, FocusManager},
    length::{MaxLen, MinLen},
};

use super::widget::{Widget, WidgetEvent};

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
pub struct DefaultCheckBoxStyle {}

impl Default for DefaultCheckBoxStyle {
    fn default() -> Self {
        Self {  }
    }
}

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
        } else {
            if pressed {
                Color::RGB(100, 200, 100) // rising
            } else {
                Color::RGB(50, 50, 50)
            }
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
pub struct TextureVariantSizeCache<'sdl, TVariant> {
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

pub struct CheckBox<'sdl, 'state> {
    pub checked: &'state Cell<bool>,
    pub focus_id: FocusID,
    pressed: bool,

    style: Box<dyn TextureVariantStyle<CheckBoxTextureVariant> + 'sdl>,
    pub size: f32,
    creator: &'sdl TextureCreator<WindowContext>,
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
        creator: &'sdl TextureCreator<WindowContext>,
    ) -> Self {
        Self {
            checked,
            focus_id,
            pressed: false,
            style,
            size: 30.,
            creator,
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

/// update fn implementation for something which can be focused and pressed
pub(crate) fn focus_press_update_implementation<T>(
    pressed: &mut bool,
    focus_id: FocusID,
    mut event: WidgetEvent,
    functionality: &mut T,
) -> Result<(), String>
where
    T: FnMut() -> Result<(), String> + ?Sized,
{
    FocusManager::default_widget_focus_behavior(focus_id, &mut event);
    let position: sdl2::rect::Rect = match event.position.into() {
        Some(v) => v,
        // the rest of this is just for drawing or being clicked, both
        // require non-zero area position
        None => return Ok(()),
    };

    // value updated each frame
    *pressed = false;

    for sdl_event in event.events.iter_mut().filter(|e| e.available()) {
        match sdl_event.e {
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            } => {
                if let Some(focus_manager) = &event.focus_manager {
                    if focus_manager.is_focused(focus_id) {
                        *pressed = true;
                        sdl_event.set_consumed();
                    }
                }
            }
            sdl2::event::Event::KeyUp {
                repeat: false,
                keycode: Some(Keycode::Return),
                ..
            } => {
                if let Some(focus_manager) = &event.focus_manager {
                    if focus_manager.is_focused(focus_id) {
                        match functionality() {
                            Ok(()) => (),
                            Err(e) => return Err(e),
                        };
                        sdl_event.set_consumed();
                    }
                }
            }
            sdl2::event::Event::MouseMotion {
                mousestate, x, y, ..
            } => {
                if !mousestate.left() {
                    continue;
                }
                if position.contains_point((x, y)) {
                    // ignore mouse events out of scroll area
                    let point_contained_in_clipping_rect = match event.canvas.clip_rect() {
                        sdl2::render::ClippingRect::Some(rect) => rect.contains_point((x, y)),
                        sdl2::render::ClippingRect::Zero => false,
                        sdl2::render::ClippingRect::None => true,
                    };
                    if !point_contained_in_clipping_rect {
                        continue;
                    }
                    *pressed = true;
                }
            }
            sdl2::event::Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                // ok even if not focused (button click works even if no
                // focus manager is used at all)
                if position.contains_point((x, y)) {
                    // ignore mouse events out of scroll area
                    let point_contained_in_clipping_rect = match event.canvas.clip_rect() {
                        sdl2::render::ClippingRect::Some(rect) => rect.contains_point((x, y)),
                        sdl2::render::ClippingRect::Zero => false,
                        sdl2::render::ClippingRect::None => true,
                    };
                    if !point_contained_in_clipping_rect {
                        continue;
                    }

                    sdl_event.set_consumed();
                    if let Some(focus_manager) = &mut event.focus_manager {
                        focus_manager.set_focus(focus_id);
                    }
                    *pressed = true;
                }
            }
            sdl2::event::Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                // ok even if not focused (button click works even if no
                // focus manager is used at all)
                if position.contains_point((x, y)) {
                    // ignore mouse events out of scroll area
                    let point_contained_in_clipping_rect = match event.canvas.clip_rect() {
                        sdl2::render::ClippingRect::Some(rect) => rect.contains_point((x, y)),
                        sdl2::render::ClippingRect::Zero => false,
                        sdl2::render::ClippingRect::None => true,
                    };
                    if !point_contained_in_clipping_rect {
                        continue;
                    }

                    match functionality() {
                        Ok(()) => (),
                        Err(e) => return Err(e),
                    };
                    sdl_event.set_consumed();
                }
            }
            _ => {}
        }
    }
    Ok(())
}

impl<'sdl, 'state> Widget for CheckBox<'sdl, 'state> {
    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        Ok((MinLen(self.size), MinLen(self.size)))
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        Ok((MaxLen(self.size), MaxLen(self.size)))
    }

    fn update(&mut self, event: WidgetEvent) -> Result<(), String> {
        focus_press_update_implementation(&mut self.pressed, self.focus_id, event, &mut || {
            let v = self.checked.get();
            let v = !v;
            self.checked.set(v);
            Ok(())
        })
    }

    fn draw(&mut self, event: WidgetEvent) -> Result<(), String> {
        let position: sdl2::rect::Rect = match event.position.into() {
            Some(v) => v,
            // the rest of this is just for drawing or being clicked, both
            // require non-zero area position
            None => return Ok(()),
        };

        let focused = event
            .focus_manager
            .map(|f| f.is_focused(self.focus_id))
            .unwrap_or(false);
        let checked = self.checked.get();
        let pressed = self.pressed;

        let variant = if focused {
            if pressed {
                if checked {
                    CheckBoxTextureVariant::FocusedPressedChecked
                } else {
                    CheckBoxTextureVariant::FocusedPressed
                }
            } else {
                if checked {
                    CheckBoxTextureVariant::FocusChecked
                } else {
                    CheckBoxTextureVariant::Focused
                }
            }
        } else {
            if checked {
                if pressed {
                    CheckBoxTextureVariant::CheckedPressed
                } else {
                    CheckBoxTextureVariant::Checked
                }
            } else {
                CheckBoxTextureVariant::Idle
            }
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
            &self.creator,
            event.canvas,
        )?;

        event.canvas.copy(txt, None, Some(position))?;
        Ok(())
    }
}
