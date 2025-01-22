use sdl2::mouse::MouseButton;
use tiny_sdl2_gui::{
    layout::{
        horizontal_layout::HorizontalLayout,
        vertical_layout::{MajorAxisMaxLenPolicy, VerticalLayout},
    },
    util::length::{MaxLenFailPolicy, MinLen, MinLenFailPolicy, MinLenPolicy},
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
    // allow to be smaller than children, to show min len fail policies
    horizontal_layout.min_h_policy = MinLenPolicy::Literal(MinLen::LAX);

    let mut binding = Debug::default();
    binding.min_h = (HEIGHT - 20.).into();
    binding.min_w = 100f32.into();
    binding.max_h = (HEIGHT - 20.).into();
    binding.max_w = (WIDTH / 5.).into();
    
    horizontal_layout.elems.push(&mut binding);

    let mut binding = Debug::default();
    binding.min_h = (HEIGHT - 20.).into();
    binding.min_w = 100f32.into();
    binding.max_h = (HEIGHT - 20.).into();
    binding.max_w = (WIDTH / 4.).into();
    binding.max_h_fail_policy = MaxLenFailPolicy::POSITIVE;
    binding.min_h_fail_policy = MinLenFailPolicy::POSITIVE;

    horizontal_layout.elems.push(&mut binding);

    let mut binding = Debug::default();
    binding.min_h = (HEIGHT - 20.).into();
    binding.min_w = 100f32.into();
    binding.max_h = (HEIGHT - 20.).into();
    binding.max_w = (WIDTH / 3.).into();
    binding.max_h_fail_policy = MaxLenFailPolicy::NEGATIVE;
    binding.min_h_fail_policy = MinLenFailPolicy::NEGATIVE;
    horizontal_layout.elems.push(&mut binding);

    let mut binding = Strut::shrinkable(20., 0.);
    horizontal_layout.elems.push(&mut binding);

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

    let mut binding = VerticalLayout {
        elems: vec![&mut v_elem_0, &mut v_elem_1, &mut v_elem_2],
        max_h_policy: MajorAxisMaxLenPolicy::Spread,
        ..Default::default()
    };
    horizontal_layout.elems.push(&mut binding);

    let mut v_elem_0 = Debug::default();
    v_elem_0.min_h = (HEIGHT / 3.).into();
    v_elem_0.max_h = (HEIGHT / 3.).into();
    v_elem_0.preferred_h = 0.5.into();

    let mut v_elem_1 = Debug::default();
    v_elem_1.min_h = (HEIGHT / 3.).into();
    v_elem_1.max_h = (HEIGHT / 3.).into();
    v_elem_1.preferred_h = 0.5.into();

    let binding = &mut VerticalLayout {
        max_h_fail_policy: MaxLenFailPolicy::NEGATIVE,
        elems: vec![&mut v_elem_0, &mut v_elem_1],
        ..Default::default()
    };
    horizontal_layout.elems.push(binding);

    // SDL SYSTEMS

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    let window = sdl_video_subsystem
        .window("debug widget + size constraint + layout", WIDTH as u32, HEIGHT as u32)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // make window respect minimum size of entire GUI
    if RESTRICT_MIN_SIZE {
        horizontal_layout.min_h_policy = MinLenPolicy::Children;
        let min = horizontal_layout.min().unwrap();
        let _ = canvas
            .window_mut()
            .set_minimum_size(min.0 .0 as u32, min.1 .0 as u32);
    }

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
            // UPDATE
            match update_gui(
                &mut horizontal_layout,
                &mut canvas,
                &mut events_accumulator,
                None,
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }

            // after gui update, use whatever is left
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

            // DRAW
            match draw_gui(
                &mut horizontal_layout,
                &mut canvas,
                &mut events_accumulator,
                None,
            ) {
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
