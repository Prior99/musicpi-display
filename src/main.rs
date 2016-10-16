extern crate spidev;
extern crate sdl2;

use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;
use std::mem::transmute;
use std::slice::from_raw_parts;
mod display;

fn main() {
    let device_count = 8;
    let mut spi = display::create_display().unwrap();
    display::setup(device_count, &mut spi);
    display::clear(device_count, &mut spi);
    display::set_intensity(2, device_count, &mut spi);

    let surface = Surface::new(32, 16, PixelFormatEnum::RGBA8888).unwrap();
    let pixels = unsafe { from_raw_parts((*surface.raw()).pixels as *const u32, 32 * 16) };
    println!("{:?}", pixels);
    let demo = [
        false, false, false, true , false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, true , false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, true , false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        true , true , true , true , true , true , true , true ,    true , true , true , true , true , true , true , true ,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, true , false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, true , false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, true , false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, true , false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    

        false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, false, false, false, false, false,    false, false, false, true , false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
        false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    false, false, false, false, false, false, false, false,    
    ];
    display::display_slice(4, 2, &demo, &mut spi);
}

