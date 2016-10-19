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
    mpd.status();
    RenderInfo {
        mpd: mpd.status().unwrap(),
        ms: ms,
        time: actual_time
    }
}

fn loop_display(mpd: &mut Client, start_time: Instant) {
    let surface = Surface::new(32, 16, PixelFormatEnum::RGBA8888).unwrap();
    let mut renderer = Renderer::from_surface(surface).unwrap();
    let mut display = Display::new(4, 2).unwrap();
    display.clear().unwrap();
    display.set_intensity(2).unwrap();
    let render = graphics::create_render(&mut renderer);
    loop {
        render(&mut renderer, get_render_info(mpd, start_time));
        update_display(&renderer, &mut display);
        thread::sleep(time::Duration::from_millis(10));
    }
}

fn loop_window(mpd: &mut Client, start_time: Instant) {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video.window("musicpi-display", 320, 160)
        .build()
        .unwrap();
    let mut renderer = window.renderer().build().unwrap();
    renderer.set_scale(10.0, 10.0);
    let mut events = sdl_context.event_pump().unwrap();
    let render = graphics::create_render(&mut renderer);
    'a: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit {..} => { break 'a; },
                _ => {}
            }
        }
        render(&mut renderer, get_render_info(mpd, start_time));
        renderer.present();
        thread::sleep(time::Duration::from_millis(10));
    }
}

fn main() {
    let yaml = load_yaml!("commandline.yml");
    let arguments = App::from_yaml(yaml).get_matches();
    let use_display = !arguments.is_present("window");
    let mut mpd = Client::connect("127.0.0.1:6600").unwrap();
    let start_time = Instant::now();
    if use_display {
        loop_display(&mut mpd, start_time);
    } else {
        loop_window(&mut mpd, start_time);
    }
}

