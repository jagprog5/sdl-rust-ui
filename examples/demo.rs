// TODO more in the demo

use std::{cell::Cell, fs::File, io::Read, path::Path, process::exit, time::Duration};

use sdl2::{mouse::MouseButton, pixels::Color, render::TextureCreator, video::WindowContext};
use tiny_sdl2_gui::{
    layout::vertical_layout::{MajorAxisMaxLenPolicy, VerticalLayout},
    util::{
        focus::{FocusID, FocusManager},
        font::{FontManager, SingleLineTextRenderType, TextRenderer},
        length::{MaxLen, MaxLenFailPolicy, MaxLenPolicy, MinLenFailPolicy},
    },
    widget::{
        button::{Button, LabelButtonStyle},
        checkbox::EmptyFocusPressWidgetSoundStyle,
        single_line_label::SingleLineLabel,
        strut::Strut,
        update_gui, Widget,
    },
};

#[path = "example_common/mod.rs"]
mod example_common;

#[derive(Debug, Clone, Copy, Default)]
enum GameState {
    #[default]
    MainMenu,
    // TODO
}

pub struct GUI<'sdl> {
    root: Box<dyn Widget + 'sdl>,
    start_focus: &'static str,
    end_focus: &'static str,
}

fn main_menu_gui<'sdl>(
    font_manager: &'sdl Cell<Option<FontManager<'sdl>>>,
    texture_creator: &'sdl TextureCreator<WindowContext>,
) -> GUI<'sdl> {
    let mut big_title = SingleLineLabel::new(
        "Game Title".into(),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        texture_creator,
    );
    big_title.max_h_fail_policy = MaxLenFailPolicy::NEGATIVE;
    big_title.min_h = 50.0.into();
    // -------------------------------------------------------------------------
    // space between the title and the buttons
    let main_menu_spacer = Strut::shrinkable(0.0.into(), MaxLen::LAX);
    // -------------------------------------------------------------------------
    let new_button_vertical_space = Strut::shrinkable(0.0.into(), 20.0.into());
    let mut new_button_label = SingleLineLabel::new(
        "New Game".into(),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        texture_creator,
    );
    new_button_label.max_h = 50.0.into();
    new_button_label.min_h = 25.0.into();
    let new_button_style = LabelButtonStyle {
        label: new_button_label,
    };
    let new_button = Button::new(
        Box::new(|| todo!()), // intentional
        FocusID {
            previous: "back".to_owned(),
            me: "new".to_owned(),
            next: "load".to_owned(),
        },
        Box::new(new_button_style),
        Box::new(EmptyFocusPressWidgetSoundStyle {}),
        texture_creator,
    );
    // -------------------------------------------------------------------------
    let load_button_vertical_space = Strut::shrinkable(0.0.into(), 20.0.into());
    let mut load_button_label = SingleLineLabel::new(
        "Load Game".into(),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        texture_creator,
    );
    load_button_label.max_h = 50.0.into();
    load_button_label.min_h = 25.0.into();
    let load_button_style = LabelButtonStyle {
        label: load_button_label,
    };
    let load_button = Button::new(
        Box::new(|| todo!()), // intentional
        FocusID {
            previous: "new".to_owned(),
            me: "load".to_owned(),
            next: "back".to_owned(),
        },
        Box::new(load_button_style),
        Box::new(EmptyFocusPressWidgetSoundStyle {}),
        texture_creator,
    );
    // -------------------------------------------------------------------------
    let back_button_vertical_space = Strut::shrinkable(0.0.into(), 20.0.into());
    let mut back_button_label = SingleLineLabel::new(
        "Back".into(),
        SingleLineTextRenderType::Blended(Color::WHITE),
        Box::new(TextRenderer::new(&font_manager)),
        texture_creator,
    );
    back_button_label.max_h = 50.0.into();
    back_button_label.min_h = 25.0.into();
    let back_button_style = LabelButtonStyle {
        label: back_button_label,
    };
    let back_button = Button::new(
        Box::new(|| {
            exit(0);
        }),
        FocusID {
            previous: "load".to_owned(),
            me: "back".to_owned(),
            next: "new".to_owned(),
        },
        Box::new(back_button_style),
        Box::new(EmptyFocusPressWidgetSoundStyle {}),
        texture_creator,
    );

    let mut main_menu_buttons = VerticalLayout::default();
    // the title is 1/4, and the buttons are 3/4 of the height
    main_menu_buttons.preferred_h = 3.0.into();
    main_menu_buttons.max_h_policy = MajorAxisMaxLenPolicy::Together(MaxLenPolicy::Children);

    let mut vertical = VerticalLayout::default();
    vertical.min_h_fail_policy = MinLenFailPolicy::NEGATIVE;
    vertical.elems.push(Box::new(big_title));

    main_menu_buttons.elems.push(Box::new(main_menu_spacer));
    main_menu_buttons.elems.push(Box::new(new_button));
    main_menu_buttons
        .elems
        .push(Box::new(new_button_vertical_space));
    main_menu_buttons.elems.push(Box::new(load_button));
    main_menu_buttons
        .elems
        .push(Box::new(load_button_vertical_space));
    main_menu_buttons.elems.push(Box::new(back_button));
    main_menu_buttons
        .elems
        .push(Box::new(back_button_vertical_space));
    vertical.elems.push(Box::new(main_menu_buttons));

    GUI {
        root: Box::new(vertical),
        start_focus: "new",
        end_focus: "back",
    }
}

fn main() -> std::process::ExitCode {
    // ========================== PRELUDE ======================================
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;
    const MAX_DELAY: Duration = Duration::from_millis(17);

    let sdl_context = sdl2::init().unwrap();
    let sdl_video_subsystem = sdl_context.video().unwrap();
    let window = sdl_video_subsystem
        .window("demo todo!", WIDTH, HEIGHT)
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
    let mut focus_manager = FocusManager::default();
    let game_state = Cell::new(GameState::default());

    example_common::gui_loop::gui_loop(MAX_DELAY, &mut event_pump, |events| {
        let mut gui = match game_state.get() {
            GameState::MainMenu => main_menu_gui(&font_manager, &texture_creator),
            #[allow(unreachable_patterns)]
            _ => todo!(),
        };
        
        // UPDATE
        match update_gui(
            gui.root.as_mut(),
            events,
            &mut focus_manager,
            &canvas,
        ) {
            Ok(()) => {}
            Err(msg) => {
                debug_assert!(false, "{}", msg); // infallible in prod
            }
        };

        FocusManager::default_start_focus_behavior(&mut focus_manager, events, &gui.start_focus, &gui.end_focus);

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

        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
            canvas.clear();
            match gui.root.as_mut().draw(&mut canvas, &mut focus_manager) {
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
