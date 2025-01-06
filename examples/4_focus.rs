
use std::{cell::Cell, fs::File, io::Read, path::Path};

use sdl2::pixels::Color;
use tiny_sdl2_gui::{layout::{horizontal_layout::HorizontalLayout, vertical_layout::VerticalLayout}, util::{focus::FocusManager, font::{FontManager, SingleLineTextRenderType, TextRenderer}, length::{MaxLen, MaxLenPolicy}}, widget::{button::{Button, DefaultButtonStyle}, checkbox::{CheckBox, DefaultCheckBoxStyle}, single_line_label::{DefaultSingleLineLabelState, SingleLineLabel}, widget::{draw_gui, update_gui, SDLEvent}}};

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

    let font_manager = Cell::new(Some(
        FontManager::new(&ttf_context, &font_file_contents).unwrap(),
    ));

    let mut sdl = example_common::sdl_util::SDLSystems::new("shift tab! mouse!", (WIDTH, HEIGHT)).unwrap();
    let mut focus_manager = FocusManager::default();

    let button_text = DefaultSingleLineLabelState{ inner: Cell::new("button".into()) };
    let button_label = SingleLineLabel::new(&button_text, SingleLineTextRenderType::Blended(Color::WHITE), Box::new(TextRenderer::new(&font_manager)), &sdl.texture_creator);
    let button_style = DefaultButtonStyle {
        label: button_label,
    };

    let background_color = Cell::new(Color::BLACK);
    let mut layout = VerticalLayout::default();

    let mut top_layout = HorizontalLayout::default();
    top_layout.max_w_policy = tiny_sdl2_gui::layout::vertical_layout::MajorAxisMaxLenPolicy::Together(MaxLenPolicy::Literal(MaxLen(0.)));

    let mut binding = CheckBox::new(&check_states[0], focus_manager.next_available_id(), Box::new(DefaultCheckBoxStyle{}), &sdl.texture_creator);
    top_layout.elems.push(&mut binding);
    let mut binding = CheckBox::new(&check_states[1], focus_manager.next_available_id(), Box::new(DefaultCheckBoxStyle{}), &sdl.texture_creator);
    top_layout.elems.push(&mut binding);
    let mut binding = CheckBox::new(&check_states[2], focus_manager.next_available_id(), Box::new(DefaultCheckBoxStyle{}), &sdl.texture_creator);
    top_layout.elems.push(&mut binding);

    let mut bottom_layout = HorizontalLayout::default();
    let mut binding = CheckBox::new(&check_states[3], focus_manager.next_available_id(), Box::new(DefaultCheckBoxStyle{}), &sdl.texture_creator);
    bottom_layout.elems.push(&mut binding);
    let mut binding = CheckBox::new(&check_states[4], focus_manager.next_available_id(), Box::new(DefaultCheckBoxStyle{}), &sdl.texture_creator);
    bottom_layout.elems.push(&mut binding);
    let mut binding = CheckBox::new(&check_states[5], focus_manager.next_available_id(), Box::new(DefaultCheckBoxStyle{}), &sdl.texture_creator);
    bottom_layout.elems.push(&mut binding);

    layout.elems.push(&mut top_layout);
    layout.elems.push(&mut bottom_layout);

    let mut button = Button::new(Box::new(|| {
        println!("Clicked!!!");
        if background_color.get() == Color::BLACK {
            background_color.set(Color::RGB(0, 24, 64));
        } else {
            background_color.set(Color::BLACK);
        }
        Ok(())
    }), focus_manager.next_available_id(), Box::new(button_style), &sdl.texture_creator);

    layout.elems.push(&mut button);

    let mut events_accumulator: Vec<SDLEvent> = Vec::new();
    'running: loop {
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
            match update_gui(&mut layout, &mut sdl.canvas, &mut events_accumulator, Some(&mut focus_manager)) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            sdl.canvas.set_draw_color(background_color.get());
            sdl.canvas.clear();
            match draw_gui(&mut layout, &mut sdl.canvas, &mut events_accumulator, Some(&mut focus_manager)) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            FocusManager::default_start_focus_behavior(&mut focus_manager, &mut events_accumulator);
            events_accumulator.clear(); // clear after use
            sdl.canvas.present();
        }

        // steady loop of 60 (nothing fancier is needed)
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
    std::process::ExitCode::SUCCESS
}
