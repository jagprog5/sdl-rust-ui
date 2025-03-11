use std::time::Duration;

use example_common::{fancy_surface, gui_loop::gui_loop};
use sdl2::mouse::MouseButton;
use tiny_sdl2_gui::{
    util::{focus::FocusManager, length::{MinLen, MinLenPolicy}},
    widget::{
        border::{Bevel, Border},
        texture::{AspectRatioFailPolicy, Texture},
        update_gui, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 400;
    const MAX_DELAY: Duration = Duration::from_millis(17);

    let mut focus_manager = FocusManager::default();

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    sdl_video_subsystem.text_input().start();
    let window = sdl_video_subsystem
        .window("border", WIDTH, HEIGHT)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
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
    let mut border = Border::new(Box::new(texture_widget), &texture_creator, Box::new(bevel));

    gui_loop(MAX_DELAY, &mut event_pump, |events| {
        // UPDATE
        match update_gui(
            &mut border,
            events,
            &mut focus_manager,
            &canvas,
        ) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        };

        // after gui update, use whatever is left
        for e in events.iter_mut().filter(|e| e.available()) {
            match e.e {
                sdl2::event::Event::MouseButtonUp {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    e.set_consumed(); // intentional redundant
                    println!("nothing consumed the click! {:?}", (x, y));
                }
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
                    return true;
                }
                _ => {}
            }
        }

        // set background black
        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas.clear();

        // DRAW
        match &mut border.draw(&mut canvas, &focus_manager) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        }
        canvas.present();
        false
    });
    std::process::ExitCode::SUCCESS
}
