extern crate spidev;
extern crate sdl2;

mod display;

use sdl2::pixels::{ Color, PixelFormatEnum };
use sdl2::surface::Surface;
use sdl2::rect::Point;
use sdl2::render::Renderer;
use std::mem::transmute;
use std::slice::from_raw_parts;
use std::{ thread, time };

use self::display::Display;

fn main() {
    let mut display = Display::new(4, 2).unwrap();
    display.clear();
    display.set_intensity(2);

    let surface = Surface::new(32, 16, PixelFormatEnum::RGBA8888).unwrap();
    let mut renderer = Renderer::from_surface(surface).unwrap();
    let mut x = 0;
    loop {
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        renderer.draw_line(Point::new(x, 2), Point::new(10, 10)).unwrap();
        let pixels = unsafe { from_raw_parts((*renderer.surface().unwrap().raw()).pixels as *const u32, 32 * 16) };
        let display_data = pixels.into_iter()
            .map(|pixel| *pixel == 255u32)
            .collect::<Vec<_>>();
        display.display(&display_data);
        thread::sleep(time::Duration::from_millis(10));
        x = (x + 1) % 32;
    }
}

