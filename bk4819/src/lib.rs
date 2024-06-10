#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

mod doc_table;

pub mod registers;
pub use registers::Register;

/// An interface to the Beken BK4819 chip.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bk4819<Scn, Scl, Sda, Delay> {
    scn: Scn,
    scl: Scl,
    sda: Sda,
    delay: Delay,
}

/// An error produced by the BK4819 interface.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// GPIO Error.
    Bus(E),
}

impl<E> From<E> for Error<E> {
    fn from(other: E) -> Self {
        Self::Bus(other)
    }
}

/// A handle to do raw communication with a BK4819.
///
/// Created by [Bk4819::transaction()].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct Raw<'a, Scn, Scl, Sda, Delay> {
    device: &'a mut Bk4819<Scn, Scl, Sda, Delay>,
}

impl<'a, Scn, Scl, Sda, Delay, E> Raw<'a, Scn, Scl, Sda, Delay>
where
    Scn: OutputPin<Error = E>,
    Scl: OutputPin<Error = E>,
    Sda: OutputPin<Error = E> + InputPin<Error = E>,
    Delay: DelayNs,
{
    /// Wait half a clock cycle.
    #[inline(always)]
    fn wait_clk(&mut self) {
        self.device.wait_clk();
    }

    /// Write a single bit to the device.
    #[inline(always)]
    fn write_bit(&mut self, bit: bool) -> Result<(), Error<E>> {
        // set data on falling edge
        self.device.sda.set_state(bit.into())?;
        self.device.scl.set_low()?;

        self.wait_clk();

        // sample data on rising edge
        self.device.scl.set_high()?;

        self.wait_clk();

        Ok(())
    }

    /// Read a single bit from the device.
    #[inline(always)]
    fn read_bit(&mut self) -> Result<bool, Error<E>> {
        // weird thing: try to read from sda now and discard it
        // to flip it to input for implementations that flip on demand
        self.device.sda.is_high()?;

        // set data on falling edge
        self.device.scl.set_low()?;

        self.wait_clk();

        // sample data on rising edge
        let bit = self.device.sda.is_high()?;
        self.device.scl.set_high()?;

        self.wait_clk();

        Ok(bit)
    }

    /// Write a byte to the device.
    fn write_u8(&mut self, mut data: u8) -> Result<(), Error<E>> {
        for _ in 0..u8::BITS {
            self.write_bit(data & (1 << (u8::BITS - 1)) > 0)?;
            data <<= 1;
        }

        Ok(())
    }

    /// Write a word to the device.
    fn write_u16(&mut self, mut data: u16) -> Result<(), Error<E>> {
        for _ in 0..u16::BITS {
            self.write_bit(data & (1 << (u16::BITS - 1)) > 0)?;
            data <<= 1;
        }

        Ok(())
    }

    /// Read a word from the device.
    fn read_u16(&mut self) -> Result<u16, Error<E>> {
        let mut value = 0;
        for _ in 0..u16::BITS {
            value <<= 1;
            value |= if self.read_bit()? { 1 } else { 0 };
        }

        Ok(value)
    }
}

impl<Scn, Scl, Sda, Delay, E> Bk4819<Scn, Scl, Sda, Delay>
where
    Scn: OutputPin<Error = E>,
    Scl: OutputPin<Error = E>,
    Sda: OutputPin<Error = E> + InputPin<Error = E>,
    Delay: DelayNs,
{
    /// Create the interface with the given pins and delay implementation.
    ///
    /// The delay implementation will be asked to delay by single
    /// microseconds.
    pub fn new(scn: Scn, scl: Scl, sda: Sda, delay: Delay) -> Result<Self, Error<E>> {
        let mut this = Self {
            scn,
            scl,
            sda,
            delay,
        };

        this.reset()?;
        Ok(this)
    }

    /// Release the pins and delay used by this interface.
    pub fn release(self) -> (Scn, Scl, Sda, Delay) {
        (self.scn, self.scl, self.sda, self.delay)
    }

    /// Wait half a clock cycle.
    #[inline(always)]
    fn wait_clk(&mut self) {
        self.delay.delay_us(1);
    }

    /// Perform a transaction, reading and writing from the device.
    fn transaction<T>(
        &mut self,
        f: impl FnOnce(&mut Raw<'_, Scn, Scl, Sda, Delay>) -> Result<T, Error<E>>,
    ) -> Result<T, Error<E>> {
        // bring scn and scl low to activate device
        self.scn.set_low()?;
        self.scl.set_low()?;

        // do the transaction
        let res = f(&mut Raw { device: self });

        // bring scl low once more at the end
        self.scl.set_low()?;

        self.wait_clk();

        // bring scn and scl high to deactivate device
        self.scn.set_high()?;
        self.scl.set_high()?;

        self.wait_clk();

        // set sda to high at the very end so its in a known state
        self.sda.set_high()?;

        res
    }

    /// Read a raw register on the device.
    pub fn read_raw(&mut self, address: u8) -> Result<u16, Error<E>> {
        self.transaction(|raw| {
            raw.write_u8(address | 0x80)?;
            raw.read_u16()
        })
    }

    /// Write a raw register to the device.
    pub fn write_raw(&mut self, address: u8, value: u16) -> Result<(), Error<E>> {
        self.transaction(|raw| {
            raw.write_u8(address & 0x7f)?;
            raw.write_u16(value)
        })
    }

    /// Modify a raw register on the device.
    pub fn modify_raw(&mut self, address: u8, f: impl FnOnce(u16) -> u16) -> Result<(), Error<E>> {
        let value = self.read_raw(address)?;
        self.write_raw(address, f(value))
    }

    /// Read a register on the device.
    pub fn read<R>(&mut self) -> Result<R, Error<E>>
    where
        R: Register,
    {
        Ok(self.read_raw(R::ADDRESS)?.into())
    }

    /// Write a register to the device.
    pub fn write<R>(&mut self, value: R) -> Result<(), Error<E>>
    where
        R: Register,
    {
        self.write_raw(R::ADDRESS, value.into())
    }

    /// Modify a register on the device.
    pub fn modify<R>(&mut self, f: impl FnOnce(R) -> R) -> Result<(), Error<E>>
    where
        R: Register,
    {
        let value = self.read()?;
        self.write(f(value))
    }

    /// Reset the device.
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        // set everything to the default state
        self.scn.set_high()?;
        self.scl.set_high()?;
        self.sda.set_high()?;

        // let it all settle
        self.wait_clk();

        self.write(registers::Reset::new().with_reset(true))?;
        self.write(registers::Reset::new())?;

        Ok(())
    }

    /// Is a given GPIO output enabled?
    pub fn gpio_is_output_enabled(&mut self, pin: u8) -> Result<bool, Error<E>> {
        Ok(self.read::<registers::GpioOutput>()?.enabled(pin))
    }

    /// Set a given GPIO output to be enabled.
    pub fn gpio_set_output_enabled(&mut self, pin: u8, enabled: bool) -> Result<(), Error<E>> {
        self.modify(|r: registers::GpioOutput| r.with_enabled(pin, enabled))
    }

    /// Is a given GPIO output set high?
    pub fn gpio_is_set_high(&mut self, pin: u8) -> Result<bool, Error<E>> {
        Ok(self.read::<registers::GpioOutput>()?.state(pin))
    }

    /// Is a given GPIO output set low?
    pub fn gpio_is_set_low(&mut self, pin: u8) -> Result<bool, Error<E>> {
        Ok(!self.gpio_is_set_high(pin)?)
    }

    /// Set a given GPIO output state.
    pub fn gpio_set_state(&mut self, pin: u8, state: bool) -> Result<(), Error<E>> {
        self.modify(|r: registers::GpioOutput| r.with_state(pin, state))
    }

    /// Set a given GPIO output high.
    pub fn gpio_set_high(&mut self, pin: u8) -> Result<(), Error<E>> {
        self.gpio_set_state(pin, true)
    }

    /// Set a given GPIO output low.
    pub fn gpio_set_low(&mut self, pin: u8) -> Result<(), Error<E>> {
        self.gpio_set_state(pin, false)
    }

    /// Toggle the given GPIO output.
    pub fn gpio_toggle(&mut self, pin: u8) -> Result<(), Error<E>> {
        self.modify(|r: registers::GpioOutput| r.with_state(pin, !r.state(pin)))
    }
}
