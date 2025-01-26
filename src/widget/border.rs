use sdl2::{
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};

use crate::util::{
    focus::FocusManager,
    length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion},
    render::{
        bottom_right_center_seeking_rect_points, center_seeking_rect_points, interpolate_color,
        up_left_center_seeking_rect_points,
    },
};

use super::{Widget, WidgetUpdateEvent};

/// interface indicating what type of border the widget should use
pub trait BorderStyle {
    /// what is the width of this border (equal all the way around)
    fn width(&self) -> u32;

    /// draw the border on the provided texture canvas. the texture will be
    /// redrawn only if the target dimensions change.
    ///
    /// the texture canvas can have a width or height of down to 1 (regardless
    /// of specified border width)
    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String>;
}

/// a default provided border style
pub struct Bevel {
    pub top_left_outer_color: Color,
    pub top_left_inner_color: Color,
    pub bottom_right_outer_color: Color,
    pub bottom_right_inner_color: Color,
    pub width: u32,
}

impl Default for Bevel {
    fn default() -> Self {
        Self {
            top_left_inner_color: Color::RGB(50, 50, 50),
            top_left_outer_color: Color::RGB(255, 255, 255),
            bottom_right_outer_color: Color::RGB(50, 50, 50),
            bottom_right_inner_color: Color::RGB(255, 255, 255),
            width: 5,
        }
    }
}

impl BorderStyle for Bevel {
    fn width(&self) -> u32 {
        self.width
    }

    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let size = canvas.output_size()?;
        let smallest_parent_len = size.0.min(size.1);
        let actual_width = self.width.min((smallest_parent_len + 1) / 2);
        for i in 0i32..actual_width as i32 {
            let progress = if self.width < 2 {
                0.
            } else {
                i as f32 / (self.width - 1) as f32
            };
            let lighter_color = interpolate_color(
                self.top_left_outer_color,
                self.top_left_inner_color,
                progress,
            );
            let lighter_points = up_left_center_seeking_rect_points(i, size);
            canvas.set_draw_color(lighter_color);
            canvas.draw_lines(lighter_points.as_ref())?;

            let darker_color = interpolate_color(
                self.bottom_right_outer_color,
                self.bottom_right_inner_color,
                progress,
            );
            let darker_points = bottom_right_center_seeking_rect_points(i, size);
            canvas.set_draw_color(darker_color);
            canvas.draw_lines(darker_points.as_ref())?;
        }
        Ok(())
    }
}

/// a default provided border style
pub struct Gradient {
    pub outer_color: Color,
    pub inner_color: Color,
    pub width: u32,
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            outer_color: Color::RGB(200, 200, 200),
            inner_color: Color::RGB(100, 100, 100),
            width: 3,
        }
    }
}

impl BorderStyle for Gradient {
    fn width(&self) -> u32 {
        self.width
    }

    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let size = canvas.output_size()?;
        let smallest_parent_len = size.0.min(size.1);
        let actual_width = self.width.min((smallest_parent_len + 1) / 2);
        for i in 0i32..actual_width as i32 {
            let progress = if self.width < 2 {
                0.
            } else {
                i as f32 / (self.width - 1) as f32
            };

            let color = interpolate_color(self.outer_color, self.inner_color, progress);
            canvas.set_draw_color(color);

            let points = center_seeking_rect_points(i, size);
            canvas.draw_lines(points.as_ref())?
        }
        Ok(())
    }
}

/// a default provided border style
pub struct Line {
    pub color: Color,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            color: Color::RGB(200, 200, 200),
        }
    }
}

impl BorderStyle for Line {
    fn width(&self) -> u32 {
        1
    }

    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let size = canvas.output_size()?;
        canvas.set_draw_color(self.color);
        let points = center_seeking_rect_points(0, size);
        canvas.draw_lines(points.as_ref())
    }
}

/// a default provided border style
pub struct Empty {
    pub width: u32,
}

impl BorderStyle for Empty {
    fn width(&self) -> u32 {
        self.width
    }

    fn draw(&self, _canvas: &mut Canvas<Window>) -> Result<(), String> {
        Ok(())
    }
}

// contains a widget within a border
pub struct Border<'sdl> {
    pub contained: &'sdl mut dyn Widget,

    /// store state for draw from update
    border_draw_pos: crate::util::rect::FRect,

    style: Box<dyn BorderStyle>,

    /// texture is re-rendered only when the width or height changes
    texture: Option<Texture<'sdl>>,
    creator: &'sdl TextureCreator<WindowContext>,
}

impl<'sdl> Border<'sdl> {
    pub fn new(
        contains: &'sdl mut dyn Widget,
        creator: &'sdl TextureCreator<WindowContext>,
        style: Box<dyn BorderStyle>,
    ) -> Self {
        Self {
            contained: contains,
            border_draw_pos: Default::default(),
            creator,
            texture: Default::default(),
            style,
        }
    }
}

impl<'sdl> Widget for Border<'sdl> {
    fn preferred_portion(&self) -> (PreferredPortion, PreferredPortion) {
        self.contained.preferred_portion()
    }

    fn preferred_width_from_height(&mut self, pref_h: f32) -> Option<Result<f32, String>> {
        let sub_amount = self.style.width() * 2; // * 2 for each side
        let sub_amount = sub_amount as f32;
        // subtract border width from the pref input before passing to the
        // contained widget. then, add it back after getting the result
        let (amount_subtracted, pref_h) = if sub_amount >= pref_h {
            // atypical case (guard against subtract into negative range)
            (pref_h, 0.)
        } else {
            // typical case
            (sub_amount, pref_h - sub_amount)
        };
        self.contained
            .preferred_width_from_height(pref_h)
            .map(|some| some.map(|ok| ok + amount_subtracted))
    }

    fn preferred_height_from_width(&mut self, pref_w: f32) -> Option<Result<f32, String>> {
        let sub_amount = self.style.width() * 2; // * 2 for each side
        let sub_amount = sub_amount as f32;
        // subtract border width from the pref input before passing to the
        // contained widget. then, add it back after getting the result
        let (amount_subtracted, pref_w) = if sub_amount >= pref_w {
            // atypical case (guard against subtract into negative range)
            (pref_w, 0.)
        } else {
            // typical case
            (sub_amount, pref_w - sub_amount)
        };
        self.contained
            .preferred_height_from_width(pref_w)
            .map(|some| some.map(|ok| ok + amount_subtracted))
    }

    fn preferred_link_allowed_exceed_portion(&self) -> bool {
        self.contained.preferred_link_allowed_exceed_portion()
    }

    fn min_w_fail_policy(&self) -> MinLenFailPolicy {
        self.contained.min_w_fail_policy()
    }

    fn min_h_fail_policy(&self) -> MinLenFailPolicy {
        self.contained.min_h_fail_policy()
    }

    fn max_w_fail_policy(&self) -> MaxLenFailPolicy {
        self.contained.max_w_fail_policy()
    }

    fn max_h_fail_policy(&self) -> MaxLenFailPolicy {
        self.contained.max_h_fail_policy()
    }

    fn min(&mut self) -> Result<(MinLen, MinLen), String> {
        let baseline = MinLen((self.style.width() * 2) as f32);
        let m = self.contained.min()?;
        Ok((m.0.combined(baseline), m.1.combined(baseline)))
    }

    fn max(&mut self) -> Result<(MaxLen, MaxLen), String> {
        let baseline = MaxLen((self.style.width() * 2) as f32);
        let m = self.contained.max()?;
        Ok((m.0.combined(baseline), m.1.combined(baseline)))
    }

    fn update(&mut self, mut event: WidgetUpdateEvent) -> Result<(), String> {
        self.border_draw_pos = event.position;
        let style_width = self.style.width() as f32;
        let position_for_child = crate::util::rect::FRect {
            x: event.position.x + style_width,
            y: event.position.y + style_width,
            w: event.position.w - style_width * 2.,
            h: event.position.h - style_width * 2., // deliberately allow negative
        };
        self.contained.update(event.sub_event(position_for_child))
    }

    fn update_adjust_position(&mut self, pos_delta: (i32, i32)) {
        self.border_draw_pos.x += pos_delta.0 as f32;
        self.border_draw_pos.y += pos_delta.1 as f32;
        self.contained.update_adjust_position(pos_delta);
    }

    fn draw(
        &mut self,
        canvas: &mut sdl2::render::WindowCanvas,
        focus_manager: Option<&FocusManager>,
    ) -> Result<(), String> {
        self.contained.draw(canvas, focus_manager)?;

        let maybe_pos: Option<sdl2::rect::Rect> = self.border_draw_pos.into();

        if let Some(pos) = maybe_pos {
            // draw border if non empty position

            let cache = self.texture.take().filter(|texture| {
                let q = texture.query();
                q.width == pos.width() && q.height == pos.height()
            });

            let texture = match cache {
                Some(v) => {
                    v // texture can be reused
                }
                None => {
                    let mut texture = self
                        .creator
                        .create_texture_target(PixelFormatEnum::ARGB8888, pos.width(), pos.height())
                        .map_err(|e| e.to_string())?;
                    // the border is drawn over top of the contained texture. but the
                    // transparent part in the middle should still show through
                    texture.set_blend_mode(sdl2::render::BlendMode::Blend);

                    let mut e_out: Option<String> = None;

                    canvas
                        .with_texture_canvas(&mut texture, |canvas| {
                            canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                            canvas.clear(); // required to prevent flickering

                            if let Err(e) = self.style.draw(canvas) {
                                e_out = Some(e);
                            }
                        })
                        .map_err(|e| e.to_string())?;

                    if let Some(e) = e_out {
                        return Err(e);
                    }

                    texture
                }
            };

            canvas.copy(&texture, None, Some(pos))?;
            self.texture = Some(texture);
        }
        Ok(())
    }
}
