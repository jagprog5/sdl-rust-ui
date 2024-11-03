# sdl-rust-ui

This is a (in progress) library for creating [immediate mode](https://en.wikipedia.org/wiki/Immediate_mode_(computer_graphics)) user interfaces, built off of [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2).

For usage, see the [examples](./examples/) and documentation.

# Widget

A [widget](./src/widget/widget.rs) is a part of the interface. They are composed in a tree hierarchy in which a parent can contain some number of children.

## Sizing Information

A widget provides sizing information to be used by the parent:

- for both the width and height, each:
    - minimum length
    - maximum length
    - preferred portion (e.g. 50% of parent)
    - length failure policies (offset applied if a minimum or maximum can't be fulfilled by parent)
- requested aspect ratio

## Drawing

The parent accumulates sizing information from all the children and determines their positions for this frame. Once the positions are known, they are all updated, then all drawn.

Although sizing information is recalculated each frame, widgets should cache and reuse textures when appropriate.

## std-lib

These default widgets have been implemented:

Layouts:
 - [vertical layout](./src/layout/vertical_layout.rs)
 - [horizontal layout](./src/layout/horizontal_layout.rs)
 - [stacked layout](./src/layout/stacked_layout.rs)

Widgets
 - [debug](./src/widget/debug.rs), for testing sizing
 - [strut](./src/widget/strut.rs), forces spaces
 - [background](./src/widget/background.rs), parallel software rendering of a background texture
 - [border](./src/widget/border.rs), contains a widget in a border with a border style
 - [texture](./src/widget/texture.rs), generic texture display with sizing control
 - [button](./src/widget/button.rs)
 - [checkbox](./src/widget/checkbox.rs)

## TODO

- text input widget
- audio
- (maybe) better default style
