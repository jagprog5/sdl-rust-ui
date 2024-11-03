/// SDL sub-systems needed for testing
pub struct SDLSystems {
    pub event_pump: sdl2::EventPump,
    pub canvas: sdl2::render::WindowCanvas,
    #[allow(dead_code)]
    pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
}

impl SDLSystems {
    /// init sdl subsystems with window title and size
    pub fn new(
        win_title: &'static str,
        win_size: (u32, u32),
    ) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let sdl_video_subsystem = sdl_context.video()?;
        let window = sdl_video_subsystem
            .window(win_title, win_size.0, win_size.1)
            .resizable()
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        let event_pump = sdl_context.event_pump()?;
        Ok(Self {
            event_pump,
            canvas,
            texture_creator,
        })
    }
}