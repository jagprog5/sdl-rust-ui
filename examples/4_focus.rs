use std::{cell::Cell, fs::File, io::Read, path::Path, time::Duration};

use example_common::gui_loop::gui_loop;
use sdl2::{mouse::MouseButton, pixels::Color};
use tiny_sdl2_gui::{
    layout::{horizontal_layout::HorizontalLayout, vertical_layout::VerticalLayout},
    util::{
        focus::{FocusID, FocusManager},
        font::{FontManager, SingleLineTextRenderType, TextRenderer},
        length::{MaxLen, MaxLenPolicy},
    },
    widget::{
        button::{Button, LabelButtonStyle},
        checkbox::{CheckBox, DefaultCheckBoxStyle},
        single_line_label::SingleLineLabel,
        update_gui, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 300;
    const HEIGHT: u32 = 200;
    const MAX_DELAY: Duration = Duration::from_millis(17);

    let check_states = (0..6).map(|_| Cell::<bool>::new(false)).collect::<Vec<_>>();

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
    #[cfg(feature = "sdl2-mixer")]
    let sound_manager = Cell::new(Some(tiny_sdl2_gui::util::audio::SoundManager::new(
        std::time::Duration::from_secs(30),
    )));

    #[cfg(feature = "sdl2-mixer")]
    let focus_sound_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("focus_sound.mp3");

    #[cfg(feature = "sdl2-mixer")]
    let press_sound_path = Path::new(".")
        .join("examples")
        .join("assets")
        .join("press_sound.mp3");

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    sdl_video_subsystem.text_input().start();
    let window = sdl_video_subsystem
        .window("shift tab! mouse!", WIDTH, HEIGHT)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // audio specific stuff. taking same values from rust-sdl2 examples
    #[cfg(feature = "sdl2-mixer")]
    let _audio = sdl_context.audio().unwrap();
    #[cfg(feature = "sdl2-mixer")]
    sdl2::mixer::open_audio(
        44_100,
        sdl2::mixer::AUDIO_S16LSB,
        sdl2::mixer::DEFAULT_CHANNELS,
        1_024,
    )
    .unwrap();
    #[cfg(feature = "sdl2-mixer")]
    let _mixer_context = sdl2::mixer::init(sdl2::mixer::InitFlag::MP3).unwrap();
    #[cfg(feature = "sdl2-mixer")]
    sdl2::mixer::allocate_channels(16);

    let mut focus_manager = FocusManager::default();

    let button_label = SingleLineLabel::new(
        "button".into(),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );
    let button_style = LabelButtonStyle {
        label: button_label,
    };

    let background_color = Cell::new(Color::BLACK);

    #[cfg(feature = "sdl2-mixer")]
    let focus_press_sound_style =
        tiny_sdl2_gui::widget::checkbox::DefaultFocusPressWidgetSoundStyle {
            sound_manager: &sound_manager,
            focus_sound_path: Some(&focus_sound_path),
            press_sound_path: Some(&press_sound_path),
            release_sound_path: Default::default(),
        };
    #[cfg(not(feature = "sdl2-mixer"))]
    let focus_press_sound_style =
        tiny_sdl2_gui::widget::checkbox::EmptyFocusPressWidgetSoundStyle {};

    let checkbox0 = CheckBox::new(
        &check_states[0],
        FocusID {
            previous: "button".to_owned(),
            me: "0".to_owned(),
            next: "1".to_owned(),
        },
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );

    let checkbox1 = CheckBox::new(
        &check_states[1],
        FocusID {
            previous: "0".to_owned(),
            me: "1".to_owned(),
            next: "2".to_owned(),
        },
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );

    let checkbox2 = CheckBox::new(
        &check_states[2],
        FocusID {
            previous: "1".to_owned(),
            me: "2".to_owned(),
            next: "3".to_owned(),
        },
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );

    let checkbox3 = CheckBox::new(
        &check_states[3],
        FocusID {
            previous: "2".to_owned(),
            me: "3".to_owned(),
            next: "4".to_owned(),
        },
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );

    let checkbox4 = CheckBox::new(
        &check_states[4],
        FocusID {
            previous: "3".to_owned(),
            me: "4".to_owned(),
            next: "5".to_owned(),
        },
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );

    let checkbox5 = CheckBox::new(
        &check_states[5],
        FocusID {
            previous: "4".to_owned(),
            me: "5".to_owned(),
            next: "button".to_owned(),
        },
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );

    let button = Button::new(
        Box::new(|| {
            println!("Clicked!!!");
            if background_color.get() == Color::BLACK {
                background_color.set(Color::RGB(0, 24, 64));
            } else {
                background_color.set(Color::BLACK);
            }
            Ok(())
        }),
        FocusID {
            previous: "5".to_owned(),
            me: "button".to_owned(),
            next: "0".to_owned(),
        },
        Box::new(button_style),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );

    let mut top_layout = HorizontalLayout::default();
    let mut bottom_layout = HorizontalLayout::default();
    let mut layout = VerticalLayout::default();

    top_layout.max_w_policy =
        tiny_sdl2_gui::layout::vertical_layout::MajorAxisMaxLenPolicy::Together(
            MaxLenPolicy::Literal(MaxLen(0.)),
        );

    top_layout.elems.push(Box::new(checkbox0));
    top_layout.elems.push(Box::new(checkbox1));
    top_layout.elems.push(Box::new(checkbox2));
    bottom_layout.elems.push(Box::new(checkbox3));
    bottom_layout.elems.push(Box::new(checkbox4));
    bottom_layout.elems.push(Box::new(checkbox5));
    layout.elems.push(Box::new(top_layout));
    layout.elems.push(Box::new(bottom_layout));
    layout.elems.push(Box::new(button));

    gui_loop(MAX_DELAY, &mut event_pump, |events| {
        // UPDATE
        match update_gui(
            &mut layout,
            events,
            &mut focus_manager,
            &canvas,
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
            "button",
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
        canvas.set_draw_color(background_color.get());
        canvas.clear();

        // DRAW
        match &mut layout.draw(&mut canvas, &mut focus_manager) {
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
