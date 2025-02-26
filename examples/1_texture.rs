use std::time::Duration;

use example_common::{fancy_surface, gui_loop::gui_loop};
use sdl2::{mouse::MouseButton, surface::Surface};
use tiny_sdl2_gui::{
    layout::horizontal_layout::HorizontalLayout,
    util::{focus::FocusManager, length::{MaxLen, MaxLenPolicy, MinLen, MinLenPolicy}},
    widget::{
        texture::{AspectRatioFailPolicy, Texture},
        update_gui, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 256 * 4;
    const HEIGHT: u32 = 256;
    const MAX_DELAY: Duration = Duration::from_millis(17);

    let mut focus_manager = FocusManager::default();

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
    horizontal_layout.elems.push(Box::new(texture_widget0));
    horizontal_layout.elems.push(Box::new(texture_widget1));
    horizontal_layout.elems.push(Box::new(texture_widget2));
    horizontal_layout.elems.push(Box::new(texture_widget3));

    gui_loop(MAX_DELAY, &mut event_pump, |events| {
        // UPDATE
        match update_gui(
            &mut horizontal_layout,
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
        match &mut horizontal_layout.draw(&mut canvas, &focus_manager) {
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
