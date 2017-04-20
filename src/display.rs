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

/// One display consisting of N x M of 8 x 8 LED matrices chained and controlled by MAX7219.
pub struct Display {
    devices_horizontal: usize,
    devices_vertical: usize,
    spi: Spidev
}

impl Display {
    /// Returns the amount of devices connected to this display.
    fn get_devices(&self) -> usize {
        self.devices_horizontal * self.devices_vertical
    }

    /// Create a new display width specified dimensions of matrices.
    ///
    /// # Arguments
    ///
    /// * `devices_horizontal` - "Width" of the display in horizontal amount of single devices.
    /// * `devices_vertical` - "Height" of the display in vertical amount of single devices.
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
            Err(setup_result.err().unwrap())
        } else {
            Ok(display)
        }
    }

    /// Setup the display the way this library will use it.
    fn setup(&mut self) -> Result<()> {
        try!(self.write_all(Register::Intensity as u8, 1));
        try!(self.write_all(Register::Decodemode as u8, 0));
        try!(self.write_all(Register::Displaytest as u8, 0));
        try!(self.write_all(Register::Shutdown as u8, 1));
        try!(self.write_all(Register::Scanlimit as u8, 7));
        Ok(())
    }

    /// Convert a slice of boolean representing the state of 8 LEDs in a row to a u8 which can be
    /// sent to the controllers.
    ///
    /// # Arguments
    ///
    /// * `slice` - Slice of 8 booleans representing 8 LEDs in a row.
    fn slice_to_row_data(slice: &[bool]) -> u8 {
        slice.iter().fold(0u8, |data, &on| data >> 1 | if on { 128u8 } else { 0u8 })
    }

    /// Write a pair of register and data to all devices in the chain.
    ///
    /// # Arguments
    ///
    /// * `register` - The register of the MAX7219 into which the data should be written.
    /// * `data` - The data that should be written into the controllers register.
    pub fn write_all(&mut self, register: u8, data: u8) -> Result<()> {
        let write_data = (0 .. self.get_devices())
            .flat_map(|_| vec![register, data])
            .collect::<Vec<_>>();
        try!(self.spi.write_all(write_data.as_slice()));
        Ok(())
    }

    /// Write a pair of register and data to a specific device, sending NOOP to all other devices.
    ///
    /// # Arguments
    ///
    /// * `register` - The register of the MAX7219 into which the data should be written.
    /// * `data` - The data that should be written into the controllers register.
    /// * `device` - The index of the device to write the data to.
    pub fn write(&mut self, register: u8, data: u8, device: usize) -> Result<()> {
        let write_data = (0 .. self.get_devices()).flat_map(|current_device| {
            if device == current_device {
                vec![register, data]
            } else {
                vec![Register::Noop as u8, 0]
            }
        }).collect::<Vec<_>>();
        try!(self.spi.write_all(write_data.as_slice()));
        Ok(())
    }

    /// Clear the display, Switching all LEDs off.
    pub fn clear(&mut self) -> Result<()> {
        for row in 1 .. 8 {
            try!(self.write_all(row as u8, 0));
        }
        Ok(())
    }

    /// Set the intensity of the whole display.
    ///
    /// # Arguments
    ///
    /// * `intensity` - The intensity to set. Values from 1 to 16 are possible.
    pub fn set_intensity(&mut self, intensity: u8) -> Result<()> {
        try!(self.write_all(Register::Intensity as u8, intensity));
        Ok(())
    }

    /// Display a Slice of data on the display.
    ///
    /// # Arguments
    ///
    /// * `slice` - Data to display.
    ///
    /// # Example
    ///
    /// ```
    /// // Display a plus on a 1x1 display
    /// let mut d = Display::new(1, 1);
    /// d.clear();
    /// d.display([
    ///     false, false, false, true, true, false, false, false,
    ///     false, false, false, true, true, false, false, false,
    ///     false, false, false, true, true, false, false, false,
    ///     true, true, true, true, true, true, true, true,
    ///     true, true, true, true, true, true, true, true,
    ///     false, false, false, true, true, false, false, false,
    ///     false, false, false, true, true, false, false, false,
    ///     false, false, false, true, true, false, false, false
    /// ]);
    pub fn display(&mut self, slice: &[bool]) -> Result<()> {
        for device_x in 0 .. self.devices_horizontal {
            for device_y in 0 .. self.devices_vertical {
                for row in 0u8 .. 8u8 {
                    let index: usize =
                        device_x * 8 +
                        device_y * self.devices_horizontal * 64 +
                        row as usize * self.devices_horizontal * 8;
                    let row_data = Display::slice_to_row_data(&slice[index .. index + 8]);
                    let device = device_x + device_y * self.devices_horizontal;
                    try!(self.write(0x8u8 - row, row_data, device));
                }
            }
        }
        Ok(())
    }
}
