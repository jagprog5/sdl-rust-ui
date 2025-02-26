use std::{cell::Cell, fs::File, io::Read, path::Path, time::Duration};

use sdl2::{mouse::MouseButton, pixels::Color};
use tiny_sdl2_gui::{
    layout::scroller::{Scroller, ScrollerSizingPolicy},
    util::{
        focus::{FocusID, FocusManager},
        font::{FontManager, SingleLineTextRenderType, TextRenderer},
        length::PreferredPortion,
    },
    widget::{
        border::{Border, Empty, Gradient, Line},
        button::{Button, LabelButtonStyle},
        checkbox::{CheckBox, DefaultCheckBoxStyle, EmptyFocusPressWidgetSoundStyle},
        debug::CustomSizingControl,
        single_line_label::SingleLineLabel,
        update_gui, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const MAX_DELAY: Duration = Duration::from_millis(17);
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

    let checkbox0_state = Cell::new(false);
    let mut checkbox0 = CheckBox::new(
        &checkbox0_state,
        FocusID {
            previous: "2".into(),
            me: "0".into(),
            next: "1".into(),
        },
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(EmptyFocusPressWidgetSoundStyle {}),
        &texture_creator0,
    );

    // =========================

    let button_label = SingleLineLabel::new(
        Cell::new("button".into()).into(),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator1,
    );
    let button1_style = LabelButtonStyle {
        label: button_label,
    };
    let button1 = Button::new(
        Box::new(|| {
            println!("Clicked!!!");
            Ok(())
        }),
        FocusID {
            previous: "0".into(),
            me: "1".into(),
            next: "2".into(),
        },
        Box::new(button1_style),
        Box::new(EmptyFocusPressWidgetSoundStyle {}),
        &texture_creator1,
    );

    let mut button1_border = Border::new(
        Box::new(button1),
        &texture_creator1,
        Box::new(Empty { width: 10 }),
    );

    // =========================

    let checkbox2_state = Cell::new(false);
    let checkbox2 = CheckBox::new(
        &checkbox2_state,
        FocusID {
            previous: "1".into(),
            me: "2".into(),
            next: "0".into(),
        },
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(EmptyFocusPressWidgetSoundStyle {}),
        &texture_creator2,
    );

    // pad the checkbox a little bit for clarity
    let checkbox2_border = Border::new(
        Box::new(checkbox2),
        &texture_creator2,
        Box::new(Empty { width: 5 }),
    );

    // contain the checkbox and padding in a border
    let checkbox2_border = Border::new(
        Box::new(checkbox2_border),
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
        Box::new(checkbox2_border),
    );
    let mut sizing = CustomSizingControl::default();
    sizing.preferred_w = PreferredPortion(0.8);
    sizing.preferred_h = PreferredPortion(0.8);
    checkbox2_scroller.sizing_policy = ScrollerSizingPolicy::Custom(sizing, Default::default());

    let mut widget_complete_2 = Border::new(
        Box::new(checkbox2_scroller),
        &texture_creator2,
        Box::new(Gradient::default()),
    );

    example_common::gui_loop::gui_loop(MAX_DELAY, &mut event_pump, |events| {
        // UPDATE
        match update_gui(
            &mut checkbox0,
            events,
            &mut focus_manager,
            &canvas0,
        ) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        };
        match update_gui(
            &mut button1_border,
            events,
            &mut focus_manager,
            &canvas1,
        ) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        };
        match update_gui(
            &mut widget_complete_2,
            events,
            &mut focus_manager,
            &canvas2,
        ) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        };
        FocusManager::default_start_focus_behavior(
            &mut focus_manager,
            events,
            "0",
            "2",
        );

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
        canvas0.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas1.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas2.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas0.clear();
        canvas1.clear();
        canvas2.clear();

        // DRAW
        match checkbox0.draw(&mut canvas0, &mut focus_manager) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        }
        match button1_border.draw(&mut canvas1, &mut focus_manager) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        }
        match widget_complete_2.draw(&mut canvas2, &mut focus_manager) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        }

        canvas0.present();
        canvas1.present();
        canvas2.present();
        false
    });
    std::process::ExitCode::SUCCESS
}
