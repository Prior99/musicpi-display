extern crate spidev;

mod display;

fn main() {
    let device_count = 8;
    let mut spi = display::create_display().unwrap();
    display::setup(device_count, &mut spi);
    display::clear(device_count, &mut spi);
    display::set_intensity(2, device_count, &mut spi);
}

