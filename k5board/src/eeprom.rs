//! Access to the on-board EEPROM, which stores configuration.

use eeprom24x::{
    addr_size::TwoBytes, page_size::B32, unique_serial::No, Eeprom24x, Error as Error24x, SlaveAddr,
};
use embedded_hal_02::blocking::delay::DelayMs;
use embedded_hal_02::timer::{CountDown, Periodic};

use crate::hal::gpio::{OpenDrain, Output, SharedPin, PA10, PA11};
use crate::shared_i2c::SharedI2c;

/// The size of the EEPROM in bytes.
pub const SIZE: usize = 0x2000;

/// The page size of this EEPROM in bytes.
pub const PAGE_SIZE: usize = 32;

/// An EEPROM error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// An error with the I2C bus.
    I2c,
    /// Too much data for a single page write.
    TooMuchData,
    /// Tried to read or write beyond the end of the EEPROM.
    InvalidAddr,
}

impl<E> From<Error24x<E>> for Error {
    fn from(other: Error24x<E>) -> Error {
        match other {
            Error24x::I2C(_) => Self::I2c,
            Error24x::TooMuchData => Self::TooMuchData,
            Error24x::InvalidAddr => Self::InvalidAddr,
        }
    }
}

/// The shared bus the EEPROM lives on.
pub type EepromI2c<'a, Timer> =
    SharedI2c<'a, Timer, SharedPin<PA10<Output<OpenDrain>>>, SharedPin<PA11<Output<OpenDrain>>>>;

/// The EEPROM interface.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Eeprom<'a, Timer> {
    eeprom: Eeprom24x<EepromI2c<'a, Timer>, B32, TwoBytes, No>,
}

/// Create an EEPROM interface from a shared bus.
pub fn new<Timer>(i2c: EepromI2c<'_, Timer>) -> Eeprom<'_, Timer>
where
    Timer: CountDown + Periodic,
{
    Eeprom::new(i2c)
}

impl<'a, Timer> Eeprom<'a, Timer>
where
    Timer: CountDown + Periodic,
{
    /// Create an EEPROM interface from a shared bus.
    pub fn new(i2c: EepromI2c<'a, Timer>) -> Self {
        Self {
            eeprom: Eeprom24x::new_24x64(i2c, SlaveAddr::default()),
        }
    }

    /// Free the EEPROM and return the shared bus.
    pub fn free(self) -> EepromI2c<'a, Timer> {
        self.eeprom.destroy()
    }

    /// Read data from the eeprom.
    pub fn read(&mut self, address: usize, data: &mut [u8]) -> Result<(), Error> {
        Ok(self.eeprom.read_data(address as u32, data)?)
    }

    /// Write one page of data, directly.
    ///
    /// See [Self::write()] for a more general-purpose writing method.
    pub fn write_page(&mut self, address: usize, data: &[u8]) -> Result<(), Error> {
        Ok(self.eeprom.write_page(address as u32, data)?)
    }

    /// Write data to the EEPROM, waiting after each page write.
    pub fn write<Delay>(
        &mut self,
        delay: &mut Delay,
        mut offset: usize,
        mut bytes: &[u8],
    ) -> Result<(), Error>
    where
        Delay: DelayMs<u8>,
    {
        if offset + bytes.len() > SIZE {
            return Err(Error::TooMuchData);
        }

        while !bytes.is_empty() {
            let this_page_offset = offset % PAGE_SIZE;
            let this_page_remaining = PAGE_SIZE - this_page_offset;
            let chunk_size = bytes.len().min(this_page_remaining);

            self.write_page(offset, &bytes[..chunk_size])?;

            offset += chunk_size;
            bytes = &bytes[chunk_size..];

            // eeprom takes 5ms, lets be safe
            delay.delay_ms(6);
        }

        Ok(())
    }
}
