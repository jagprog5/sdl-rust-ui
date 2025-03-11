use std::time::Duration;

use example_common::gui_loop::gui_loop;
use sdl2::mouse::MouseButton;
use tiny_sdl2_gui::{
    layout::{
        horizontal_layout::HorizontalLayout,
        vertical_layout::{MajorAxisMaxLenPolicy, VerticalLayout},
    },
    util::{focus::FocusManager, length::{MaxLenFailPolicy, MinLen, MinLenFailPolicy, MinLenPolicy}},
    widget::{
        debug::Debug,
        strut::Strut,
        update_gui, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: f32 = 800.;
    const HEIGHT: f32 = 400.;
    const MAX_DELAY: Duration = Duration::from_millis(17);

    const RESTRICT_MIN_SIZE: bool = false;

    let mut focus_manager = FocusManager::default();
    
    let mut horizontal_0 = Debug::default();
    horizontal_0.min_h = (HEIGHT - 20.).into();
    horizontal_0.min_w = 100f32.into();
    horizontal_0.max_h = (HEIGHT - 20.).into();
    horizontal_0.max_w = (WIDTH / 5.).into();

    let mut horizontal_1 = Debug::default();
    horizontal_1.min_h = (HEIGHT - 20.).into();
    horizontal_1.min_w = 100f32.into();
    horizontal_1.max_h = (HEIGHT - 20.).into();
    horizontal_1.max_w = (WIDTH / 4.).into();
    horizontal_1.max_h_fail_policy = MaxLenFailPolicy::POSITIVE;
    horizontal_1.min_h_fail_policy = MinLenFailPolicy::POSITIVE;

    let mut horizontal_2 = Debug::default();
    horizontal_2.min_h = (HEIGHT - 20.).into();
    horizontal_2.min_w = 100f32.into();
    horizontal_2.max_h = (HEIGHT - 20.).into();
    horizontal_2.max_w = (WIDTH / 3.).into();
    horizontal_2.max_h_fail_policy = MaxLenFailPolicy::NEGATIVE;
    horizontal_2.min_h_fail_policy = MinLenFailPolicy::NEGATIVE;

    let horizontal_3 = Strut::shrinkable(20.0.into(), 0.0.into());

    let mut v_elem_0 = Debug::default();
    v_elem_0.min_h = (HEIGHT / 4.).into();
    v_elem_0.max_h = (HEIGHT / 3.).into();
    v_elem_0.preferred_h = 0.5.into();

    let mut v_elem_1 = Debug::default();
    v_elem_1.min_h = (HEIGHT / 4.).into();
    v_elem_1.max_h = (HEIGHT / 2.).into();
    v_elem_1.preferred_h = 0.5.into();

    let mut v_elem_2 = Debug::default();
    v_elem_2.min_h = (HEIGHT / 4.).into();
    v_elem_2.max_h = (HEIGHT / 3.).into();
    v_elem_2.preferred_h = 0.5.into();

    let mut horizontal_4 = VerticalLayout {
        max_h_policy: MajorAxisMaxLenPolicy::Spread,
        ..Default::default()
    };
    horizontal_4.elems.push(Box::new(v_elem_0));
    horizontal_4.elems.push(Box::new(v_elem_1));
    horizontal_4.elems.push(Box::new(v_elem_2));

    let mut v_elem_0 = Debug::default();
    v_elem_0.min_h = (HEIGHT / 3.).into();
    v_elem_0.max_h = (HEIGHT / 3.).into();
    v_elem_0.preferred_h = 0.5.into();

    let mut v_elem_1 = Debug::default();
    v_elem_1.min_h = (HEIGHT / 3.).into();
    v_elem_1.max_h = (HEIGHT / 3.).into();
    v_elem_1.preferred_h = 0.5.into();

    let mut horizontal_5 = VerticalLayout {
        max_h_fail_policy: MaxLenFailPolicy::NEGATIVE,
        ..Default::default()
    };

    horizontal_5.elems.push(Box::new(v_elem_0));
    horizontal_5.elems.push(Box::new(v_elem_1));
    
    let mut horizontal_layout = HorizontalLayout::default();
    // allow to be smaller than children, to show min len fail policies
    horizontal_layout.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);
    
    horizontal_layout.elems.push(Box::new(horizontal_0));
    horizontal_layout.elems.push(Box::new(horizontal_1));
    horizontal_layout.elems.push(Box::new(horizontal_2));
    horizontal_layout.elems.push(Box::new(horizontal_3));
    horizontal_layout.elems.push(Box::new(horizontal_4));
    horizontal_layout.elems.push(Box::new(horizontal_5));

    // SDL SYSTEMS

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    sdl_video_subsystem.text_input().start();
    let window = sdl_video_subsystem
        .window(
            "debug widget + size constraint + layout",
            WIDTH as u32,
            HEIGHT as u32,
        )
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // make window respect minimum size of entire GUI
    if RESTRICT_MIN_SIZE {
        horizontal_layout.min_h_policy = MinLenPolicy::Children;
        let min = horizontal_layout.min().unwrap();
        let _ = canvas
            .window_mut()
            .set_minimum_size(min.0 .0 as u32, min.1 .0 as u32);
    }

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
