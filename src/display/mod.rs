use std::io::Result;
use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions};

enum Register {
	Noop = 0x0,
	Decodemode = 0x9,
	Intensity = 0xA,
	Scanlimit = 0xB,
	Shutdown = 0xC,
	Displaytest = 0xF
}

pub fn create_display() -> Result<Spidev> {
    let mut device = try!(Spidev::open("/dev/spidev0.0"));
    let mut options = SpidevOptions::new();
    options.bits_per_word(8);
    options.max_speed_hz(20000);
	try!(device.configure(&options));
    Ok(device)
}

fn write_vector(data: Vec<u8>, spi: &mut Spidev) -> Result<()> {
    try!(spi.write(data.as_slice()));
    Ok(())
}

pub fn clear(device_count: u32, spi: &mut Spidev) -> Result<()> {
    for row in 0x1 .. 0x8 {
        try!(write_all(row as u8, 0, device_count, spi));
    }
    Ok(())
}

pub fn write_all(register: u8, data: u8, device_count: u32, spi: &mut Spidev) -> Result<()> {
    let write_data = (0u32 .. device_count)
        .flat_map(|current_device| vec![register, data])
        .collect::<Vec<_>>();
    return write_vector(write_data, spi);
}

pub fn write(register: u8, data: u8, device_count: u32, device: u32, spi: &mut Spidev) -> Result<()> {
    let write_data = (0 .. device_count).flat_map(|current_device| {
        if device == current_device {
            vec![register, data]
        } else {
            vec![Register::Noop as u8, 0]
        }
    }).collect::<Vec<_>>();
    return write_vector(write_data, spi);
}

pub fn set_intensity(intensity: u8, device_count: u32, spi: &mut Spidev) {
    write_all(Register::Intensity as u8, 1, device_count, spi);
}

pub fn setup(device_count: u32, spi: &mut Spidev) {
    write_all(Register::Intensity as u8, 1, device_count, spi);
    write_all(Register::Decodemode as u8, 0, device_count, spi);
    write_all(Register::Displaytest as u8, 0, device_count, spi);
    write_all(Register::Shutdown as u8, 1, device_count, spi);
    write_all(Register::Scanlimit as u8, 7, device_count, spi);
}
