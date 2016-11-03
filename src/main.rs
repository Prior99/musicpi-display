#![feature(iter_min_by)]
extern crate spidev;
extern crate sdl2;
extern crate sdl2_image;
extern crate chrono;
extern crate mpd;
#[macro_use]
extern crate clap;
extern crate pulse_simple;
extern crate dft;
extern crate bus;
extern crate nalgebra;
extern crate core;

mod graphics;
mod info;
mod spectrum;
mod display;
mod target;

use bus::{Bus, BusReader};
use target::Target;
use target::display::TargetDisplay;
use target::window::TargetWindow;
use std::thread::{spawn, JoinHandle};
use std::sync::mpsc::{sync_channel, channel, Receiver, SyncSender, Sender};
use clap::{App};
use spectrum::SpectrumResult;
use graphics::RenderInfo;

#[derive(Clone, PartialEq)]
pub enum ControlStatus {
    Abort
}

fn thread_render(
        control_tx: SyncSender<ControlStatus>,
        control_rx: BusReader<ControlStatus>,
        info_rx: Receiver<RenderInfo>,
        spectrum_rx: Receiver<SpectrumResult>,
        use_display: bool) -> JoinHandle<()> {
    spawn(move || {
        let mut target: Box<Target> = if use_display {
            Box::new(TargetDisplay::new(info_rx, spectrum_rx).unwrap())
        } else {
            Box::new(TargetWindow::new(info_rx, spectrum_rx).unwrap())
        };
        if !target.run(control_rx).is_ok() {
            control_tx.send(ControlStatus::Abort).ok();
        }
    })
}

fn thread_spectrum(
        control_tx: SyncSender<ControlStatus>,
        control_rx: BusReader<ControlStatus>,
        spectrum_tx: Sender<SpectrumResult>) -> JoinHandle<()> {
    spawn(move || {
        if !spectrum::run(control_rx, spectrum_tx).is_ok() {
            control_tx.send(ControlStatus::Abort).ok();
        }
    })
}

fn thread_info(
        control_tx: SyncSender<ControlStatus>,
        control_rx: BusReader<ControlStatus>,
        info_tx: SyncSender<RenderInfo>) -> JoinHandle<()> {
    spawn(move || {
        if !info::run(control_rx, info_tx).is_ok() {
            control_tx.send(ControlStatus::Abort).ok();
        }
    })
}

fn main() {
    let yaml = load_yaml!("commandline.yml");
    let arguments = App::from_yaml(yaml).get_matches();
    let use_display = !arguments.is_present("window");
    let (info_tx, info_rx) = sync_channel(0);
    let (spectrum_tx, spectrum_rx) = channel();
    let (control_tx, control_rx) = sync_channel(3);
    let mut control_bus = Bus::new(3);
    let join_render = thread_render(control_tx.clone(), control_bus.add_rx(), info_rx, spectrum_rx, use_display);
    let join_info = thread_info(control_tx.clone(), control_bus.add_rx(), info_tx);
    let join_spectrum = thread_spectrum(control_tx.clone(), control_bus.add_rx(), spectrum_tx);
    let join_control = spawn(move || {
        for message in control_rx.iter() {
            control_bus.broadcast(message.clone());
            if message == ControlStatus::Abort {
                break;
            }
        }
    });
    join_render.join().expect("The render thread crashed.");
    join_info.join().expect("The info thread crashed.");
    join_spectrum.join().expect("The audio analysing thread crashed.");
    join_control.join().expect("The control thread crashed.");
}

