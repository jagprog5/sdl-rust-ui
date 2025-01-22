use std::cell::Cell;

use rand::Rng;
use sdl2::pixels::Color;
use tiny_sdl2_gui::{
    layout::scroller::{Scroller, ScrollerSizingPolicy},
    util::{
        focus::{CircularUID, FocusManager, PRNGBytes, RefCircularUIDCell, UID},
        length::PreferredPortion,
    },
    widget::{
        background::{BackgroundSizingPolicy, SolidColorBackground},
        border::{Bevel, Border, Empty, Gradient, Line},
        checkbox::{CheckBox, DefaultCheckBoxStyle},
        debug::CustomSizingControl,
        widget::{draw_gui, update_gui, SDLEvent},
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

fn main() -> std::process::ExitCode {
    const WIDTH: u32 = 300;
    const HEIGHT: u32 = 200;

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    let window = sdl_video_subsystem
        .window("nested scroller test", WIDTH, HEIGHT)
        .resizable()
        .position_centered()
        .build().unwrap();
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut focus_manager = FocusManager::default();

    let checkbox_state = Cell::new(false);

    let mut rng = rand::thread_rng();

    let mut get_prng_bytes = || {
        let mut bytes = [0u8; 8];
        rng.fill(&mut bytes);
        PRNGBytes(bytes)
    };

    let checkbox_focus_id = get_prng_bytes();
    let checkbox_focus_id = UID::new(checkbox_focus_id);
    let checkbox_focus_id = CircularUID::new(checkbox_focus_id);
    let checkbox_focus_id = Cell::new(checkbox_focus_id);
    let checkbox_focus_id = RefCircularUIDCell(&checkbox_focus_id);
    let focus_press_sound_style =
        tiny_sdl2_gui::widget::checkbox::EmptyFocusPressWidgetSoundStyle {};

    // there is a checkbox. it is the only element
    let mut checkbox = CheckBox::new(
        &checkbox_state,
        checkbox_focus_id,
        Box::new(DefaultCheckBoxStyle {}),
        Box::new(focus_press_sound_style),
        &texture_creator,
    );
    checkbox.focus_id.single_id_loop();

    // pad the checkbox a little bit for clarity
    let mut checkbox_border1 = Border::new(
        &mut checkbox,
        &texture_creator,
        Box::new(Empty { width: 5 }),
    );

    // contain the checkbox and padding in a border
    let mut checkbox_border2 = Border::new(
        &mut checkbox_border1,
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
        &mut checkbox_border2,
    );
    let mut sizing = CustomSizingControl::default();
    sizing.preferred_w = PreferredPortion(0.8);
    sizing.preferred_h = PreferredPortion(0.8);
    inner_scroller4.sizing_policy = ScrollerSizingPolicy::Custom(sizing, Default::default());

    // contain all of the above in a border
    let mut inner_content_border5 = Border::new(
        &mut inner_scroller4,
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
        &mut inner_content_border5,
    );
    outer_scroller6.sizing_policy = ScrollerSizingPolicy::Custom(sizing, Default::default());
    outer_scroller6.restrict_scroll = false;

    // contain all of the above in a border
    let mut outer_content_border7 = Border::new(
        &mut outer_scroller6,
        &texture_creator,
        Box::new(Bevel::default()),
    );

    let mut content_background8 = SolidColorBackground {
        color: Color::BLACK,
        contained: &mut outer_content_border7,
        sizing_policy: BackgroundSizingPolicy::Children,
    };

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
                &mut content_background9,
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
                checkbox_focus_id.uid(),
                checkbox_focus_id.uid(),
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
                &mut content_background9,
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
