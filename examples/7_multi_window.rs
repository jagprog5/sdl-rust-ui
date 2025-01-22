use std::{cell::Cell, fs::File, io::Read, path::Path};

use rand::Rng;
use sdl2::pixels::Color;
use tiny_sdl2_gui::{
    layout::scroller::{Scroller, ScrollerSizingPolicy}, util::{
        focus::{CircularUID, FocusManager, PRNGBytes, RefCircularUIDCell, UID}, font::{FontManager, SingleLineTextRenderType, TextRenderer}, length::PreferredPortion}, widget::{
        border::{Border, Empty, Gradient, Line}, button::{Button, DefaultButtonStyle}, checkbox::{CheckBox, DefaultCheckBoxStyle, EmptyFocusPressWidgetSoundStyle}, debug::CustomSizingControl, single_line_label::{DefaultSingleLineLabelState, SingleLineLabel}, widget::{draw_gui, update_gui, SDLEvent}
    }
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {

    let mut focus_manager = FocusManager::default();

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let mut font_file = File::open(
        Path::new(".")
            .join("examples")
            .join("assets")
            .join("TEMPSITC-REDUCED.TTF"),
    )
    .unwrap();
    let mut font_file_contents: Vec<u8> = Vec::new();
    font_file.read_to_end(&mut font_file_contents).unwrap();
    let font_file_contents = font_file_contents;
    drop(font_file);
    let font_manager = Cell::new(Some(FontManager::new(&ttf_context, &font_file_contents)));

    // SDL SYSTEMS
    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let window0 = sdl_video_subsystem
        .window("window 0", 300, 300)
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let mut canvas0 = window0.into_canvas().present_vsync().build().unwrap();
    // texture creators from different canvases must NOT be mixed
    let texture_creator0 = canvas0.texture_creator();

    let window1 = sdl_video_subsystem
        .window("window 1", 250, 250)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas1 = window1.into_canvas().present_vsync().build().unwrap();
    // texture creators from different canvases MUST NOT be mixed
    let texture_creator1 = canvas1.texture_creator();

    let window2 = sdl_video_subsystem
        .window("window 2", 200, 200)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas2 = window2.into_canvas().present_vsync().build().unwrap();
    // texture creators from different canvases MUST NOT be mixed
    let texture_creator2 = canvas2.texture_creator();

    let mut rng = rand::thread_rng();

    let mut get_prng_bytes = || {
        let mut bytes = [0u8; 8];
        rng.fill(&mut bytes);
        PRNGBytes(bytes)
    };

    let checkbox0_focus_id = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));
    let checkbox0_focus_id = RefCircularUIDCell(&checkbox0_focus_id);
    let checkbox0_state = Cell::new(false);
    let mut checkbox0 = CheckBox::new(
        &checkbox0_state,
        checkbox0_focus_id,
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(EmptyFocusPressWidgetSoundStyle {}),
        &texture_creator0,
    );

    // =========================

    let button_text = DefaultSingleLineLabelState {
        inner: Cell::new("button".into()),
    };
    let button_label = SingleLineLabel::new(
        &button_text,
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator1,
    );
    let button1_style = DefaultButtonStyle {
        label: button_label,
    };
    let button1_focus_id = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));
    let button1_focus_id = RefCircularUIDCell(&button1_focus_id);
    let mut button1 = Button::new(
        Box::new(|| {
            println!("Clicked!!!");
            Ok(())
        }),
        button1_focus_id,
        Box::new(button1_style),
        Box::new(EmptyFocusPressWidgetSoundStyle {}),
        &texture_creator1,
    );

    let mut button1_border = Border::new(&mut button1, &texture_creator1, Box::new(Empty { width: 10 }));

    // =========================

    let checkbox2_focus_id = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));
    let checkbox2_focus_id = RefCircularUIDCell(&checkbox2_focus_id);
    let checkbox2_state = Cell::new(false);
    let mut checkbox2 = CheckBox::new(
        &checkbox2_state,
        checkbox2_focus_id,
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(EmptyFocusPressWidgetSoundStyle {}),
        &texture_creator2,
    );

    // pad the checkbox a little bit for clarity
    let mut checkbox2_border = Border::new(
        &mut checkbox2,
        &texture_creator2,
        Box::new(Empty { width: 5 }),
    );

    // contain the checkbox and padding in a border
    let mut checkbox2_border = Border::new(
        &mut checkbox2_border,
        &texture_creator2,
        Box::new(Line::default()),
    );

    let inner_scroll_x = Cell::new(0i32);
    let inner_scroll_y = Cell::new(0i32);
    let mut checkbox2_scroller = Scroller::new(
        true,
        true,
        &inner_scroll_x,
        &inner_scroll_y,
        &mut checkbox2_border,
    );
    let mut sizing = CustomSizingControl::default();
    sizing.preferred_w = PreferredPortion(0.8);
    sizing.preferred_h = PreferredPortion(0.8);
    checkbox2_scroller.sizing_policy = ScrollerSizingPolicy::Custom(sizing, Default::default());

    let mut widget_complete_2 = Border::new(
        &mut checkbox2_scroller,
        &texture_creator2,
        Box::new(Gradient::default()),
    );

    // ===========================

    button1_focus_id.set_after(&checkbox0_focus_id);
    checkbox2_focus_id.set_after(&button1_focus_id);
    checkbox0_focus_id.set_after(&checkbox2_focus_id);

    let mut events_accumulator: Vec<SDLEvent> = Vec::new();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Window { win_event, .. } => {
                    match win_event {
                        sdl2::event::WindowEvent::Close => break 'running,
                        _ => {
                            events_accumulator.push(SDLEvent::new(event));
                        }
                    }
                }
                sdl2::event::Event::Quit { .. } => {
                    break 'running;
                }
                _ => {
                    events_accumulator.push(SDLEvent::new(event));
                },
            }
        }

        // lower cpu usage when idle (could be more fine grained, only updating
        // the relevant window instead)
        let empty = events_accumulator.len() == 0;

        if !empty {
            match update_gui(
                &mut checkbox0,
                &mut canvas0,
                &mut events_accumulator,
                Some(&mut focus_manager),
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            match update_gui(
                &mut button1_border,
                &mut canvas1,
                &mut events_accumulator,
                Some(&mut focus_manager),
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            match update_gui(
                &mut widget_complete_2,
                &mut canvas2,
                &mut events_accumulator,
                Some(&mut focus_manager),
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            FocusManager::default_start_focus_behavior(
                &mut focus_manager,
                &mut events_accumulator,
                checkbox0_focus_id.uid(),
                checkbox2_focus_id.uid(),
            );

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
            canvas0.set_draw_color(sdl2::pixels::Color::BLACK);
            canvas1.set_draw_color(sdl2::pixels::Color::BLACK);
            canvas2.set_draw_color(sdl2::pixels::Color::BLACK);
            canvas0.clear();
            canvas1.clear();
            canvas2.clear();

            // DRAW
            match draw_gui(
                &mut checkbox0,
                &mut canvas0,
                &mut events_accumulator,
                Some(&mut focus_manager),
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }

            match draw_gui(
                &mut button1_border,
                &mut canvas1,
                &mut events_accumulator,
                Some(&mut focus_manager),
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            match draw_gui(
                &mut widget_complete_2,
                &mut canvas2,
                &mut events_accumulator,
                Some(&mut focus_manager),
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }

            canvas0.present();
            canvas1.present();
            canvas2.present();
        }
    }
    std::process::ExitCode::SUCCESS
}
