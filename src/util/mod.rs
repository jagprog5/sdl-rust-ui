pub mod focus;
pub mod render;
pub mod length;
pub mod rect;
pub(crate) mod rust;

// this module is not disabled when sdl-ttf is disabled - the traits are still
// valid and can be implemented without sdl2-ttf
pub mod font;

// module disabled with sdl2-mixer. unlike font, which declares some traits,
// those traits for audio are instead declared in their respective widget since
// they are suitably specific to each widget's needs
#[cfg(feature = "sdl2-mixer")]
pub mod audio;
pub(crate) mod shuffle;
