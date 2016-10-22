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
pub mod display;

use sdl2::surface::Surface;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use std::slice::from_raw_parts;
use std::{thread, time};
use std::sync::mpsc::{sync_channel, Receiver};
use clap::{App};

use display::Display;
use graphics::RenderInfo;
use info::loop_info;
use spectrum::loop_spectrum;

fn update_display(renderer: &Renderer, display: &mut Display) {
    let pixels = unsafe { from_raw_parts((*renderer.surface().unwrap().raw()).pixels as *const u32, 32 * 16) };
    let display_data = pixels.into_iter()
        .map(|pixel| *pixel & 0xFF == 0xFFu32)
        .collect::<Vec<_>>();
    display.display(&display_data).unwrap();
}

fn loop_display(info_receiver: Receiver<RenderInfo>, spectrum_receiver: Receiver<Vec<f32>>) {
    let surface = Surface::new(32, 16, PixelFormatEnum::RGBA8888).unwrap();
    let mut renderer = Renderer::from_surface(surface).unwrap();
    let mut display = Display::new(4, 2).unwrap();
    display.clear().unwrap();
    display.set_intensity(1).unwrap();
    let render = graphics::create_render(&mut renderer);
    let mut render_info = info_receiver.recv().unwrap();
    let mut spectrum = spectrum_receiver.recv().unwrap();
    loop {
        let result = info_receiver.try_recv();
        if result.is_ok() {
            render_info = result.unwrap();
        }
        let spectrum_result = spectrum_receiver.recv();
        if spectrum_result.is_ok() {
            spectrum = spectrum_result.unwrap();
        }
        render(&mut renderer, render_info.clone(), spectrum.clone());
        update_display(&renderer, &mut display);
        thread::sleep(time::Duration::from_millis(1000/60));
    }
}

fn loop_window(info_receiver: Receiver<RenderInfo>, spectrum_receiver: Receiver<Vec<f32>>) {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video.window("musicpi-display", 320, 160)
        .build()
        .unwrap();
    let mut renderer = window.renderer().build().unwrap();
    renderer.set_scale(10.0, 10.0);
    let mut events = sdl_context.event_pump().unwrap();
    let render = graphics::create_render(&mut renderer);
    let mut render_info = info_receiver.recv().unwrap();
    let mut spectrum = spectrum_receiver.recv().unwrap();
    'a: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit {..} => { break 'a; },
                _ => {}
            }
        }
        let info_result = info_receiver.try_recv();
        if info_result.is_ok() {
            render_info = info_result.unwrap();
        }
        let spectrum_result = spectrum_receiver.recv();
        if spectrum_result.is_ok() {
            spectrum = spectrum_result.unwrap();
        }
        render(&mut renderer, render_info.clone(), spectrum.clone());
        renderer.present();
        thread::sleep(time::Duration::from_millis(10));
    }
}

fn main() {
    let yaml = load_yaml!("commandline.yml");
    let arguments = App::from_yaml(yaml).get_matches();
    let use_display = !arguments.is_present("window");
    let (info_sender, info_receiver) = sync_channel(0);
    let (spectrum_sender, spectrum_receiver) = sync_channel(0);
    let render_thread = thread::spawn(move || {
        if use_display {
            loop_display(info_receiver, spectrum_receiver);
        } else {
            loop_window(info_receiver, spectrum_receiver);
        }
    });
    let update_thread = thread::spawn(move || {
        loop_info(info_sender);
    });
    let spectrum_thread = thread::spawn(move || {
        loop_spectrum(spectrum_sender);
    });
    render_thread.join();
    update_thread.join();
    spectrum_thread.join();
}

