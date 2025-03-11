use std::{cell::Cell, fs::File, io::Read, path::Path, time::Duration};

use sdl2::{mouse::MouseButton, pixels::Color};
use tiny_sdl2_gui::{
    layout::{
        horizontal_layout::HorizontalLayout,
        scroller::{ScrollAspectRatioDirectionPolicy, Scroller, ScrollerSizingPolicy},
        vertical_layout::VerticalLayout,
    },
    util::{
        focus::{FocusID, FocusManager},
        font::{FontManager, SingleLineTextRenderType, TextRenderer},
        length::{
            AspectRatioPreferredDirection, MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLen,
            MinLenFailPolicy,
        }, rust::CellRefOrCell,
    },
    widget::{
        border::{Bevel, Border, Empty, Gradient},
        button::{Button, LabelButtonStyle},
        debug::CustomSizingControl,
        multi_line_label::{MultiLineLabel, MultiLineMinHeightFailPolicy},
        single_line_label::SingleLineLabel,
        single_line_text_input::{
            DefaultSingleLineEditStyle,
            SingleLineTextInput,
        },
        update_gui, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 300;
    const HEIGHT: u32 = 200;
    const MAX_DELAY: Duration = Duration::from_millis(17);

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
    sdl_video_subsystem.text_input().start();
    let window = sdl_video_subsystem
        .window("text input test", WIDTH, HEIGHT)
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

    let multiline_text = Cell::new("content will be displayed here".to_owned());
    let mut text_display = MultiLineLabel::new(
        CellRefOrCell::Ref(&multiline_text),
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
    let mut text_display = Scroller::new(false, true, &scroll_x, &scroll_y, Box::new(text_display));
    text_display.sizing_policy = ScrollerSizingPolicy::Custom(
        CustomSizingControl::default(),
        ScrollAspectRatioDirectionPolicy::Literal(AspectRatioPreferredDirection::HeightFromWidth),
    );

    let text_str = Cell::new("content".to_owned());

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
        FocusID {
            previous: "button".to_owned(),
            me: "text input".to_owned(),
            next: "button".to_owned(),
        },
        CellRefOrCell::Ref(&text_str),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );

    let text_entered_functionality = || {
        let text_content = text_str.take();
        if text_content.is_empty() {
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

    let mut enter_button_content = SingleLineLabel::new(
        CellRefOrCell::Cell(Cell::new("=>".into())),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );
    enter_button_content.min_h = MinLen(30.);
    enter_button_content.max_h = MaxLen(0.);

    let enter_button_style = LabelButtonStyle {
        label: enter_button_content,
    };

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

    let enter_button = Button::new(
        Box::new(text_entered_functionality),
        FocusID {
            previous: "text input".to_owned(),
            me: "button".to_owned(),
            next: "text input".to_owned(),
        },
        Box::new(enter_button_style),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );

    let text_input = Border::new(
        Box::new(text_input),
        &texture_creator,
        Box::new(Empty { width: 2 }),
    );

    let enter_buttom = Border::new(
        Box::new(enter_button),
        &texture_creator,
        Box::new(Empty { width: 2 }),
    );
    let enter_buttom = Border::new(
        Box::new(enter_buttom),
        &texture_creator,
        Box::new(Bevel::default()),
    );

    let mut bottom_layout = HorizontalLayout::default();

    bottom_layout.elems.push(Box::new(text_input));
    bottom_layout.elems.push(Box::new(enter_buttom));

    // the whole bottom part is as short as possible
    bottom_layout.max_h_policy = MaxLenPolicy::Literal(MaxLen(0.));

    let bottom_border = Border::new(
        Box::new(bottom_layout),
        &texture_creator,
        Box::new(Gradient::default()),
    );

    let mut layout = VerticalLayout::default();
    // update order should be reversed, as the multiline label widget relies on
    // the changes from the text input.
    //
    // doesn't really matter for this example
    layout.reverse = true;
    layout.min_w_fail_policy = MinLenFailPolicy::NEGATIVE;

    layout.elems.push(Box::new(text_display));
    layout.elems.push(Box::new(bottom_border));

    example_common::gui_loop::gui_loop(MAX_DELAY, &mut event_pump, |events| {
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
            "text input",
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
        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
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
