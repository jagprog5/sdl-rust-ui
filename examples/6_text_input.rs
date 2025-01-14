use std::{cell::Cell, fs::File, io::Read, path::Path};

use compact_str::CompactString;
use sdl2::pixels::Color;
use tiny_sdl2_gui::{
    layout::{
        horizontal_layout::HorizontalLayout,
        scroller::{ScrollAspectRatioDirectionPolicy, Scroller, ScrollerSizingPolicy},
        vertical_layout::VerticalLayout,
    },
    util::{
        focus::FocusManager,
        font::{FontManager, SingleLineTextRenderType, TextRenderer},
        length::{
            AspectRatioPreferredDirection, MaxLenFailPolicy, MinLen,
            MinLenFailPolicy, PreferredPortion,
        },
    },
    widget::{
        border::{Bevel, Border, Empty, Gradient},
        button::{Button, DefaultButtonStyle},
        debug::CustomSizingControl,
        multi_line_label::{MultiLineLabel, MultiLineMinHeightFailPolicy},
        single_line_label::SingleLineLabel,
        single_line_text_input::{DefaultSingleLineEditStyle, DefaultSingleLineTextEditState, SingleLineTextEditState, SingleLineTextInput},
        widget::{draw_gui, update_gui, SDLEvent},
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 300;
    const HEIGHT: u32 = 200;

    let mut sdl =
        example_common::sdl_util::SDLSystems::new("text input test", (WIDTH, HEIGHT)).unwrap();
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

    let font_manager = Cell::new(Some(
        FontManager::new(&ttf_context, &font_file_contents).unwrap(),
    ));

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
        &sdl.texture_creator,
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

    let mut text_input = SingleLineTextInput::new(
        Box::new(|| Ok(())), // replaced below
        Box::new(DefaultSingleLineEditStyle::default()),
        focus_manager.next_available_id(),
        &text_str,
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
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

    let mut text_input = Border::new(
        &mut text_input,
        &sdl.texture_creator,
        Box::new(Empty { width: 2 }),
    );

    let binding = CompactString::from("=>");
    let mut enter_button_content = SingleLineLabel::new(
        &binding,
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
    );
    enter_button_content.min_h = MinLen(30.);
    enter_button_content.preferred_w = PreferredPortion::EMPTY;

    let enter_button_style = DefaultButtonStyle {
        label: enter_button_content,
    };

    let mut enter_button = Button::new(
        Box::new(|| text_entered_functionality()),
        focus_manager.next_available_id(),
        Box::new(enter_button_style),
        &sdl.texture_creator,
    );
    let mut enter_buttom = Border::new(
        &mut enter_button,
        &sdl.texture_creator,
        Box::new(Empty { width: 2 }),
    );
    let mut enter_buttom = Border::new(
        &mut enter_buttom,
        &sdl.texture_creator,
        Box::new(Bevel::default()),
    );

    bottom_layout.elems.push(&mut text_input);
    bottom_layout.elems.push(&mut enter_buttom);

    // the whole bottom part is as short as possible
    bottom_layout.preferred_h = PreferredPortion::EMPTY;

    let mut bottom_border = Border::new(
        &mut bottom_layout,
        &sdl.texture_creator,
        Box::new(Gradient::default()),
    );

    layout.elems.push(&mut bottom_border);

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
            sdl.canvas.set_draw_color(Color::BLACK);
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
            FocusManager::default_start_focus_behavior(&mut focus_manager, &mut events_accumulator);
            for e in events_accumulator.iter_mut().filter(|e| e.available()) {
                match e.e {
                    sdl2::event::Event::KeyDown {
                        keycode: Some(sdl2::keyboard::Keycode::Escape),
                        repeat: false,
                        ..
                    } => {
                        // if unprocessed escape key
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
