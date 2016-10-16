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
    options.max_speed_hz(8_000_000);
	try!(device.configure(&options));
    Ok(device)
}

fn write_vector(data: Vec<u8>, spi: &mut Spidev) -> Result<()> {
    try!(spi.write(data.as_slice()));
    Ok(())
}

pub fn display_slice(displays_horizontal: usize, displays_vertical: usize, slice: &[bool], spi: &mut Spidev) {
    let device_count = displays_horizontal * displays_vertical;
    for display_x in 0 .. displays_horizontal {
        for display_y in 0 .. displays_vertical {
            for row in 0u8 .. 8u8 {
                let index: usize = display_x * 8 + display_y * displays_horizontal * 64 + row as usize * displays_horizontal * 8;
                let row_data = slice_to_row_data(&slice[index .. index + 8]);
                let device = display_x + display_y * displays_horizontal;
                write(0x8u8 - row, row_data, device_count, device, spi);
            }
        }
    }
}

fn slice_to_row_data(slice: &[bool]) -> u8 {
    slice.iter()
        .fold(0u8, |data, &on| data >> 1 | if on { 128u8 } else { 0u8 })
}

pub fn clear(device_count: usize, spi: &mut Spidev) -> Result<()> {
    for row in 0x1 .. 0x8 {
        try!(write_all(row as u8, 0, device_count, spi));
    }
    Ok(())
}

pub fn write_all(register: u8, data: u8, device_count: usize, spi: &mut Spidev) -> Result<()> {
    let write_data = (0usize .. device_count)
        .flat_map(|current_device| vec![register, data])
        .collect::<Vec<_>>();
    write_vector(write_data, spi)
}

pub fn write(register: u8, data: u8, device_count: usize, device: usize, spi: &mut Spidev) -> Result<()> {
    let write_data = (0 .. device_count).flat_map(|current_device| {
        if device == current_device {
            vec![register, data]
        } else {
            vec![Register::Noop as u8, 0]
        }
    }).collect::<Vec<_>>();
    write_vector(write_data, spi)
}

pub fn set_intensity(intensity: u8, device_count: usize, spi: &mut Spidev) {
    write_all(Register::Intensity as u8, 1, device_count, spi);
}

pub fn setup(device_count: usize, spi: &mut Spidev) {
    write_all(Register::Intensity as u8, 1, device_count, spi);
    write_all(Register::Decodemode as u8, 0, device_count, spi);
    write_all(Register::Displaytest as u8, 0, device_count, spi);
    write_all(Register::Shutdown as u8, 1, device_count, spi);
    write_all(Register::Scanlimit as u8, 7, device_count, spi);
}
