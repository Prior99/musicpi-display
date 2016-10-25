extern crate spidev;
extern crate sdl2;
extern crate sdl2_image;
extern crate chrono;
extern crate mpd;
#[macro_use]
extern crate clap;
extern crate pulse_simple;
extern crate dft;

mod graphics;
mod info;
mod spectrum;
mod display;
mod render;

use render::RenderTarget;
use render::display::RenderTargetDisplay;
use render::window::RenderTargetWindow;
use std::{thread, time};
use std::sync::mpsc::sync_channel;
use clap::{App};

fn main() {
    let yaml = load_yaml!("commandline.yml");
    let arguments = App::from_yaml(yaml).get_matches();
    let use_display = !arguments.is_present("window");
    let (info_sender, info_receiver) = sync_channel(0);
    let (spectrum_sender, spectrum_receiver) = sync_channel(0);
    let render_thread = thread::spawn(move || {
        if use_display {
            let mut renderer = RenderTargetDisplay::new(info_receiver, spectrum_receiver);
            renderer.run();
        } else {
            let mut renderer = RenderTargetWindow::new(info_receiver, spectrum_receiver);
            renderer.run();
        }
    });
    let update_thread = thread::spawn(move || {
        info::run(info_sender);
    });
    let spectrum_thread = thread::spawn(move || {
        spectrum::run(spectrum_sender);
    });
    render_thread.join();
    update_thread.join();
    spectrum_thread.join();
}

