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
mod target;

use target::Target;
use target::display::TargetDisplay;
use target::window::TargetWindow;
use std::thread;
use std::sync::mpsc::{sync_channel, channel};
use clap::{App};

fn main() {
    let yaml = load_yaml!("commandline.yml");
    let arguments = App::from_yaml(yaml).get_matches();
    let use_display = !arguments.is_present("window");
    let (info_sender, info_receiver) = sync_channel(0);
    let (spectrum_sender, spectrum_receiver) = channel();
    let render_thread = thread::spawn(move || {
        let mut target: Box<Target> = if use_display {
            Box::new(TargetDisplay::new(info_receiver, spectrum_receiver))
        } else {
            Box::new(TargetWindow::new(info_receiver, spectrum_receiver))
        };
        target.run();
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

