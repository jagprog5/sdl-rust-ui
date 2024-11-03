use noise::{
    utils::ColorGradient, Add, BasicMulti, Perlin, RotatePoint, ScaleBias, ScalePoint,
    Seedable, TranslatePoint, Turbulence,
};
#[cfg(feature = "noise")]
use noise::{Cylinders, Fbm, MultiFractal, NoiseFn, OpenSimplex};
use sdl2::{
    pixels::Color, rect::Rect, render::TextureCreator, surface::Surface, video::WindowContext,
};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use super::widget::{Widget, WidgetEvent};

pub trait SoftwareRenderBackgroundStyle: Send + Sync {
    /// retrieve color at coordinate to draw a static texture
    fn get(&self, x: usize, y: usize) -> Color;

    /// samples every n points in the x and y coordinates - tunable performance
    fn scale_factor(&self) -> u32;
}

#[cfg(feature = "noise")]
pub struct Smooth {
    noise: Fbm<OpenSimplex>,
    scale_factor: u32,
}

#[cfg(feature = "noise")]
impl Smooth {
    /// fast. suitable for whole background size rendering within 1 frame (with
    /// parallel feature enabled)
    pub fn fast(random_seed: u32) -> Self {
        Self {
            scale_factor: 3,
            noise: Fbm::<OpenSimplex>::new(random_seed)
                .set_frequency(0.05)
                .set_octaves(3),
        }
    }

    pub fn slow(random_seed: u32) -> Self {
        Self {
            noise: Fbm::<OpenSimplex>::new(random_seed)
                .set_frequency(0.05)
                .set_octaves(5),
            scale_factor: 1,
        }
    }
}

#[cfg(feature = "noise")]
impl SoftwareRenderBackgroundStyle for Smooth {
    fn get(&self, x: usize, y: usize) -> Color {
        let arg: [f64; 2] = [x as f64, y as f64];
        let noise_value = ((((self.noise.get(arg) + 1.0) / 2.) * 0xFF as f64).round()) as u8;
        Color::RGB(noise_value, noise_value, noise_value)
    }

    fn scale_factor(&self) -> u32 {
        self.scale_factor
    }
}

#[cfg(feature = "noise")]
pub struct Wood {
    // perhaps there is a better way of using templates for this?
    wood_fn: Turbulence<
        RotatePoint<
            TranslatePoint<
                Turbulence<
                    Add<f64, Cylinders, ScaleBias<f64, ScalePoint<BasicMulti<Perlin>>, 2>, 2>,
                    Perlin,
                >,
            >,
        >,
        Perlin,
    >,
    wood_gradient: ColorGradient,
    scale_factor: u32,
}

#[cfg(feature = "noise")]
impl Wood {
    pub fn new(random_seed: u32) -> Self {
        // modified from: https://github.com/Razaekel/noise-rs/
        // (same license)
        let base_wood = Cylinders::new().set_frequency(16.0);
        let wood_grain_noise = BasicMulti::<Perlin>::new(0)
            .set_frequency(48.0)
            .set_persistence(0.5)
            .set_lacunarity(2.20703125)
            .set_octaves(2);
        let scaled_base_wood_grain = ScalePoint::new(wood_grain_noise).set_z_scale(0.25);
        let wood_grain = ScaleBias::new(scaled_base_wood_grain)
            .set_scale(0.25)
            .set_bias(0.125);
        let combined_wood = Add::new(base_wood, wood_grain);
        let perturbed_wood = Turbulence::<_, Perlin>::new(combined_wood)
            .set_seed(random_seed)
            .set_frequency(4.0)
            .set_power(1.0 / 256.0)
            .set_roughness(4);
        let translated_wood = TranslatePoint::new(perturbed_wood).set_y_translation(1.48);
        let rotated_wood = RotatePoint::new(translated_wood).set_angles(84.0, 0.0, 0.0, 0.0);
        let final_wood = Turbulence::<_, Perlin>::new(rotated_wood)
            .set_seed(random_seed.checked_add(1).unwrap_or(0))
            .set_frequency(2.0)
            .set_power(1.0 / 64.0)
            .set_roughness(4);
        let wood_gradient = ColorGradient::new()
            .clear_gradient()
            .add_gradient_point(-1.000, [189, 94, 4, 255])
            .add_gradient_point(0.500, [144, 48, 6, 255])
            .add_gradient_point(1.0, [60, 10, 8, 255]);
        Self {
            wood_fn: final_wood,
            wood_gradient,
            scale_factor: 3,
        }
    }
}

#[cfg(feature = "noise")]
impl SoftwareRenderBackgroundStyle for Wood {
    fn get(&self, x: usize, y: usize) -> Color {
        let arg: [f64; 2] = [x as f64, y as f64];
        let arg = [arg[0] / 500., arg[1] / 500.];
        let val = self.wood_fn.get(arg);
        let val = self.wood_gradient.get_color(val);
        Color::RGBA(val[0], val[1], val[2], val[3])
    }

    fn scale_factor(&self) -> u32 {
        self.scale_factor
    }
}

// =============================================================================

/// based on width and height, if larger than cached then creates new surface and texture
struct SoftwareRenderBackgroundCache<'sdl> {
    pub texture: sdl2::render::Texture<'sdl>,
    pub surface: sdl2::surface::Surface<'sdl>, // reuse previous computation - only expanded size is calculated
}

/// suitable for background coloring. for example, multiple widgets can be
/// composed in a stacked layout.
pub struct SoftwareRenderBackground<'sdl, Style: SoftwareRenderBackgroundStyle> {
    style: Style,

    color_mod: (u8, u8, u8),

    creator: &'sdl TextureCreator<WindowContext>,
    cache: Option<SoftwareRenderBackgroundCache<'sdl>>,
}

impl<'sdl, Style: SoftwareRenderBackgroundStyle> SoftwareRenderBackground<'sdl, Style> {
    pub fn new(style: Style, creator: &'sdl TextureCreator<WindowContext>) -> Self {
        Self {
            style,
            creator,
            color_mod: (0xFF, 0xFF, 0xFF),
            cache: Default::default(),
        }
    }

    pub fn set_color_mod(&mut self, color_mod: (u8, u8, u8)) {
        self.color_mod = color_mod;
        if let Some(cache) = &mut self.cache {
            cache
                .texture
                .set_color_mod(self.color_mod.0, self.color_mod.1, self.color_mod.2);
        }
    }

    pub fn get_color_mod(&self) -> (u8, u8, u8) {
        self.color_mod
    }
}

impl<'sdl, Style: SoftwareRenderBackgroundStyle> Widget for SoftwareRenderBackground<'sdl, Style> {
    fn draw(&mut self, event: WidgetEvent) -> Result<(), String> {
        let position = match event.position {
            Some(v) => v,
            None => return Ok(()), // no input handling
        };

        let position = crate::util::length::frect_to_rect(position);

        let scale_factor = self.style.scale_factor();

        let (texture, surface) = match self.cache.take() {
            Some(cache) => {
                if cache.surface.width() >= position.width() / scale_factor
                    && cache.surface.height() >= position.height() / scale_factor
                {
                    // large enough to use cache
                    (cache.texture, cache.surface)
                } else {
                    let old_width = cache.surface.width();
                    let old_height = cache.surface.height();
                    let new_width = (position.width() / scale_factor).max(old_width);
                    let new_height = (position.height() / scale_factor).max(old_height);
                    // must expand texture in the cache
                    let mut surface = Surface::new(
                        new_width,
                        new_height,
                        sdl2::pixels::PixelFormatEnum::ARGB8888,
                    )?;

                    // reuse what was already computed
                    cache.surface.blit(None, &mut surface, None)?;

                    let row_stride = new_width as usize * 4;
                    surface.with_lock_mut(|buffer| {
                        // draw the expanded height
                        if new_height > cache.surface.height() {
                            #[cfg(feature = "rayon")]
                            let row_iter = buffer.par_chunks_exact_mut(row_stride);
                            #[cfg(not(feature = "rayon"))]
                            let row_iter = buffer.chunks_exact_mut(row_stride);

                            let row_iter = row_iter.skip(old_height as usize);
                            row_iter.enumerate().for_each(|(row_index, row)| {
                                let row_index = row_index + old_height as usize;
                                let pixel_iter = row.chunks_exact_mut(4);

                                pixel_iter.enumerate().for_each(|(pixel_index, pixel)| {
                                    let x = pixel_index;
                                    let y = row_index;
                                    let color = self
                                        .style
                                        .get(x * scale_factor as usize, y * scale_factor as usize);
                                    pixel[0] = color.b;
                                    pixel[1] = color.g;
                                    pixel[2] = color.r;
                                    pixel[3] = color.a;
                                });
                            });
                        }

                        // draw the expanded width + corner
                        if new_width > cache.surface.width() {
                            #[cfg(feature = "rayon")]
                            let row_iter = buffer.par_chunks_exact_mut(row_stride);
                            #[cfg(not(feature = "rayon"))]
                            let row_iter = buffer.chunks_exact_mut(row_stride);

                            row_iter.enumerate().for_each(|(row_index, row)| {
                                let pixel_iter = row.chunks_exact_mut(4);

                                let pixel_iter = pixel_iter.skip(old_width as usize);
                                pixel_iter.enumerate().for_each(|(pixel_index, pixel)| {
                                    let x = (pixel_index + old_width as usize) as usize;
                                    let y = row_index;
                                    let color = self
                                        .style
                                        .get(x * scale_factor as usize, y * scale_factor as usize);
                                    pixel[0] = color.b;
                                    pixel[1] = color.g;
                                    pixel[2] = color.r;
                                    pixel[3] = color.a;
                                });
                            });
                        }
                    });

                    let mut surface_copy = Surface::new(
                        new_width,
                        new_height,
                        sdl2::pixels::PixelFormatEnum::ARGB8888,
                    )?;

                    surface.blit(None, &mut surface_copy, None)?;

                    let mut texture = self
                        .creator
                        .create_texture_from_surface(surface)
                        .map_err(|e| e.to_string())?;
                    texture.set_color_mod(self.color_mod.0, self.color_mod.1, self.color_mod.2);
                    texture.set_scale_mode(sdl2::render::ScaleMode::Linear);
                    (texture, surface_copy)
                }
            }
            None => {
                // create texture from scratch
                let mut surface = Surface::new(
                    position.width() / scale_factor,
                    position.height() / scale_factor,
                    sdl2::pixels::PixelFormatEnum::ARGB8888,
                )?;

                surface.with_lock_mut(|buffer| {
                    let width = (position.width() / scale_factor) as usize;
                    let row_stride = width as usize * 4;

                    // let start = Instant::now();

                    #[cfg(feature = "rayon")]
                    let row_iter = buffer.par_chunks_exact_mut(row_stride);
                    #[cfg(not(feature = "rayon"))]
                    let row_iter = buffer.chunks_exact_mut(row_stride);

                    row_iter.enumerate().for_each(|(row_index, row)| {
                        let pixel_iter = row.chunks_exact_mut(4);
                        pixel_iter.enumerate().for_each(|(pixel_index, pixel)| {
                            let x = pixel_index;
                            let y = row_index;
                            let color = self
                                .style
                                .get(x * scale_factor as usize, y * scale_factor as usize);
                            pixel[0] = color.b;
                            pixel[1] = color.g;
                            pixel[2] = color.r;
                            pixel[3] = color.a;
                        });
                    });

                    // println!("{}", start.elapsed().as_millis());
                });

                let mut surface_copy = Surface::new(
                    position.width() / scale_factor,
                    position.height() / scale_factor,
                    sdl2::pixels::PixelFormatEnum::ARGB8888,
                )?;

                surface.blit(None, &mut surface_copy, None)?;

                let mut texture = self
                    .creator
                    .create_texture_from_surface(surface)
                    .map_err(|e| e.to_string())?;
                texture.set_color_mod(self.color_mod.0, self.color_mod.1, self.color_mod.2);
                texture.set_scale_mode(sdl2::render::ScaleMode::Linear);
                (texture, surface_copy)
            }
        };

        event.canvas.copy(
            &texture,
            Rect::new(
                0,
                0,
                position.width() / scale_factor,
                position.height() / scale_factor,
            ),
            position,
        )?;

        self.cache = Some(SoftwareRenderBackgroundCache { texture, surface });
        Ok(())
    }
}
