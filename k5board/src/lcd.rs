//! LCD control.

use core::cell::UnsafeCell;
use core::convert::Infallible;

use display_interface_spi::SPIInterface;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{OriginDimensions, Size};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::primitives::Rectangle;
use embedded_graphics_core::Pixel;
use embedded_hal_02::blocking::delay::DelayMs;
use st7565::modes::GraphicsMode;
use st7565::types::{BoosterRatio, PowerControlMode};
use st7565::{GraphicsPageBuffer, ST7565};

use crate::hal::gpio::alt::spi0;
use crate::hal::gpio::{Alternate, Output, PushPull, PB10, PB11, PB7, PB8, PB9};
use crate::hal::power::Gate;
use crate::hal::spi;
use crate::pac::portcon::portb_sel1;
use crate::pac::SPI0;

/// The width of the LCD.
pub const WIDTH: usize = 128;

/// The height of the LCD.
pub const HEIGHT: usize = 64;

/// Number of pages in the display buffer.
const PAGES: usize = HEIGHT / 8;

/// The pins and peripherals required for the LCD.
#[derive(Debug)]
// defmt not implemented for SPI0 (??)
pub struct Parts {
    /// The SPI0 peripheral.
    pub spi: SPI0,
    /// The gate controlling SPI0 power.
    pub gate: Gate<SPI0>,
    /// The LCD chip select pin.
    pub cs: PB7<Output<PushPull>>,
    /// The LCD clock pin.
    pub clk: PB8<Alternate<{ portb_sel1::PORTB8_A::Spi0Clk as u8 }, Output<PushPull>>>,
    /// The LCD A0 pin.
    pub a0: PB9<Output<PushPull>>,
    /// The LCD serial in pin.
    pub mosi: PB10<Alternate<{ portb_sel1::PORTB10_A::Spi0Mosi as u8 }, Output<PushPull>>>,
    /// The LCD reset pin.
    pub res: PB11<Output<PushPull>>,
}

/// The LCD's display spec for st7565 crate.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct DisplaySpec;

impl st7565::DisplaySpecs<WIDTH, HEIGHT, PAGES> for DisplaySpec {
    const FLIP_ROWS: bool = false;
    const FLIP_COLUMNS: bool = true;
    const INVERTED: bool = false;
    const BIAS_MODE_1: bool = false;
    const POWER_CONTROL: PowerControlMode = PowerControlMode {
        booster_circuit: true,
        voltage_regulator_circuit: true,
        voltage_follower_circuit: true,
    };
    const VOLTAGE_REGULATOR_RESISTOR_RATIO: u8 = 0x4;
    const ELECTRONIC_VOLUME: u8 = 0x1f;
    const BOOSTER_RATIO: BoosterRatio = BoosterRatio::StepUp2x3x4x;
    const COLUMN_OFFSET: u8 = 4;
}

/// The LCD interface.
///
/// This can be used as a draw target for the embedded-graphics crate.
pub struct Lcd {
    #[allow(clippy::type_complexity)]
    lcd: ST7565<
        SPIInterface<spi::MasterTx<SPI0>, PB9<Output<PushPull>>, PB7<Output<PushPull>>>,
        DisplaySpec,
        GraphicsMode<'static, WIDTH, PAGES>,
        WIDTH,
        HEIGHT,
        PAGES,
    >,
    res: PB11<Output<PushPull>>,
}

/// An LCD error.
pub type Error = display_interface::DisplayError;

/// Create a new LCD from parts.
pub fn new<Delay>(delay: &mut Delay, parts: Parts) -> Result<Lcd, (Parts, Error)>
where
    Delay: DelayMs<u8>,
{
    Lcd::new(delay, parts)
}

impl Lcd {
    /// Create a new LCD from parts.
    pub fn new<Delay>(delay: &mut Delay, parts: Parts) -> Result<Self, (Parts, Error)>
    where
        Delay: DelayMs<u8>,
    {
        let spi = spi::new(parts.spi, parts.gate)
            .divider(spi::ClockDivider::Div16)
            .mode(spi::Mode::MODE_3)
            .bit_order(spi::BitOrder::Msb)
            .master_tx(parts.clk.into(), parts.mosi.into());

        let lcd_interface = SPIInterface::new(spi, parts.a0, parts.cs);

        // use a static backing buffer, no matter where this struct ends up
        static mut PAGE_BUFFER: UnsafeCell<GraphicsPageBuffer<WIDTH, PAGES>> =
            UnsafeCell::new(GraphicsPageBuffer::new());

        // safety: we possess multiple unique tokens that ensure this static
        // cannot be borrowed more than once.
        // free() returns these tokens while also dropping this borrow.
        // we are relying on st7565 crate not to stash this reference
        // somewhere unexpected.
        let lcd = unsafe {
            ST7565::new(lcd_interface, DisplaySpec)
                .into_graphics_mode(PAGE_BUFFER.get().as_mut().unwrap())
        };

        let mut lcd = Self {
            lcd,
            res: parts.res,
        };

        if let Err(e) = lcd.reset(delay) {
            return Err((lcd.free(), e));
        }

        if let Err(e) = lcd.flush() {
            return Err((lcd.free(), e));
        }

        if let Err(e) = lcd.set_display_on(true) {
            return Err((lcd.free(), e));
        }

        Ok(lcd)
    }

    /// Free the components of the LCD.
    pub fn free(self) -> Parts {
        let (_, lcd_interface) = self.lcd.release_display_interface();
        let (spi, a0, cs) = lcd_interface.release();
        let (config, clk, (), mosi, ()) = spi.free();
        let (spi, gate) = config.free();
        Parts {
            spi,
            gate,
            cs,
            clk: match clk {
                spi0::Clk::PB8(pb8) => pb8,
                _ => unreachable!(), // not set in new()
            },
            a0,
            mosi: match mosi {
                spi0::Mosi::PB10(pb10) => pb10,
                _ => unreachable!(), // not set in new()
            },
            res: self.res,
        }
    }

    /// Sets the line offset, effectively scrolling the display through memory.
    pub fn set_line_offset(&mut self, offset: u8) -> Result<(), Error> {
        self.lcd.set_line_offset(offset)
    }

    /// Sets whether the pixels should be inverted.
    pub fn set_inverted(&mut self, inverted: bool) -> Result<(), Error> {
        self.lcd.set_inverted(inverted)
    }

    /// Displays all points of the display.
    pub fn display_all_points(&mut self, enable: bool) -> Result<(), Error> {
        self.lcd.display_all_points(enable)
    }

    /// Enable/disable the display output.
    pub fn set_display_on(&mut self, on: bool) -> Result<(), Error> {
        self.lcd.set_display_on(on)
    }

    /// Reset the LCD.
    pub fn reset<Delay>(&mut self, delay: &mut Delay) -> Result<(), Error>
    where
        Delay: DelayMs<u8>,
    {
        self.lcd.reset(&mut self.res, delay).map_err(|e| match e {
            st7565::Error::Comm(comm) => comm,
            st7565::Error::Pin(pin) => match pin {},
        })
    }

    /// Write the LCD framebuffer to the screen.
    pub fn flush(&mut self) -> Result<(), Error> {
        self.lcd.flush()
    }
}

impl OriginDimensions for Lcd {
    fn size(&self) -> Size {
        self.lcd.size()
    }
}

impl DrawTarget for Lcd {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.lcd.draw_iter(pixels)
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.lcd.fill_contiguous(area, colors)
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.lcd.fill_solid(area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.lcd.clear(color)
    }
}
