use std::{cell::Cell, fs::File, io::Read, path::Path};

use rand::Rng;
use sdl2::pixels::Color;
use tiny_sdl2_gui::{
    layout::{horizontal_layout::HorizontalLayout, vertical_layout::VerticalLayout},
    util::{
        focus::{CircularUID, FocusManager, PRNGBytes, RefCircularUIDCell, UID},
        font::{FontManager, SingleLineTextRenderType, TextRenderer},
        length::{MaxLen, MaxLenPolicy},
    },
    widget::{
        button::{Button, DefaultButtonStyle},
        checkbox::{CheckBox, DefaultCheckBoxStyle},
        single_line_label::{DefaultSingleLineLabelState, SingleLineLabel},
        widget::{draw_gui, update_gui, SDLEvent},
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 300;
    const HEIGHT: u32 = 200;

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
    let sound_manager = Cell::new(Some(tiny_sdl2_gui::util::audio::SoundManager::new(std::time::Duration::from_secs(30))));

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

    let mut sdl =
        example_common::sdl_util::SDLSystems::new("shift tab! mouse!", (WIDTH, HEIGHT)).unwrap();

    // audio specific stuff. taking same values from rust-sdl2 examples
    #[cfg(feature = "sdl2-mixer")]
    let _audio = sdl.sdl_context.audio().unwrap();
    #[cfg(feature = "sdl2-mixer")]
    sdl2::mixer::open_audio(44_100, sdl2::mixer::AUDIO_S16LSB, sdl2::mixer::DEFAULT_CHANNELS, 1_024).unwrap();
    #[cfg(feature = "sdl2-mixer")]
    let _mixer_context = sdl2::mixer::init(sdl2::mixer::InitFlag::MP3).unwrap();
    #[cfg(feature = "sdl2-mixer")]
    sdl2::mixer::allocate_channels(16);

    let mut focus_manager = FocusManager::default();

    let button_text = DefaultSingleLineLabelState {
        inner: Cell::new("button".into()),
    };
    let button_label = SingleLineLabel::new(
        &button_text,
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
    );
    let button_style = DefaultButtonStyle {
        label: button_label,
    };

    let background_color = Cell::new(Color::BLACK);
    let mut layout = VerticalLayout::default();
    let mut top_layout = HorizontalLayout::default();
    let mut bottom_layout = HorizontalLayout::default();

    top_layout.max_w_policy =
        tiny_sdl2_gui::layout::vertical_layout::MajorAxisMaxLenPolicy::Together(
            MaxLenPolicy::Literal(MaxLen(0.)),
        );

    let mut rng = rand::thread_rng();

    let mut get_prng_bytes = || {
        let mut bytes = [0u8; 8];
        rng.fill(&mut bytes);
        PRNGBytes(bytes)
    };

    // step by step
    let checkbox0_focus_id = get_prng_bytes();
    let checkbox0_focus_id = UID::new(checkbox0_focus_id);
    let checkbox0_focus_id = CircularUID::new(checkbox0_focus_id);
    // checkbox borrows the focus id. since it's in a cell, modification can
    // still be made
    let checkbox0_focus_id = Cell::new(checkbox0_focus_id);

    #[cfg(feature = "sdl2-mixer")]
    let focus_press_sound_style = tiny_sdl2_gui::widget::checkbox::DefaultFocusPressWidgetSoundStyle {
        sound_manager: &sound_manager,
        focus_sound_path: Some(&focus_sound_path),
        press_sound_path: Some(&press_sound_path),
        release_sound_path: Default::default(),
    };
    #[cfg(not(feature = "sdl2-mixer"))]
    let focus_press_sound_style = tiny_sdl2_gui::widget::checkbox::EmptyFocusPressWidgetSoundStyle {};

    let mut checkbox0 = CheckBox::new(
        &check_states[0],
        RefCircularUIDCell(&checkbox0_focus_id),
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style.clone()),
        &sdl.texture_creator,
    );

    let binding = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));
    let mut checkbox1 = CheckBox::new(
        &check_states[1],
        *RefCircularUIDCell(&binding).set_after(&checkbox0.focus_id),
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style.clone()),
        &sdl.texture_creator,
    );

    let binding = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));
    let mut checkbox2 = CheckBox::new(
        &check_states[2],
        *RefCircularUIDCell(&binding).set_after(&checkbox1.focus_id),
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style.clone()),
        &sdl.texture_creator,
    );

    let binding = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));
    let mut checkbox3 = CheckBox::new(
        &check_states[3],
        *RefCircularUIDCell(&binding).set_after(&checkbox2.focus_id),
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style.clone()),
        &sdl.texture_creator,
    );

    let binding = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));
    let mut checkbox4 = CheckBox::new(
        &check_states[4],
        *RefCircularUIDCell(&binding).set_after(&checkbox3.focus_id),
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style.clone()),
        &sdl.texture_creator,
    );

    let checkbox5_focus_id = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));
    let mut checkbox5 = CheckBox::new(
        &check_states[5],
        *RefCircularUIDCell(&checkbox5_focus_id).set_after(&checkbox4.focus_id),
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style.clone()),
        &sdl.texture_creator,
    );

    let button_focus_id = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));
    let mut button = Button::new(
        Box::new(|| {
            println!("Clicked!!!");
            if background_color.get() == Color::BLACK {
                background_color.set(Color::RGB(0, 24, 64));
            } else {
                background_color.set(Color::BLACK);
            }
            Ok(())
        }),
        RefCircularUIDCell(&button_focus_id),
        Box::new(button_style),
        Box::new(focus_press_sound_style.clone()),
        &sdl.texture_creator,
    );

    top_layout.elems.push(&mut checkbox0);
    top_layout.elems.push(&mut checkbox1);
    top_layout.elems.push(&mut checkbox2);
    bottom_layout.elems.push(&mut checkbox3);
    bottom_layout.elems.push(&mut checkbox4);
    bottom_layout.elems.push(&mut checkbox5);
    layout.elems.push(&mut top_layout);
    layout.elems.push(&mut bottom_layout);
    layout.elems.push(&mut button);

    // testing modification after layout is constructed. note that layout
    // constructions mutably borrows the components. but we can still do the
    // modification of the focus ids! this allows elements to be added or
    // removed to/from the focus loop on the fly
    let mut button_focus_id_get = button_focus_id.get();
    let mut checkbox0_focus_id_get = checkbox0_focus_id.get();
    let mut checkbox5_focus_id_get = checkbox5_focus_id.get();
    button_focus_id_get
        .set_after(&mut checkbox5_focus_id_get)
        .set_before(&mut checkbox0_focus_id_get);
    button_focus_id.set(button_focus_id_get);
    checkbox0_focus_id.set(checkbox0_focus_id_get);
    checkbox5_focus_id.set(checkbox5_focus_id_get);

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
            match update_gui(
                &mut layout,
                &mut sdl.canvas,
                &mut events_accumulator,
                Some(&mut focus_manager),
            ) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            sdl.canvas.set_draw_color(background_color.get());
            sdl.canvas.clear();
            match draw_gui(
                &mut layout,
                &mut sdl.canvas,
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
                checkbox0_focus_id.get().uid(),
                button_focus_id.get().uid(),
            );

            // if unprocessed escape key
            for e in events_accumulator.iter_mut().filter(|e| e.available()) {
                match e.e {
                    sdl2::event::Event::KeyDown {
                        keycode: Some(sdl2::keyboard::Keycode::Escape),
                        repeat: false,
                        ..
                    } => {
                        e.set_consumed(); // intentional redundant
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
