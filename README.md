# sdl-rust-ui

[Video](https://youtu.be/3zBEHgBt7EQ?si=eZAg6nufy3mj0sCg)🔗

This is a (in progress) library for creating [immediate mode](https://en.wikipedia.org/wiki/Immediate_mode_(computer_graphics)) user interfaces, built off of [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2).

For usage, see the [examples](./examples/) and documentation.

# What's Special?

## Screen Updates

Some GUI frameworks assume that the screen updates all the time, e.g. 60fps.
Especially immediate mode GUIs. This makes the update order pretty lenient. If A
depends on B, and A is updated before B, then A will be drawn with B's state
from one frame behind. For most frameworks this doesn't matter, since once the
next frame comes around it will get back in sync with the underlying data.

With this framework, the screen only needs to be updated once something happens.
Everything will look as it should _within that same frame_. For the provided
examples, the idle CPU usage is very near 0.

## Sizing Information

Widgets compose a tree hierarchy. Parent widgets receive sizing information from
the children and layout appropriately.

I haven't seen sizing or layouts expressed in this specific way; 
most frameworks use a preferred size. Here, a preferred portion is used instead. This works well for letting the UI scale with the window size. There's also support for aspect ratios and "length failure" policies (what happens if a minimum or maximum can't be fulfilled by the parent). 

# std-lib
 - [vertical layout](./src/layout/vertical_layout.rs)
 - [horizontal layout](./src/layout/horizontal_layout.rs)
 - [scroll area](./src/layout/scroller.rs)
 - [clipper](./src/layout/clipper.rs)
 - [debug](./src/widget/debug.rs), for testing sizing
 - [strut](./src/widget/strut.rs), forces spaces
 - [background](./src/widget/background.rs), solid color or parallel software rendering of a background texture
 - [border](./src/widget/border.rs), contains a widget in a border with a border style
 - [texture](./src/widget/texture.rs), generic texture display with sizing control
 - [single](./src/widget/single_line_label.rs) and [multiline](./src/widget/multi_line_label.rs) labels
 - [single line text input](./src/widget/single_line_text_input.rs)
 - [button](./src/widget/button.rs)
 - [checkbox](./src/widget/checkbox.rs)
