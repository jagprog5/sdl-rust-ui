use std::{cell::Cell, fs::File, io::Read, path::Path, time::Duration};

use example_common::gui_loop::gui_loop;
use sdl2::{mouse::MouseButton, pixels::Color};
use tiny_sdl2_gui::{
    layout::{horizontal_layout::HorizontalLayout, vertical_layout::VerticalLayout},
    util::{
        focus::FocusManager, font::{FontManager, SingleLineTextRenderType, TextRenderer}, length::{MaxLen, MaxLenFailPolicy, MinLen, MinLenFailPolicy}, rust::CellRefOrCell
    },
    widget::{
        background::BackgroundSizingPolicy,
        debug::CustomSizingControl,
        multi_line_label::{MultiLineLabel, MultiLineMinHeightFailPolicy},
        single_line_label::SingleLineLabel,
        texture::AspectRatioFailPolicy,
        update_gui, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;
    const MAX_DELAY: Duration = Duration::from_millis(17);

    let mut focus_manager = FocusManager::default();

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    let window = sdl_video_subsystem
        .window("labels", WIDTH, HEIGHT)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let ttf_context = sdl2::ttf::init().unwrap();

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

    let top_label_text = Cell::new("hello".to_owned());

    let mut top_label = SingleLineLabel::new(
        CellRefOrCell::Ref(&top_label_text),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );

    top_label.min_h_fail_policy = MinLenFailPolicy::NEGATIVE; // go up if too small
    top_label.min_h = MinLen(50.); // for testing
    top_label.max_h = MaxLen(150.);

    // right align in vertical layout
    top_label.max_w_fail_policy = MaxLenFailPolicy::POSITIVE;
    top_label.min_w_fail_policy = MinLenFailPolicy::NEGATIVE;

    // ====================== MIDDLE LABEL =====================================

    let mut middle_label = SingleLineLabel::new(
        "the quick brown fox".into(),
        SingleLineTextRenderType::Shaded(Color::WHITE, Color::GRAY),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );
    middle_label.request_aspect_ratio = false;

    // ======================== BOTTOM LABELS ==================================

    let bottom_left_label = SingleLineLabel::new(
        CellRefOrCell::from(Cell::new("horizontal".to_owned())),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );

    let mut bottom_right_label = SingleLineLabel::new(
        CellRefOrCell::from(Cell::new("horizontal2q|".to_owned())),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
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
        multiline_string_displayed.into(),
        20,
        Color::WHITE,
        Box::new(TextRenderer::new(&font_manager)),
        &texture_creator,
    );
    multiline_widget.min_h_policy = MultiLineMinHeightFailPolicy::CutOff(1.0);
    multiline_widget.max_h_policy = MaxLenFailPolicy::NEGATIVE;

    // ======================== BUILD GUI ======================================

    #[cfg(feature = "noise")]
    let mut rng = rand::thread_rng();
    #[cfg(feature = "noise")]
    let random_number: u32 = rand::Rng::gen(&mut rng);
    #[cfg(feature = "noise")]
    let mut top = tiny_sdl2_gui::widget::background::SoftwareRenderBackground::new(
        &mut top_label,
        tiny_sdl2_gui::widget::background::Wood::new(random_number),
        &texture_creator,
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

    let mut bottom_layout = HorizontalLayout::default();
    let mut layout = VerticalLayout::default();
    layout.elems.push(Box::new(top));
    layout.elems.push(Box::new(middle_label));
    layout.elems.push(Box::new(multiline_widget));
    bottom_layout.elems.push(Box::new(bottom_left_label));
    bottom_layout.elems.push(Box::new(bottom_right_label));
    layout.elems.push(Box::new(bottom_layout));

    gui_loop(MAX_DELAY, &mut event_pump, |events| {
        // UPDATE
        top_label_text.set(format!("{:?}", canvas.output_size().unwrap()));
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
        match &mut layout.draw(&mut canvas, &focus_manager) {
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
