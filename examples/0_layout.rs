use sdl2::mouse::MouseButton;
use tiny_sdl2_gui::{
    layout::{
        horizontal_layout::HorizontalLayout,
        vertical_layout::{MajorAxisMaxLenPolicy, VerticalLayout},
    },
    util::length::{MaxLenFailPolicy, MinLenFailPolicy, MinLenPolicy},
    widget::{
        debug::Debug,
        strut::Strut,
        widget::{draw_gui, update_gui, SDLEvent, Widget},
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: f32 = 800.;
    const HEIGHT: f32 = 400.;

    const RESTRICT_MIN_SIZE: bool = false;

    let mut horizontal_layout = HorizontalLayout::default();

    let mut binding = Debug {
        min_h: (HEIGHT - 20.).into(),
        min_w: 100f32.into(),
        max_h: (HEIGHT - 20.).into(),
        max_w: (WIDTH / 5.).into(),
        ..Default::default()
    };
    horizontal_layout.elems.push(&mut binding);

    let mut binding = Debug {
        min_h: (HEIGHT - 20.).into(),
        min_w: 100f32.into(),
        max_h: (HEIGHT - 20.).into(),
        max_w: (WIDTH / 4.).into(),
        max_h_fail_policy: MaxLenFailPolicy::POSITIVE,
        min_h_fail_policy: MinLenFailPolicy::POSITIVE,
        ..Default::default()
    };
    horizontal_layout.elems.push(&mut binding);

    let mut binding = Debug {
        min_h: (HEIGHT - 20.).into(),
        min_w: 100f32.into(),
        max_h: (HEIGHT - 20.).into(),
        max_w: (WIDTH / 3.).into(),
        max_h_fail_policy: MaxLenFailPolicy::NEGATIVE,
        min_h_fail_policy: MinLenFailPolicy::NEGATIVE,
        ..Default::default()
    };
    horizontal_layout.elems.push(&mut binding);

    let mut binding = Strut::shrinkable(20., 0.);
    horizontal_layout.elems.push(&mut binding);

    let mut v_elem_0 = Debug {
        min_h: (HEIGHT / 4.).into(),
        max_h: (HEIGHT / 3.).into(),
        preferred_h: 0.5.into(),
        ..Default::default()
    };
    let mut v_elem_1 = Debug {
        min_h: (HEIGHT / 4.).into(),
        max_h: (HEIGHT / 2.).into(),
        preferred_h: 0.5.into(),
        ..Default::default()
    };
    let mut v_elem_2 = Debug {
        min_h: (HEIGHT / 4.).into(),
        max_h: (HEIGHT / 3.).into(),
        preferred_h: 0.5.into(),
        ..Default::default()
    };
    let mut binding = VerticalLayout {
        elems: vec![&mut v_elem_0, &mut v_elem_1, &mut v_elem_2],
        max_h_policy: MajorAxisMaxLenPolicy::Spread,
        ..Default::default()
    };
    horizontal_layout.elems.push(&mut binding);

    let mut v_elem_0 = Debug {
        min_h: (HEIGHT / 3.).into(),
        max_h: (HEIGHT / 3.).into(),
        preferred_h: 0.5.into(),
        ..Default::default()
    };

    let mut v_elem_1 = Debug {
        min_h: (HEIGHT / 3.).into(),
        max_h: (HEIGHT / 3.).into(),
        preferred_h: 0.5.into(),
        ..Default::default()
    };

    let binding = &mut VerticalLayout {
        max_h_fail_policy: MaxLenFailPolicy::NEGATIVE,
        elems: vec![&mut v_elem_0, &mut v_elem_1],
        ..Default::default()
    };
    horizontal_layout.elems.push(binding);

    let mut sdl = match example_common::sdl_util::SDLSystems::new(
        "debug widget + size constraint + layout",
        (WIDTH as u32, HEIGHT as u32),
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e.to_string());
            return std::process::ExitCode::FAILURE;
        }
    };

    // make window respect minimum size of entire GUI
    if RESTRICT_MIN_SIZE {
        horizontal_layout.min_h_policy = MinLenPolicy::Children;
        let min = horizontal_layout.min().unwrap();
        let _ = sdl
            .canvas
            .window_mut()
            .set_minimum_size(min.0 .0 as u32, min.1 .0 as u32);
    }

    let mut events_accumulator: Vec<SDLEvent> = Vec::new();
    'running: loop {
        for event in sdl.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {
                    events_accumulator.push(SDLEvent::new(event));
                }
            }
        }

        let empty = events_accumulator.len() == 0; // lower cpu usage when idle

        if !empty {
            // UPDATE
            match update_gui(
                &mut horizontal_layout,
                &mut sdl.canvas,
                &mut events_accumulator,
                None,
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }

            // set background black
            sdl.canvas.set_draw_color(sdl2::pixels::Color::BLACK);
            sdl.canvas.clear();

            // DRAW
            match draw_gui(
                &mut horizontal_layout,
                &mut sdl.canvas,
                &mut events_accumulator,
                None,
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            for e in events_accumulator.iter_mut().filter(|e| e.available()) {
                match e.e {
                    sdl2::event::Event::MouseButtonUp {
                        x,
                        y,
                        mouse_btn: MouseButton::Left,
                        ..
                    } => {
                        e.set_consumed(); // intentional redundant
                        println!("nothing was clicked! {:?}", (x, y));
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
