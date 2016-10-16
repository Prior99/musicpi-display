# MusicPi Display

A ~~NodeJs~~ Rust software for displaying information about currently playing music on a 32x16 pixel LED matrix.
The matrix is connected to a Raspberry Pi via SPI and is running on Max7219 LED controller.

The rendering is done using ~~Canvas~~ SDL2 for ~~Node~~ Rust.

# Installation

This package is intended to be used on a Raspberry Pi 2 or higher. It has to be compiled on the target machine,
as native libraries for SDL2 will be missing during cross-compilation.

It is possible to install rust and cargo on the Pi from the following AUR packages:
 - [rust-arm-bin](https://aur.archlinux.org/packages/rust-arm-bin/)
 - [cargo-arm-bin](https://aur.archlinux.org/packages/cargo-arm-bin/)

Also the packages for SDL2 and SDL2_image need to be installed.
