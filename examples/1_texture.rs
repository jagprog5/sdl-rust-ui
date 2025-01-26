use example_common::fancy_surface;
use sdl2::surface::Surface;
use tiny_sdl2_gui::{
    layout::horizontal_layout::HorizontalLayout,
    util::length::{MaxLen, MaxLenPolicy, MinLen, MinLenPolicy},
    widget::{
        texture::{AspectRatioFailPolicy, Texture},
        update_gui, SDLEvent, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 256 * 4;
    const HEIGHT: u32 = 256;

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    let window = sdl_video_subsystem
        .window(
            "left three are aspect ratio failures. last one requests aspect ratio",
            WIDTH,
            HEIGHT,
        )
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let surface = fancy_surface::mul_mod();

    let mut surface0 = Surface::new(256, 256, sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
    surface
        .blit(None, &mut surface0, None)
        .expect("failed blit");
    let mut surface1 = Surface::new(256, 256, sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
    surface
        .blit(None, &mut surface1, None)
        .expect("failed blit");
    let mut surface2 = Surface::new(256, 256, sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
    surface
        .blit(None, &mut surface2, None)
        .expect("failed blit");
    let mut surface3 = Surface::new(256, 256, sdl2::pixels::PixelFormatEnum::ARGB8888).unwrap();
    surface
        .blit(None, &mut surface3, None)
        .expect("failed blit");

    let texture0 = texture_creator
        .create_texture_from_surface(surface0)
        .expect("err create texture");
    let mut texture_widget0 = Texture::new(&texture0);
    texture_widget0.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
    texture_widget0.request_aspect_ratio = false;
    texture_widget0.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget0.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
    texture_widget0.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget0.max_h_policy = MaxLenPolicy::Literal(MaxLen::LAX);

    let texture1 = texture_creator
        .create_texture_from_surface(surface1)
        .expect("err create texture");
    let mut texture_widget1 = Texture::new(&texture1);
    texture_widget1.aspect_ratio_fail_policy = AspectRatioFailPolicy::ZoomOut((0.5, 0.5));
    texture_widget1.request_aspect_ratio = false;
    texture_widget1.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget1.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
    texture_widget1.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget1.max_h_policy = MaxLenPolicy::Literal(MaxLen::LAX);

    let texture2 = texture_creator
        .create_texture_from_surface(surface2)
        .expect("err create texture");
    let mut texture_widget2 = Texture::new(&texture2);
    texture_widget2.aspect_ratio_fail_policy = AspectRatioFailPolicy::ZoomIn((0.5, 0.5));
    texture_widget2.request_aspect_ratio = false;
    texture_widget2.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget2.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
    texture_widget2.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget2.max_h_policy = MaxLenPolicy::Literal(MaxLen::LAX);

    let texture3 = texture_creator
        .create_texture_from_surface(surface3)
        .expect("err create texture");
    let mut texture_widget3 = Texture::new(&texture3);
    texture_widget3.preferred_link_allowed_exceed_portion = true;
    texture_widget3.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget3.max_w_policy = MaxLenPolicy::Literal(MaxLen::LAX);
    texture_widget3.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget3.max_h_policy = MaxLenPolicy::Literal(MaxLen::LAX);

    let mut horizontal_layout = HorizontalLayout::default();
    horizontal_layout.elems.push(&mut texture_widget0);
    horizontal_layout.elems.push(&mut texture_widget1);
    horizontal_layout.elems.push(&mut texture_widget2);
    horizontal_layout.elems.push(&mut texture_widget3);

    let mut events_accumulator: Vec<SDLEvent> = Vec::new();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    break 'running;
                }
                _ => {
                    events_accumulator.push(SDLEvent::new(event));
                }
            }
        }

        let empty = events_accumulator.is_empty(); // lower cpu usage when idle

        if !empty {
            match update_gui(
                &mut horizontal_layout,
                &mut events_accumulator,
                None,
                &canvas,
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            };
            for e in events_accumulator.iter_mut().filter(|e| e.available()) {
                if let sdl2::event::Event::KeyDown {
                        keycode: Some(sdl2::keyboard::Keycode::Escape),
                        repeat,
                        ..
                    } = e.e {
                    // if unprocessed escape key
                    e.set_consumed(); // intentional redundant
                    if repeat {
                        continue;
                    }
                    break 'running;
                }
            }
            events_accumulator.clear(); // clear after use

            // set background black
            canvas.set_draw_color(sdl2::pixels::Color::BLACK);
            canvas.clear();
            match horizontal_layout.draw(&mut canvas, None) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            };
            canvas.present();
        }
    }
    std::process::ExitCode::SUCCESS
}
