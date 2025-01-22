use std::{cell::Cell, fs::File, io::Read, path::Path};

use compact_str::CompactString;
use rand::Rng;
use sdl2::pixels::Color;
use tiny_sdl2_gui::{
    layout::{
        horizontal_layout::HorizontalLayout,
        scroller::{ScrollAspectRatioDirectionPolicy, Scroller, ScrollerSizingPolicy},
        vertical_layout::VerticalLayout,
    },
    util::{
        focus::{CircularUID, FocusManager, PRNGBytes, RefCircularUIDCell, UID},
        font::{FontManager, SingleLineTextRenderType, TextRenderer},
        length::{
            AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen, MinLenFailPolicy
        },
    },
    widget::{
        border::{Bevel, Border, Empty, Gradient},
        button::{Button, DefaultButtonStyle},
        debug::CustomSizingControl,
        multi_line_label::{MultiLineLabel, MultiLineMinHeightFailPolicy},
        single_line_label::SingleLineLabel,
        single_line_text_input::{
            DefaultSingleLineEditStyle, DefaultSingleLineTextEditState, SingleLineTextEditState,
            SingleLineTextInput,
        },
        widget::{draw_gui, update_gui, SDLEvent},
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 300;
    const HEIGHT: u32 = 200;

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

    #[cfg(feature = "sdl2-mixer")]
    let text_input_sound = Path::new(".")
        .join("examples")
        .join("assets")
        .join("text_input_sound.mp3");

    #[cfg(feature = "sdl2-mixer")]
    let sound_manager = Cell::new(Some(tiny_sdl2_gui::util::audio::SoundManager::new(
        std::time::Duration::from_secs(30),
    )));

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    let window = sdl_video_subsystem
        .window("text input test", WIDTH, HEIGHT)
        .resizable()
        .position_centered()
        .build().unwrap();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build().unwrap();
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
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

    let mut rng = rand::thread_rng();
    let mut get_prng_bytes = || {
        let mut bytes = [0u8; 8];
        rng.fill(&mut bytes);
        PRNGBytes(bytes)
    };

    let text_input_focus_id = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));

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

    let mut layout = VerticalLayout::default();
    // update order should be reversed, as the multiline label widget relies on
    // the changes from the text input.
    //
    // doesn't really matter for this example
    layout.reverse = true;
    layout.min_w_fail_policy = MinLenFailPolicy::NEGATIVE;

    let multiline_text = Cell::new("content will be displayed here".to_owned());
    let mut text_display = MultiLineLabel::new(
        &multiline_text,
        20,
        Color::WHITE,
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );
    // put at the bottom and cut off the top if too small
    text_display.max_h_policy = MaxLenFailPolicy::POSITIVE;
    text_display.min_h_policy =
        MultiLineMinHeightFailPolicy::None(MinLenFailPolicy::NEGATIVE, MaxLenFailPolicy::POSITIVE);

    let scroll_x: Cell<i32> = Default::default();
    let scroll_y: Cell<i32> = Default::default();
    let mut text_display = Scroller::new(false, true, &scroll_x, &scroll_y, &mut text_display);
    text_display.sizing_policy = ScrollerSizingPolicy::Custom(
        CustomSizingControl::default(),
        ScrollAspectRatioDirectionPolicy::Literal(AspectRatioPreferredDirection::HeightFromWidth),
    );

    layout.elems.push(&mut text_display);

    let mut bottom_layout = HorizontalLayout::default();
    let text_str = DefaultSingleLineTextEditState {
        inner: CompactString::from("content").into(),
    };

    #[cfg(feature = "sdl2-mixer")]
    let text_input_sound_style =
        tiny_sdl2_gui::widget::single_line_text_input::DefaultSingleLineTextInputSoundStyle {
            sound_manager: &sound_manager,
            focus_sound_path: Some(&focus_sound_path),
            text_added_sound_path: Some(&text_input_sound),
            text_removed_sound_path: Some(&text_input_sound),
            enter_sound_path: Some(&press_sound_path),
        };
    #[cfg(not(feature = "sdl2-mixer"))]
    let text_input_sound_style =
        tiny_sdl2_gui::widget::single_line_text_input::EmptySingleLineTextInputSoundStyle {};

    let mut text_input = SingleLineTextInput::new(
        Box::new(|| Ok(())), // replaced below
        Box::new(DefaultSingleLineEditStyle::default()),
        Box::new(text_input_sound_style),
        RefCircularUIDCell(&text_input_focus_id),
        &text_str,
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );

    let text_entered_functionality = || {
        let text_content = text_str.get();
        if text_content.len() == 0 {
            return Ok(());
        }
        text_str.set("".into());
        scroll_y.set(0);

        let mut multiline_content = multiline_text.take();
        multiline_content += "\n";
        multiline_content += &text_content;
        multiline_text.set(multiline_content);
        Ok(())
    };

    text_input.functionality = Box::new(text_entered_functionality);

    let binding = CompactString::from("=>");
    let mut enter_button_content = SingleLineLabel::new(
        &binding,
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );
    enter_button_content.min_h = MinLen(30.);
    enter_button_content.max_h = MaxLen(0.);

    let enter_button_style = DefaultButtonStyle {
        label: enter_button_content,
    };

    let enter_button_focus_id = Cell::new(CircularUID::new(UID::new(get_prng_bytes())));

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

    let mut enter_button = Button::new(
        Box::new(|| text_entered_functionality()),
        *RefCircularUIDCell(&enter_button_focus_id)
            .set_after(&text_input.focus_id)
            .set_before(&text_input.focus_id),
        Box::new(enter_button_style),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );
    enter_button.focus_id.set_after(&mut text_input.focus_id);
    enter_button.focus_id.set_before(&mut text_input.focus_id);

    let mut text_input = Border::new(
        &mut text_input,
        &texture_creator,
        Box::new(Empty { width: 2 }),
    );

    let mut enter_buttom = Border::new(
        &mut enter_button,
        &texture_creator,
        Box::new(Empty { width: 2 }),
    );
    let mut enter_buttom = Border::new(
        &mut enter_buttom,
        &texture_creator,
        Box::new(Bevel::default()),
    );

    bottom_layout.elems.push(&mut text_input);
    bottom_layout.elems.push(&mut enter_buttom);

    // the whole bottom part is as short as possible
    bottom_layout.max_h_policy = MaxLenPolicy::Literal(MaxLen(0.));

    let mut bottom_border = Border::new(
        &mut bottom_layout,
        &texture_creator,
        Box::new(Gradient::default()),
    );

    layout.elems.push(&mut bottom_border);

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
            match update_gui(
                &mut layout,
                &mut canvas,
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
                text_input_focus_id.get().uid(),
                enter_button_focus_id.get().uid(),
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
            
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            match draw_gui(
                &mut layout,
                &mut canvas,
                &mut events_accumulator,
                Some(&mut focus_manager),
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
