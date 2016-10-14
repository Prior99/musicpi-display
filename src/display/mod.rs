extern crate spidev;

use std::io::Result;
use spidev::[Spidev,SpidevOptions];

fn create_display() -> Result<Spidev> {
    let mut device = try!(Spidev::open("/dev/spidev0.0"));
    let mut options = SpidevOptions::new();
    options.bits_per_word(8);
    options.max_speed_hz(20000);
    try!(device.configure(options));
    Ok(device)
}

fn () ->
