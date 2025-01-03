
use std::cell::Cell;

use sdl2::pixels::Color;
use tiny_sdl2_gui::{layout::{scroller::Scroller, stacked_layout::{StackedLayout, StackedLayoutLiteralSizing, StackedLayoutSizingPolicy}}, util::{focus::FocusManager, length::PreferredPortion}, widget::{background::{Smooth, SoftwareRenderBackground, SolidColorBackground}, border::{Border, Empty, Gradient, Line}, checkbox::{CheckBox, DefaultCheckBoxStyle}, widget::{draw_gui, update_gui, SDLEvent}}};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 300;
    const HEIGHT: u32 = 200;

    let mut sdl = example_common::sdl_util::SDLSystems::new("nested scroller test", (WIDTH, HEIGHT)).unwrap();
    let mut focus_manager = FocusManager::default();

    let checkbox_state = Cell::new(false);
    
    // there is a checkbox
    let mut checkbox0 = CheckBox::new(&checkbox_state, focus_manager.next_available_id(), Box::new(DefaultCheckBoxStyle::default()), &sdl.texture_creator);

    // pad the checkbox a little bit for clarity
    let mut checkbox_border1 = Border::new(&mut checkbox0, &sdl.texture_creator, Box::new(Empty{ width: 5} ));

    // contain the checkbox and padding in a border
    let mut checkbox_border2 = Border::new(&mut checkbox_border1, &sdl.texture_creator, Box::new(Line::default()));

    // the checkbox + padding + border is contained in a scroll area. the scroll
    // area has a sizing which is a bit smaller than the parent
    //
    // padding
    let mut checkbox_stacked_layout3 = StackedLayout::default();
    checkbox_stacked_layout3.elems.push(&mut checkbox_border2);
    let mut sizing = StackedLayoutLiteralSizing::default();
    sizing.preferred_w = PreferredPortion(0.8);
    sizing.preferred_h = PreferredPortion(0.8);
    checkbox_stacked_layout3.sizing_policy = StackedLayoutSizingPolicy::Literal(sizing);
    // scroll area
    let inner_scroll_x = Cell::new(0i32);
    let inner_scroll_y = Cell::new(0i32);
    let mut inner_scroller4 = Scroller::new(true, true, &inner_scroll_x, &inner_scroll_y, &mut checkbox_stacked_layout3);

    // contain all of the above in a border
    let mut inner_content_border5 = Border::new(&mut inner_scroller4, &sdl.texture_creator, Box::new(Gradient::default()));
    
    // pad the above
    let mut stacked_layout_6 = StackedLayout::default();
    stacked_layout_6.elems.push(&mut inner_content_border5);
    stacked_layout_6.sizing_policy = StackedLayoutSizingPolicy::Literal(StackedLayoutLiteralSizing::default());

    // place the above padded in something with a black background
    let mut stacked_layout_7 = StackedLayout::default();
    let mut binding = SolidColorBackground { color: Color::BLACK };
    stacked_layout_7.elems.push(&mut binding);
    stacked_layout_7.elems.push(&mut stacked_layout_6);

    // contain all of the above in a scroll area
    let outer_scroll_x = Cell::new(0i32);
    let outer_scroll_y = Cell::new(0i32);
    let mut outer_scroller8 = Scroller::new(true, true, &outer_scroll_x, &outer_scroll_y, &mut stacked_layout_7);

    // contains all of the above in something which gives a black background and
    // make is a bit smaller than the parent
    let mut outer_scroller_with_background = StackedLayout::default();
    let mut binding = SolidColorBackground { color: Color::BLACK };
    outer_scroller_with_background.elems.push(&mut binding);
    outer_scroller_with_background.elems.push(&mut outer_scroller8);
    let mut sizing = StackedLayoutLiteralSizing::default();
    sizing.preferred_w = PreferredPortion(0.8);
    sizing.preferred_h = PreferredPortion(0.8);
    outer_scroller_with_background.sizing_policy = StackedLayoutSizingPolicy::Literal(sizing);

    let mut outer_scroller = Border::new(&mut outer_scroller_with_background, &sdl.texture_creator, Box::new(Gradient::default()));
    
    let mut outer_stacked = StackedLayout::default();
    outer_stacked.sizing_policy = StackedLayoutSizingPolicy::Literal(StackedLayoutLiteralSizing::default());
    let mut binding = SoftwareRenderBackground::new(Smooth::fast(0), &sdl.texture_creator);
    outer_stacked.elems.push(&mut binding);
    outer_stacked.elems.push(&mut outer_scroller);
    
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
            match update_gui(&mut outer_stacked, &mut sdl.canvas, &mut events_accumulator, Some(&mut focus_manager)) {
                Ok(()) => {}
                Err(msg) => {
                    debug_assert!(false, "{}", msg); // infallible in prod
                }
            }
            sdl.canvas.set_draw_color(Color::BLACK);
            sdl.canvas.clear();
            match draw_gui(&mut outer_stacked, &mut sdl.canvas, &mut events_accumulator, Some(&mut focus_manager)) {
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
