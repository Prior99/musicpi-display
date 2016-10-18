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

pub struct Display {
    devices_horizontal: usize,
    devices_vertical: usize,
    spi: Spidev
}

impl Display {
    pub fn get_devices(&self) -> usize {
        return self.devices_horizontal * self.devices_vertical;
    }

    pub fn new(devices_horizontal: usize, devices_vertical: usize) -> Result<Display> {
        let mut device = try!(Spidev::open("/dev/spidev0.0"));
        let mut options = SpidevOptions::new();
        options.bits_per_word(8);
        options.max_speed_hz(8_000_000);
        try!(device.configure(&options));
        let mut display = Display {
            devices_horizontal: devices_horizontal,
            devices_vertical: devices_vertical,
            spi: device
        };
        let setup_result = display.setup();
        if !setup_result.is_ok() {
            setup_result.err()
        } else {
            Ok(display)
        }
    }

    fn setup(&mut self) -> Result<()> {
        try!(display.write_all(Register::Intensity as u8, 1));
        try!(display.write_all(Register::Decodemode as u8, 0));
        try!(display.write_all(Register::Displaytest as u8, 0));
        try!(display.write_all(Register::Shutdown as u8, 1));
        try!(display.write_all(Register::Scanlimit as u8, 7));
    }

    fn slice_to_row_data(slice: &[bool]) -> u8 {
        slice.iter().fold(0u8, |data, &on| data >> 1 | if on { 128u8 } else { 0u8 })
    }

    pub fn write_all(&mut self, register: u8, data: u8) -> Result<()> {
        let write_data = (0 .. self.get_devices())
            .flat_map(|current_device| vec![register, data])
            .collect::<Vec<_>>();
        try!(self.spi.write(write_data.as_slice()));
        Ok(())
    }

    pub fn write(&mut self, register: u8, data: u8, device: usize) -> Result<()> {
        let write_data = (0 .. self.get_devices()).flat_map(|current_device| {
            if device == current_device {
                vec![register, data]
            } else {
                vec![Register::Noop as u8, 0]
            }
        }).collect::<Vec<_>>();
        try!(self.spi.write(write_data.as_slice()));
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        for row in 1 .. 8 {
            try!(self.write_all(row as u8, 0));
        }
        Ok(())
    }

    pub fn set_intensity(&mut self, intensity: u8) {
        self.write_all(Register::Intensity as u8, 1);
    }

    pub fn display(&mut self, slice: &[bool]) {
        let devices = self.get_devices();
        for device_x in 0 .. self.devices_horizontal {
            for device_y in 0 .. self.devices_vertical {
                for row in 0u8 .. 8u8 {
                    let index: usize =
                        device_x * 8 +
                        device_y * self.devices_horizontal * 64 +
                        row as usize * self.devices_horizontal * 8;
                    let row_data = Display::slice_to_row_data(&slice[index .. index + 8]);
                    let device = device_x + device_y * self.devices_horizontal;
                    self.write(0x8u8 - row, row_data, device);
                }
            }
        }
    }
}
