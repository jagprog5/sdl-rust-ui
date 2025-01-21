use std::{cell::Cell, fs::File, io::Read, path::Path};

use compact_str::CompactString;
use sdl2::pixels::Color;
use tiny_sdl2_gui::{
    layout::{horizontal_layout::HorizontalLayout, vertical_layout::VerticalLayout},
    util::{
        font::{FontManager, SingleLineTextRenderType, TextRenderer},
        length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy},
    },
    widget::{
        background::BackgroundSizingPolicy,
        debug::CustomSizingControl,
        multi_line_label::{MultiLineLabel, MultiLineMinHeightFailPolicy},
        single_line_label::{DefaultSingleLineLabelState, SingleLineLabel},
        texture::AspectRatioFailPolicy,
        widget::{draw_gui, update_gui, SDLEvent},
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let mut sdl = example_common::sdl_util::SDLSystems::new("labels", (WIDTH, HEIGHT)).unwrap();
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

    // ====================== TOP LABEL ========================================

    let top_label_text = DefaultSingleLineLabelState {
        inner: Cell::new("hello".into()),
    };
    let mut top_label = SingleLineLabel::new(
        &top_label_text,
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
    );

    top_label.min_h_fail_policy = MinLenFailPolicy::NEGATIVE; // go up if too small
    top_label.min_h = MinLen(50.); // for testing
    top_label.max_h = MaxLen(150.);

    // right align in vertical layout
    top_label.max_w_fail_policy = MaxLenFailPolicy::POSITIVE;
    top_label.min_w_fail_policy = MinLenFailPolicy::NEGATIVE;

    // ====================== MIDDLE LABEL =====================================

    let middle_label_text = CompactString::from("the quick brown fox");
    let mut middle_label = SingleLineLabel::new(
        &middle_label_text,
        SingleLineTextRenderType::Shaded(Color::WHITE, Color::GRAY),
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
    );
    middle_label.request_aspect_ratio = false;

    // ======================== BOTTOM LABELS ==================================

    let bottom_left_label_text = CompactString::from("horizontal");
    let mut bottom_left_label = SingleLineLabel::new(
        &bottom_left_label_text,
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
    );

    let bottom_right_label_text = CompactString::from("horizontal2q|");
    let mut bottom_right_label = SingleLineLabel::new(
        &bottom_right_label_text,
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
    );
    bottom_right_label.min_h_fail_policy = MinLenFailPolicy::NEGATIVE;
    bottom_right_label.min_h = MinLen(50.); // for testing
    bottom_right_label.max_h = MaxLen(100.);
    // right align + varying size in horizontal layout is a bit more tricky
    bottom_right_label.max_w_fail_policy = MaxLenFailPolicy::POSITIVE;
    bottom_right_label.min_w_fail_policy = MinLenFailPolicy::NEGATIVE;
    bottom_right_label.aspect_ratio_fail_policy = AspectRatioFailPolicy::ZoomOut((1., 0.5));

    let multiline_string_displayed = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_owned();
    let mut multiline_widget = MultiLineLabel::new(
        &multiline_string_displayed,
        20,
        Color::WHITE,
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
    );
    multiline_widget.min_h_policy = MultiLineMinHeightFailPolicy::CutOff(1.0);
    multiline_widget.max_h_policy = MaxLenFailPolicy::NEGATIVE;

    // ======================== BUILD GUI ======================================

    let mut layout = VerticalLayout::default();
    let mut bottom_layout = HorizontalLayout::default();

    #[cfg(feature = "noise")]
    let mut rng = rand::thread_rng();
    #[cfg(feature = "noise")]
    let random_number: u32 = rand::Rng::gen(&mut rng);
    #[cfg(feature = "noise")]
    let mut top = tiny_sdl2_gui::widget::background::SoftwareRenderBackground::new(
        &mut top_label,
        tiny_sdl2_gui::widget::background::Wood::new(random_number),
        &sdl.texture_creator,
    );
    #[cfg(feature = "noise")]
    top.set_color_mod((200, 200, 200)); // dim a bit

    #[cfg(not(feature = "noise"))]
    let mut top = tiny_sdl2_gui::widget::background::SolidColorBackground {
        color: Color::RGB(255, 127, 80),
        contained: &mut top_label,
        sizing_policy: Default::default(),
    };

    top.sizing_policy = BackgroundSizingPolicy::Custom(CustomSizingControl::default()); // expand

    layout.elems.push(&mut top);
    layout.elems.push(&mut middle_label);
    layout.elems.push(&mut multiline_widget);
    bottom_layout.elems.push(&mut bottom_left_label);
    bottom_layout.elems.push(&mut bottom_right_label);
    layout.elems.push(&mut bottom_layout);

    let mut events_accumulator: Vec<SDLEvent> = Vec::new();
    'running: loop {
        let label_str = format!("{:?}", sdl.canvas.output_size().unwrap());
        let label_str = CompactString::from(label_str);
        top_label_text.inner.set(label_str);

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
            match update_gui(&mut layout, &mut sdl.canvas, &mut events_accumulator, None) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            sdl.canvas.set_draw_color(sdl2::pixels::Color::BLACK);
            sdl.canvas.clear();
            match draw_gui(&mut layout, &mut sdl.canvas, &mut events_accumulator, None) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
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
            sdl.canvas.present();
        }

        // steady loop of 60 (nothing fancier is needed)
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
    std::process::ExitCode::SUCCESS
}
