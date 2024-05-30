//! Bit-banged I2C bus that can be shared, with raw access.

use core::cell::{RefCell, RefMut};

use bitbang_hal::i2c::{Error as BBError, I2cBB};
use critical_section::{with, CriticalSection, Mutex};
use embedded_hal_02::blocking::i2c as hal02;
use embedded_hal_02::digital::v2::{InputPin, OutputPin, PinState};
use embedded_hal_02::timer::{CountDown, Periodic};
use embedded_hal_1::i2c as hal1;

use crate::hal::block;

// unfortunately, shared_bus does not expose the underlying object,
// so we can't access the raw methods on the bitbang_hal object
// so we basically roll our own here.

// this isn't shared_bus's fault -- they need trait support in embedded-hal
// to do this safely.

/// A bit-bang I2C error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Bus,
    NoAck,
    InvalidData,
}

impl<E> From<BBError<E>> for Error {
    fn from(other: BBError<E>) -> Error {
        match other {
            BBError::Bus(_) => Self::Bus,
            BBError::NoAck => Self::NoAck,
            BBError::InvalidData => Self::InvalidData,
        }
    }
}

/// The pins and parts required for a shared I2C bus.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Parts<Timer, Scl, Sda> {
    /// A periodic timer that rolls over at twice the desired I2C frequency.
    pub clk: Timer,
    /// The SCL pin.
    pub scl: Scl,
    /// The SDA pin.
    pub sda: Sda,
}

// internally, we would rather use this structure
type PartsTuple<Timer, Scl, Sda> = (Timer, (Scl, Sda));

/// Storage for a shared I2C bus.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SharedI2cController<Timer, Scl, Sda> {
    parts: Mutex<RefCell<PartsTuple<Timer, Scl, Sda>>>,
}

/// A shared I2C bus.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SharedI2c<'a, Timer, Scl, Sda> {
    parts: &'a Mutex<RefCell<PartsTuple<Timer, Scl, Sda>>>,
}

/// Create a shared I2C bus from parts.
pub fn new<Timer, Scl, Sda>(parts: Parts<Timer, Scl, Sda>) -> SharedI2cController<Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin + InputPin,
{
    SharedI2cController::new(parts)
}

impl<Timer, Scl, Sda> SharedI2cController<Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin + InputPin,
{
    /// Create a shared I2C bus from parts.
    pub fn new(parts: Parts<Timer, Scl, Sda>) -> Self {
        Self {
            parts: Mutex::new(RefCell::new((parts.clk, (parts.scl, parts.sda)))),
        }
    }

    /// Free the shared I2C bus and recover the parts.
    pub fn free(self) -> Parts<Timer, Scl, Sda> {
        let (clk, (scl, sda)) = self.parts.into_inner().into_inner();
        Parts { clk, scl, sda }
    }

    /// Acquire an instance of the shared I2C bus.
    pub fn acquire(&self) -> SharedI2c<'_, Timer, Scl, Sda> {
        SharedI2c { parts: &self.parts }
    }
}

// we need to implement some hal traits on RefMut, use a newtype
#[repr(transparent)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct Wrap<'a, T>(RefMut<'a, T>);

impl<'a, Timer> CountDown for Wrap<'a, Timer>
where
    Timer: CountDown,
{
    type Time = Timer::Time;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        self.0.start(count)
    }

    fn wait(&mut self) -> block::Result<(), void::Void> {
        self.0.wait()
    }
}

impl<'a, Timer> Periodic for Wrap<'a, Timer> where Timer: Periodic {}

impl<'a, Pin> InputPin for Wrap<'a, Pin>
where
    Pin: InputPin,
{
    type Error = Pin::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.0.is_high()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.0.is_low()
    }
}

impl<'a, Pin> OutputPin for Wrap<'a, Pin>
where
    Pin: OutputPin,
{
    type Error = Pin::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }

    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        self.0.set_state(state)
    }
}

/// The inner type of a shared I2C bus, used for raw operations.
///
/// Prefer to use the embedded-hal traits when possible.
#[repr(transparent)]
pub struct SharedI2cRaw<'a, Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin + InputPin,
{
    // use a newtype here ust to hide Wrap type
    bb: I2cBB<Wrap<'a, Scl>, Wrap<'a, Sda>, Wrap<'a, Timer>>,
}

/// Provides raw I2C bus primitives.
pub trait I2cRaw {
    /// The Error type this I2C bus produces.
    type Error;

    /// Send a raw I2C start.
    fn start_raw(&mut self) -> Result<(), Self::Error>;

    /// Send a raw I2C stop.
    fn stop_raw(&mut self) -> Result<(), Self::Error>;

    /// Read raw bytes from the bus.
    fn read_raw(&mut self, input: &mut [u8]) -> Result<(), Self::Error>;

    /// Write raw bytes to the bus.
    fn write_raw(&mut self, output: &[u8]) -> Result<(), Self::Error>;
}

impl<Timer, Scl, Sda> I2cRaw for I2cBB<Scl, Sda, Timer>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin<Error = Scl::Error> + InputPin<Error = Scl::Error>,
{
    type Error = BBError<Scl::Error>;

    fn start_raw(&mut self) -> Result<(), Self::Error> {
        self.raw_i2c_start()
    }

    fn stop_raw(&mut self) -> Result<(), Self::Error> {
        self.raw_i2c_stop()
    }

    fn read_raw(&mut self, input: &mut [u8]) -> Result<(), Self::Error> {
        self.raw_read_from_slave(input)
    }

    fn write_raw(&mut self, output: &[u8]) -> Result<(), Self::Error> {
        self.raw_write_to_slave(output)
    }
}

impl<'a, Timer, Scl, Sda> I2cRaw for SharedI2cRaw<'a, Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin<Error = Scl::Error> + InputPin<Error = Scl::Error>,
{
    type Error = Error;

    fn start_raw(&mut self) -> Result<(), Self::Error> {
        Ok(self.bb.raw_i2c_start()?)
    }

    fn stop_raw(&mut self) -> Result<(), Self::Error> {
        Ok(self.bb.raw_i2c_stop()?)
    }

    fn read_raw(&mut self, input: &mut [u8]) -> Result<(), Self::Error> {
        Ok(self.bb.raw_read_from_slave(input)?)
    }

    fn write_raw(&mut self, output: &[u8]) -> Result<(), Self::Error> {
        Ok(self.bb.raw_write_to_slave(output)?)
    }
}

impl<'a, Timer, Scl, Sda> SharedI2c<'a, Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin<Error = Scl::Error> + InputPin<Error = Scl::Error>,
{
    fn borrow_bitbang<'cs>(&self, cs: CriticalSection<'cs>) -> SharedI2cRaw<'cs, Timer, Scl, Sda>
    where
        'a: 'cs,
    {
        let parts = self.parts.borrow_ref_mut(cs);
        let (timer, rest) = RefMut::map_split(parts, |parts| (&mut parts.0, &mut parts.1));
        let (scl, sda) = RefMut::map_split(rest, |rest| (&mut rest.0, &mut rest.1));

        SharedI2cRaw {
            bb: I2cBB::new(Wrap(scl), Wrap(sda), Wrap(timer)),
        }
    }

    /// Borrow the I2C bus for raw access.
    ///
    /// Whenever possible, prefer to use the embedded-hal traits instead.
    pub fn with_raw<R>(&self, f: impl FnOnce(&mut SharedI2cRaw<Timer, Scl, Sda>) -> R) -> R {
        with(|cs| f(&mut self.borrow_bitbang(cs)))
    }
}

impl<'a, Timer, Scl, Sda> hal02::Read for SharedI2c<'a, Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin<Error = Scl::Error> + InputPin<Error = Scl::Error>,
{
    type Error = Error;

    fn read(&mut self, addr: u8, input: &mut [u8]) -> Result<(), Self::Error> {
        self.with_raw(|raw| Ok(raw.bb.read(addr, input)?))
    }
}

impl<'a, Timer, Scl, Sda> hal02::Write for SharedI2c<'a, Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin<Error = Scl::Error> + InputPin<Error = Scl::Error>,
{
    type Error = Error;

    fn write(&mut self, addr: u8, output: &[u8]) -> Result<(), Self::Error> {
        self.with_raw(|raw| Ok(raw.bb.write(addr, output)?))
    }
}

impl<'a, Timer, Scl, Sda> hal02::WriteRead for SharedI2c<'a, Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin<Error = Scl::Error> + InputPin<Error = Scl::Error>,
{
    type Error = Error;

    fn write_read(&mut self, addr: u8, output: &[u8], input: &mut [u8]) -> Result<(), Self::Error> {
        self.with_raw(|raw| Ok(raw.bb.write_read(addr, output, input)?))
    }
}

impl hal1::Error for Error {
    fn kind(&self) -> hal1::ErrorKind {
        match self {
            Self::Bus => hal1::ErrorKind::Bus,
            Self::NoAck => hal1::ErrorKind::NoAcknowledge(hal1::NoAcknowledgeSource::Unknown),
            Self::InvalidData => hal1::ErrorKind::Other,
        }
    }
}

impl<'a, Timer, Scl, Sda> hal1::ErrorType for SharedI2c<'a, Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin<Error = Scl::Error> + InputPin<Error = Scl::Error>,
{
    type Error = Error;
}

impl<'a, Timer, Scl, Sda> hal1::I2c for SharedI2c<'a, Timer, Scl, Sda>
where
    Timer: CountDown + Periodic,
    Scl: OutputPin,
    Sda: OutputPin<Error = Scl::Error> + InputPin<Error = Scl::Error>,
{
    fn transaction(
        &mut self,
        address: hal1::SevenBitAddress,
        operations: &mut [hal1::Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.with_raw(|raw| {
            // FIXME repeated reads, or reads not at the end, probably fail

            // ST
            raw.start_raw()?;

            let mut last_op_write = None;
            for op in operations.iter_mut() {
                match op {
                    hal1::Operation::Read(buf) => {
                        if last_op_write != Some(false) {
                            if last_op_write.is_some() {
                                // SR
                                raw.start_raw()?;
                            }

                            // SAD+R
                            raw.write_raw(&[(address << 1) | 0x1])?;
                        }
                        last_op_write = Some(false);

                        raw.read_raw(buf)?;
                    }
                    hal1::Operation::Write(buf) => {
                        if last_op_write != Some(true) {
                            if last_op_write.is_some() {
                                // SR
                                raw.start_raw()?;
                            }

                            // SAD+W
                            raw.write_raw(&[address << 1])?;
                        }
                        last_op_write = Some(true);

                        raw.write_raw(buf)?;
                    }
                }
            }

            // SP
            raw.stop_raw()
        })
    }
}
