use sdl2::surface::Surface;

/// 256x256
#[allow(dead_code)]
pub fn mul_mod() -> Surface<'static> {
    let mut surface = Surface::new(256, 256, sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
    surface.with_lock_mut(|buffer| {
        for x in 0i32..256 {
            for y in 0i32..256 {
                let pixel_offset = (4 * (x + y * 256)) as usize;
                buffer[pixel_offset] = ((y as f32 / 255.0) * 0xFF as f32) as u8;
                buffer[pixel_offset + 1] = ((x as f32 / 255.0) * 0xFF as f32) as u8;
                buffer[pixel_offset + 2] = 0xFF - buffer[pixel_offset + 1];
                buffer[pixel_offset + 3] = ((x * y) % 0xFF) as u8;
            }
        }
    });
    surface
}

/// 256x256
#[allow(dead_code)]
pub fn and() -> Surface<'static> {
    let mut surface = Surface::new(256, 256, sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
    surface.with_lock_mut(|buffer| {
        for x in 0i32..256 {
            for y in 0i32..256 {
                let pixel_offset = (4 * (x + y * 256)) as usize;
                buffer[pixel_offset] = ((y as f32 / 255.0) * 0xFF as f32) as u8;
                buffer[pixel_offset + 1] = ((x as f32 / 255.0) * 0xFF as f32) as u8;
                buffer[pixel_offset + 2] = 0xFF - buffer[pixel_offset + 1];
                if x & y == 0 || (255 - x) & (255 - y) == 0 {
                    buffer[pixel_offset + 3] = 0;
                } else {
                    buffer[pixel_offset + 3] = 0xFF;
                }
            }
        }
    });
    surface
}