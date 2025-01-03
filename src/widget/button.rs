use crate::util::focus::FocusID;
use crate::util::length::{
    frect_to_rect, MaxLen, MinLen
};

use super::checkbox::{TextureVariantSizeCache, TextureVariantStyle};
use super::label::Label;
use super::widget::{Widget, WidgetEvent};

use sdl2::pixels::Color;

use sdl2::rect::FRect;
use sdl2::{
    rect::Point,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
};

#[derive(Clone, Copy)]
pub enum ButtonTextureVariant {
    Idle,
    Focused,
    FocusedPressed,
}

/// a default provided check box style
pub struct DefaultButtonStyle<'sdl, 'state> {
    pub label: Label<'sdl, 'state>,
}

/// as well as indicating how variants of the widget state populate a size cache
/// (TextureVariantStyle), it also dictates the button's sizing information
pub trait ButtonStyle<TVariant>: TextureVariantStyle<TVariant> {
    fn as_mut_widget(&mut self) -> &mut dyn Widget;
    fn as_widget(&self) -> &dyn Widget;
    fn as_mut_texture_variant_style(&mut self) -> &mut dyn TextureVariantStyle<TVariant>;
}

impl<'sdl, 'state> ButtonStyle<ButtonTextureVariant> for DefaultButtonStyle<'sdl, 'state> {
    fn as_mut_widget(&mut self) -> &mut dyn Widget {
        &mut self.label
    }

    fn as_widget(&self) -> &dyn Widget {
        &self.label
    }

    fn as_mut_texture_variant_style(&mut self) -> &mut dyn TextureVariantStyle<ButtonTextureVariant> {
        self
    }
}

impl<'sdl, 'state> TextureVariantStyle<ButtonTextureVariant> for DefaultButtonStyle<'sdl, 'state> {
    fn draw(
        &mut self,
        variant: ButtonTextureVariant,
        canvas: &mut Canvas<Window>,
    ) -> Result<(), String> {
        let size = canvas.output_size().map_err(|e| e.to_string())?;

        let amount_inward = 5i32;

        if size.0 <= amount_inward as u32 || size.1 <= amount_inward as u32 {
            return Ok(()); // too small to draw properly
        }

        let color = match variant {
            ButtonTextureVariant::Idle => Color::RGB(50, 50, 50),
            ButtonTextureVariant::Focused => Color::RGB(118, 73, 206),
            ButtonTextureVariant::FocusedPressed => Color::RGB(200, 200, 200),
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

        // draw foreground
        let mut event = WidgetEvent {
            focus_manager: None,
            position: Some(FRect::new(0., 0., size.0 as f32, size.1 as f32)),
            aspect_ratio_priority: Default::default(),
            events: Default::default(),
            canvas,
        };

        match self.label.update(event.dup()) {
            Ok(()) => (),
            Err(e) => return Err(e),
        };

        match self.label.draw(event) {
            Ok(()) => (),
            Err(e) => return Err(e),
        };

        Ok(())
    }
}

pub struct Button<'sdl> {
    functionality: Box<dyn FnMut() -> Result<(), String> + 'sdl>,
    pub focus_id: FocusID,
    pressed: bool, // internal state for drawing

    style: Box<dyn ButtonStyle<ButtonTextureVariant> +'sdl>,
    creator: &'sdl TextureCreator<WindowContext>,
    idle: TextureVariantSizeCache<'sdl, ButtonTextureVariant>,
    focused: TextureVariantSizeCache<'sdl, ButtonTextureVariant>,
    focus_pressed: TextureVariantSizeCache<'sdl, ButtonTextureVariant>,
}

impl<'sdl> Button<'sdl> {
    pub fn new(
        functionality: Box<dyn FnMut() -> Result<(), String> + 'sdl>,
        focus_id: FocusID,
        style: Box<dyn ButtonStyle<ButtonTextureVariant> + 'sdl>,
        creator: &'sdl TextureCreator<WindowContext>,
    ) -> Self {
        Self {
            functionality,
            focus_id,
            pressed: false,
            style,
            creator,
            idle: Default::default(),
            focused: Default::default(),
            focus_pressed: Default::default(),
        }
    }
}

impl<'sdl> Widget for Button<'sdl> {
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
        self.style.as_mut_widget().preferred_width_from_height(pref_h)
    }

    /// implementors should use this to enforce an aspect ratio
    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        self.style.as_mut_widget().preferred_height_from_width(pref_w)
    }

    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.style.as_widget().preferred_link_allowed_exceed_portion()
    }

    fn update(&mut self, event: WidgetEvent) -> Result<(), String> {
        let fun: &mut dyn FnMut() -> Result<(), String> = &mut self.functionality;
        super::checkbox::focus_press_update_implementation(
            &mut self.pressed,
            self.focus_id,
            event,
            fun
        )
    }

    fn draw(&mut self, event: super::widget::WidgetEvent) -> Result<(), String> {
        let position = match frect_to_rect(event.position) {
            Some(v) => v,
            // the rest of this is just for drawing or being clicked, both
            // require non-zero area position
            None => return Ok(()),
        };

        let focused = event.focus_manager.map(|f| f.is_focused(self.focus_id)).unwrap_or(false);
        let pressed = self.pressed;

        let variant = if focused {
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
            &self.creator,
            event.canvas,
        )?;

        event.canvas.copy(txt, None, Some(position))?;
        Ok(())
    }
}
