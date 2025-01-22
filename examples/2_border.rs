use example_common::fancy_surface;
use tiny_sdl2_gui::{
    util::length::{MinLen, MinLenPolicy},
    widget::{
        border::{Bevel, Border},
        texture::{AspectRatioFailPolicy, Texture},
        widget::{draw_gui, update_gui, SDLEvent},
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 400;

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    let window = sdl_video_subsystem
        .window("border", WIDTH, HEIGHT)
        .resizable()
        .position_centered()
        .build().unwrap();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let surface = fancy_surface::and();
    let texture = texture_creator
        .create_texture_from_surface(surface)
        .expect("err create texture");
    let mut texture_widget = Texture::new(&texture);
    texture_widget.request_aspect_ratio = false;
    texture_widget.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
    texture_widget.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);

    let mut bevel = Bevel::default();
    bevel.width = 10;
    let mut border = Border::new(&mut texture_widget, &texture_creator, Box::new(bevel));

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

        let empty = events_accumulator.len() == 0; // lower cpu usage when idle

        if !empty {
            match update_gui(&mut border, &mut canvas, &mut events_accumulator, None) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            for e in events_accumulator.iter_mut().filter(|e| e.available()) {
                match e.e {
                    sdl2::event::Event::KeyDown {
                        keycode: Some(sdl2::keyboard::Keycode::Escape),
                        repeat,
                        ..
                    } => {
                        // if unprocessed escape key
                        e.set_consumed(); // intentional redundant
                        if repeat {
                            continue;
                        }
                        break 'running;
                    }
                    _ => {}
                }
            }
            events_accumulator.clear(); // clear after use
            
            // set background black
            canvas.set_draw_color(sdl2::pixels::Color::BLACK);
            canvas.clear();
            match draw_gui(&mut border, &mut canvas, &mut events_accumulator, None) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            canvas.present();
        }

    }
    std::process::ExitCode::SUCCESS
}
