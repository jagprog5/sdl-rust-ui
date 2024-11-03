use std::{cell::Cell, fs::File, io::Read, path::Path};

use compact_str::CompactString;
use rand::Rng;
use sdl2::pixels::Color;
use tiny_sdl2_gui::{layout::{horizontal_layout::HorizontalLayout, stacked_layout::{StackedLayout, StackedLayoutLiteralSizing, StackedLayoutSizingPolicy}, vertical_layout::VerticalLayout}, util::{font::{FontManager, TextRenderType, TextRenderer}, length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy, PreferredPortion}}, widget::{background::{SoftwareRenderBackground, Wood}, label::{DefaultLabelState, Label}, texture::AspectRatioFailPolicy, widget::{draw_gui, update_gui, SDLEvent}}};


#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 200;

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

    let font_manager = Cell::new(Some(
        FontManager::new(&ttf_context, &font_file_contents).unwrap(),
    ));

    // ====================== TOP LABEL ========================================

    let top_label_text = DefaultLabelState {
        inner: Cell::new("hello".into()),
    };
    let mut top_label = Label::new(
        &top_label_text,
        TextRenderType::Blended(Color::WHITE),
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
    let mut middle_label = Label::new(
        &middle_label_text,
        TextRenderType::Shaded(Color::WHITE, Color::GRAY),
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
    );
    middle_label.request_aspect_ratio = false;

    // ======================== BOTTOM LABELS ==================================
    
    let bottom_left_label_text = CompactString::from("horizontal");
    let bottom_left_label = Label::new(
        &bottom_left_label_text,
        TextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &sdl.texture_creator,
    );

    let bottom_right_label_text = CompactString::from("horizontal2q|");
    let mut bottom_right_label = Label::new(
        &bottom_right_label_text,
        TextRenderType::Blended(Color::WHITE),
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

    // ======================== BUILD GUI ======================================

    let mut layout = VerticalLayout::default();
    let mut bottom_layout = HorizontalLayout::default();
    bottom_layout.preferred_h =  PreferredPortion::EMPTY;
    let mut top_layout = StackedLayout::default();
    top_layout.sizing_policy = StackedLayoutSizingPolicy::Literal(StackedLayoutLiteralSizing::default());

    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen();
    #[cfg(feature = "noise")]
    {
        let mut noise_background = SoftwareRenderBackground::new(Wood::new(random_number), &sdl.texture_creator);
        noise_background.set_color_mod((200, 200, 200)); // dim a bit
        top_layout.elems.push(Box::new(noise_background));

    }
    top_layout.elems.push(Box::new(top_label));
    layout.elems.push(Box::new(top_layout));
    layout.elems.push(Box::new(middle_label));

    bottom_layout.elems.push(Box::new(bottom_left_label));
    bottom_layout.elems.push(Box::new(bottom_right_label));
    layout.elems.push(Box::new(bottom_layout));

    let mut events_accumulator: Vec<SDLEvent> = Vec::new();
    'running: loop {
        let label_str = format!("{:?}", sdl.canvas.output_size().unwrap());
        let label_str = CompactString::from(label_str);
        top_label_text.inner.set(label_str);

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
            events_accumulator.clear(); // clear after use
            sdl.canvas.present();
        }

        // steady loop of 60 (nothing fancier is needed)
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
    std::process::ExitCode::SUCCESS
}
