use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;

use crate::util::focus::{FocusID, FocusManager};
use crate::util::length::{MaxLen, MinLen};

use super::checkbox::{FocusPressWidgetSoundStyle, TextureVariantSizeCache, TextureVariantStyle};
use super::{Widget, WidgetUpdateEvent};

#[cfg(feature = "sdl2-ttf")]
use super::single_line_label::SingleLineLabel;

#[derive(Clone, Copy)]
pub enum ButtonTextureVariant {
    Idle,
    Focused,
    FocusedPressed,
}

// a button style which contains a label and draws a focus border with lines on
// top of that
#[cfg(feature = "sdl2-ttf")]
pub struct LabelButtonStyle<'sdl, 'state> {
    pub label: SingleLineLabel<'sdl, 'state>,
}

/// as well as indicating how variants of the widget state populate a size cache
/// (TextureVariantStyle), it also dictates the button's sizing information
pub trait ButtonStyle<TVariant>: TextureVariantStyle<TVariant> {
    fn as_mut_widget(&mut self) -> &mut dyn Widget;
    fn as_widget(&self) -> &dyn Widget;
    fn as_mut_texture_variant_style(&mut self) -> &mut dyn TextureVariantStyle<TVariant>;
}

#[cfg(feature = "sdl2-ttf")]
impl<'sdl, 'state> ButtonStyle<ButtonTextureVariant> for LabelButtonStyle<'sdl, 'state> {
    fn as_mut_widget(&mut self) -> &mut dyn Widget {
        &mut self.label
    }

    fn as_widget(&self) -> &dyn Widget {
        &self.label
    }

    fn as_mut_texture_variant_style(
        &mut self,
    ) -> &mut dyn TextureVariantStyle<ButtonTextureVariant> {
        self
    }
}

#[cfg(feature = "sdl2-ttf")]
impl<'sdl, 'state> TextureVariantStyle<ButtonTextureVariant> for LabelButtonStyle<'sdl, 'state> {
    fn draw(
        &mut self,
        variant: ButtonTextureVariant,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    ) -> Result<(), String> {
        let size = canvas.output_size().map_err(|e| e.to_string())?;

        let amount_inward = 5i32;

        if size.0 <= amount_inward as u32 || size.1 <= amount_inward as u32 {
            return Ok(()); // too small to draw properly
        }

        let color = match variant {
            ButtonTextureVariant::Idle => sdl2::pixels::Color::RGB(50, 50, 50),
            ButtonTextureVariant::Focused => sdl2::pixels::Color::RGB(118, 73, 206),
            ButtonTextureVariant::FocusedPressed => sdl2::pixels::Color::RGB(200, 200, 200),
        };

        canvas.set_draw_color(color);

        let top_left_points = [
            sdl2::rect::Point::new(amount_inward, 0),
            sdl2::rect::Point::new(0, 0),
            sdl2::rect::Point::new(0, amount_inward),
        ];

        let bottom_left_points = [
            sdl2::rect::Point::new(amount_inward, size.1 as i32 - 1),
            sdl2::rect::Point::new(0, size.1 as i32 - 1),
            sdl2::rect::Point::new(0, size.1 as i32 - 1 - amount_inward),
        ];

        let top_right_points = [
            sdl2::rect::Point::new(size.0 as i32 - 1 - amount_inward, 0),
            sdl2::rect::Point::new(size.0 as i32 - 1, 0),
            sdl2::rect::Point::new(size.0 as i32 - 1, amount_inward),
        ];

        let bottom_right_points = [
            sdl2::rect::Point::new(size.0 as i32 - 1 - amount_inward, size.1 as i32 - 1),
            sdl2::rect::Point::new(size.0 as i32 - 1, size.1 as i32 - 1),
            sdl2::rect::Point::new(size.0 as i32 - 1, size.1 as i32 - 1 - amount_inward),
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

        // draw foreground
        let mut event = WidgetUpdateEvent {
            position: crate::util::rect::FRect {
                x: 0.,
                y: 0.,
                w: size.0 as f32,
                h: size.1 as f32,
            },
            aspect_ratio_priority: Default::default(),
            events: Default::default(),
            focus_manager: &mut FocusManager::default(), // dummy
            clipping_rect: sdl2::render::ClippingRect::None,
            // does not matter, as the window_id is used to filter relevant
            // events and no events are being passed in
            window_id: u32::MAX,
        };

        match self.label.update(event.dup()) {
            Ok(()) => (),
            Err(e) => return Err(e),
        };

        match self.label.draw(canvas, &FocusManager::default()) {
            Ok(()) => (),
            Err(e) => return Err(e),
        };

        Ok(())
    }
}

pub struct Button<'sdl, 'state> {
    pub functionality: Box<dyn FnMut() -> Result<(), String> + 'state>,
    pub focus_id: FocusID,
    /// internal state for drawing
    pressed: bool,
    /// hovered is only used if no focus manager is available
    hovered: bool,
    /// internal state for sound
    focused_previous_frame: bool,

    /// how does the button look
    style: Box<dyn ButtonStyle<ButtonTextureVariant> + 'sdl>,
    /// what sounds should be played when the button is interacted with
    sounds: Box<dyn FocusPressWidgetSoundStyle + 'sdl>,

    /// state stored for draw from update
    draw_pos: crate::util::rect::FRect,

    creator: &'sdl TextureCreator<WindowContext>,
    idle: TextureVariantSizeCache<'sdl, ButtonTextureVariant>,
    focused: TextureVariantSizeCache<'sdl, ButtonTextureVariant>,
    focus_pressed: TextureVariantSizeCache<'sdl, ButtonTextureVariant>,
}

impl<'sdl, 'state> Button<'sdl, 'state> {
    pub fn new(
        functionality: Box<dyn FnMut() -> Result<(), String> + 'state>,
        focus_id: FocusID,
        style: Box<dyn ButtonStyle<ButtonTextureVariant> + 'sdl>,
        sounds: Box<dyn FocusPressWidgetSoundStyle + 'sdl>,
        creator: &'sdl TextureCreator<WindowContext>,
    ) -> Self {
        Self {
            functionality,
            focus_id,
            pressed: false,
            hovered: false,
            focused_previous_frame: false,
            style,
            sounds,
            creator,
            idle: Default::default(),
            focused: Default::default(),
            focus_pressed: Default::default(),
            draw_pos: Default::default(),
        }
    }
}

impl<'sdl, 'state> Widget for Button<'sdl, 'state> {
    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        self.style.as_mut_widget().min()
    }

    fn min_w_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        self.style.as_widget().min_w_fail_policy()
    }

    fn min_h_fail_policy(&self) -> crate::util::length::MinLenFailPolicy {
        self.style.as_widget().min_h_fail_policy()
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        self.style.as_mut_widget().max()
    }

    fn max_w_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        self.style.as_widget().max_w_fail_policy()
    }

    fn max_h_fail_policy(&self) -> crate::util::length::MaxLenFailPolicy {
        self.style.as_widget().max_h_fail_policy()
    }

    fn preferred_portion(
        &self,
    ) -> (
        crate::util::length::PreferredPortion,
        crate::util::length::PreferredPortion,
    ) {
        self.style.as_widget().preferred_portion()
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        self.style
            .as_mut_widget()
            .preferred_width_from_height(pref_h)
    }

    /// implementors should use this to enforce an aspect ratio
    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        self.style
            .as_mut_widget()
            .preferred_height_from_width(pref_w)
    }

    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.style
            .as_widget()
            .preferred_link_allowed_exceed_portion()
    }

    fn update(&mut self, event: WidgetUpdateEvent) -> Result<(), String> {
        self.draw_pos = event.position;
        let fun: &mut dyn FnMut() -> Result<(), String> = &mut self.functionality;
        super::checkbox::focus_press_update_implementation(
            &mut self.hovered,
            &mut self.pressed,
            &mut self.focused_previous_frame,
            &self.focus_id,
            event,
            fun,
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
        let pressed = self.pressed;

        let variant = if focused || self.hovered {
            if pressed {
                ButtonTextureVariant::FocusedPressed
            } else {
                ButtonTextureVariant::Focused
            }
        } else {
            ButtonTextureVariant::Idle
        };

        let cache = match variant {
            ButtonTextureVariant::Idle => &mut self.idle,
            ButtonTextureVariant::Focused => &mut self.focused,
            ButtonTextureVariant::FocusedPressed => &mut self.focus_pressed,
        };

        let txt = cache.render(
            self.style.as_mut_texture_variant_style(),
            variant,
            (position.width(), position.height()),
            self.creator,
            canvas,
        )?;

        canvas.copy(txt, None, Some(position))?;
        Ok(())
    }
}
