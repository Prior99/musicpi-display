extern crate spidev;
extern crate sdl2;
extern crate sdl2_image;
extern crate chrono;
extern crate mpd;
#[macro_use]
extern crate clap;

mod graphics;
pub mod display;

use sdl2::surface::Surface;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use std::slice::from_raw_parts;
use std::{thread, time};
use clap::{App};
use mpd::Client;
use std::time::Instant;
use self::display::Display;
use self::graphics::RenderInfo;
use chrono::{DateTime, Local};
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

fn update_display(renderer: &Renderer, display: &mut Display) {
    let pixels = unsafe { from_raw_parts((*renderer.surface().unwrap().raw()).pixels as *const u32, 32 * 16) };
    let display_data = pixels.into_iter()
        .map(|pixel| *pixel == 255u32)
        .collect::<Vec<_>>();
    display.display(&display_data).unwrap();
}

fn get_render_info(mpd: &mut Client, start_time: Instant) -> RenderInfo {
    let elapsed = Instant::now().duration_since(start_time);
    let ms = (1_000_000_000 * elapsed.as_secs() + elapsed.subsec_nanos() as u64)/(1_000_000);
    let actual_time: DateTime<Local> = Local::now();
    let status = mpd.status().unwrap();
    let optional_song = mpd.currentsong().unwrap();
    let (title, artist) = if optional_song.is_some() {
        let song = optional_song.unwrap();
        (song.title.unwrap_or(String::from("")), song.tags.get("Artist").unwrap_or(&String::from("")).clone())
    } else {
        (String::from(""), String::from(""))
    };
    RenderInfo {
        volume: status.volume,
        ms: ms,
        time: actual_time,
        song: title,
        artist: artist
    }
}

fn loop_display(receiver: Receiver<RenderInfo>) {
    let surface = Surface::new(32, 16, PixelFormatEnum::RGBA8888).unwrap();
    let mut renderer = Renderer::from_surface(surface).unwrap();
    let mut display = Display::new(4, 2).unwrap();
    display.clear().unwrap();
    display.set_intensity(2).unwrap();
    let render = graphics::create_render(&mut renderer);
    let mut render_info = receiver.recv().unwrap();
    loop {
        let result = receiver.try_recv();
        if result.is_ok() {
            render_info = result.unwrap();
        }
        render(&mut renderer, render_info.clone());
        update_display(&renderer, &mut display);
        thread::sleep(time::Duration::from_millis(10));
    }
}

fn loop_window(receiver: Receiver<RenderInfo>) {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video.window("musicpi-display", 320, 160)
        .build()
        .unwrap();
    let mut renderer = window.renderer().build().unwrap();
    renderer.set_scale(10.0, 10.0);
    let mut events = sdl_context.event_pump().unwrap();
    let render = graphics::create_render(&mut renderer);
    let mut render_info = receiver.recv().unwrap();
    'a: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit {..} => { break 'a; },
                _ => {}
            }
        }
        let result = receiver.try_recv();
        if result.is_ok() {
            render_info = result.unwrap();
        }
        render(&mut renderer, render_info.clone());
        renderer.present();
        thread::sleep(time::Duration::from_millis(10));
    }
}

fn main() {
    let yaml = load_yaml!("commandline.yml");
    let arguments = App::from_yaml(yaml).get_matches();
    let use_display = !arguments.is_present("window");
    let (sender, receiver) = sync_channel(0);
    let render_thread = thread::spawn(move || {
        if use_display {
            loop_display(receiver);
        } else {
            loop_window(receiver);
        }
    });
    let update_thread = thread::spawn(move || {
        let mut mpd = Client::connect("127.0.0.1:6600").unwrap();
        let start_time = Instant::now();
        loop {
            sender.send(get_render_info(&mut mpd, start_time));
        }
    });
    render_thread.join();
    update_thread.join();
}

