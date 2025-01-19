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

    let mut sdl = match example_common::sdl_util::SDLSystems::new("border", (WIDTH, HEIGHT)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e.to_string());
            return std::process::ExitCode::FAILURE;
        }
    };

    let surface = fancy_surface::and();
    let texture = sdl
        .texture_creator
        .create_texture_from_surface(surface)
        .expect("err create texture");
    let mut texture_widget = Texture::new(&texture);
    texture_widget.request_aspect_ratio = false;
    texture_widget.aspect_ratio_fail_policy = AspectRatioFailPolicy::Stretch;
    texture_widget.min_w_policy = MinLenPolicy::Literal(MinLen::LAX);
    texture_widget.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);

    let mut bevel = Bevel::default();
    bevel.width = 10;
    let mut border = Border::new(&mut texture_widget, &sdl.texture_creator, Box::new(bevel));

    let mut events_accumulator: Vec<SDLEvent> = Vec::new();
    'running: loop {
        for event in sdl.event_pump.poll_iter() {
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
            match update_gui(&mut border, &mut sdl.canvas, &mut events_accumulator, None) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            // set background black
            sdl.canvas.set_draw_color(sdl2::pixels::Color::BLACK);
            sdl.canvas.clear();
            match draw_gui(&mut border, &mut sdl.canvas, &mut events_accumulator, None) {
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
            sdl.canvas.present();
        }

        // steady loop of 60 (nothing fancier is needed)
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
    std::process::ExitCode::SUCCESS
}
