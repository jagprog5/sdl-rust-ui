use std::{cell::Cell, time::Duration};

use sdl2::{mouse::MouseButton, pixels::Color};
use tiny_sdl2_gui::{
    layout::scroller::{Scroller, ScrollerSizingPolicy},
    util::{
        focus::{FocusID, FocusManager},
        length::PreferredPortion,
    },
    widget::{
        background::{BackgroundSizingPolicy, SolidColorBackground},
        border::{Bevel, Border, Empty, Gradient, Line},
        checkbox::{CheckBox, DefaultCheckBoxStyle},
        debug::CustomSizingControl,
        update_gui, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 300;
    const HEIGHT: u32 = 200;
    const MAX_DELAY: Duration = Duration::from_millis(17);

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    sdl_video_subsystem.text_input().start();
    let window = sdl_video_subsystem
        .window("nested scroller test", WIDTH, HEIGHT)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut focus_manager = FocusManager::default();

    let checkbox_state = Cell::new(false);

    let focus_press_sound_style =
        tiny_sdl2_gui::widget::checkbox::EmptyFocusPressWidgetSoundStyle {};

    // there is a checkbox. it is the only element
    let checkbox = CheckBox::new(
        &checkbox_state,
        FocusID {
            previous: "focus".to_owned(),
            me: "focus".to_owned(),
            next: "focus".to_owned(),
        },
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );

    // pad the checkbox a little bit for clarity
    let checkbox_border1 = Border::new(
        Box::new(checkbox),
        &texture_creator,
        Box::new(Empty { width: 5 }),
    );

    // contain the checkbox and padding in a border
    let mut checkbox_border2 = Border::new(
        Box::new(checkbox_border1),
        &texture_creator,
        Box::new(Line::default()),
    );

    // the checkbox + padding + border is contained in a scroll area. the scroll
    // area has a sizing which is a bit smaller than the parent (it ignores the
    // sizing of the contained)
    let inner_scroll_x = Cell::new(0i32);
    let inner_scroll_y = Cell::new(0i32);
    let mut inner_scroller4 = Scroller::new(
        true,
        true,
        &inner_scroll_x,
        &inner_scroll_y,
        Box::new(checkbox_border2),
    );
    let mut sizing = CustomSizingControl::default();
    sizing.preferred_w = PreferredPortion(0.8);
    sizing.preferred_h = PreferredPortion(0.8);
    inner_scroller4.sizing_policy = ScrollerSizingPolicy::Custom(sizing, Default::default());

    // contain all of the above in a border
    let inner_content_border5 = Border::new(
        Box::new(inner_scroller4),
        &texture_creator,
        Box::new(Gradient::default()),
    );

    // contain all of the above in a scroll area
    let outer_scroll_x = Cell::new(0i32);
    let outer_scroll_y = Cell::new(0i32);
    let mut outer_scroller6 = Scroller::new(
        true,
        true,
        &outer_scroll_x,
        &outer_scroll_y,
        Box::new(inner_content_border5),
    );
    outer_scroller6.sizing_policy = ScrollerSizingPolicy::Custom(sizing, Default::default());
    outer_scroller6.restrict_scroll = false;

    // contain all of the above in a border
    let mut outer_content_border7 = Border::new(
        Box::new(outer_scroller6),
        &texture_creator,
        Box::new(Bevel::default()),
    );

    let mut content_background8 = SolidColorBackground::new(
        Color::BLACK,
        &mut outer_content_border7,
        BackgroundSizingPolicy::Children,
    );

    #[cfg(feature = "noise")]
    let mut content_background9 = tiny_sdl2_gui::widget::background::SoftwareRenderBackground::new(
        &mut content_background8,
        tiny_sdl2_gui::widget::background::Smooth::fast(0),
        &texture_creator,
    );

    #[cfg(not(feature = "noise"))]
    let mut content_background9 = tiny_sdl2_gui::widget::background::SolidColorBackground {
        color: Color::RGB(100, 100, 100),
        contained: &mut content_background8,
        sizing_policy: Default::default(),
    };

    content_background9.sizing_policy =
        BackgroundSizingPolicy::Custom(CustomSizingControl::default());

    example_common::gui_loop::gui_loop(MAX_DELAY, &mut event_pump, |events| {
        // UPDATE
        match update_gui(
            &mut content_background9,
            events,
            &mut focus_manager,
            &canvas,
        ) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        };

        FocusManager::default_start_focus_behavior(&mut focus_manager, events, "focus", "focus");

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
        match &mut content_background9.draw(&mut canvas, &mut focus_manager) {
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
