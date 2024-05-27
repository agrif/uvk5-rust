#![no_std]
#![no_main]

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Text};

use k5board::hal;
use panic_halt as _;

use hal::time::Hertz;

k5board::version!(concat!(env!("CARGO_PKG_VERSION"), "lcd"));

#[cortex_m_rt::entry]
fn main() -> ! {
    // grab peripherals and initialize the clock
    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU);
    let clocks = power.clocks.sys_internal_24mhz().freeze();

    // turn on GPIOB
    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_b = ports.port_b.enable(power.gates.gpio_b);

    // turn TIMER_BASE0 into a 1kHz resolution timer for delays
    let mut delay = hal::timer::new(p.TIMER_BASE0, power.gates.timer_base0)
        .frequency::<{ Hertz::kHz(1).to_Hz() }>(&clocks)
        .unwrap()
        .split(&clocks)
        .low
        .timing();

    // set up backlight and lcd
    let mut backlight = k5board::backlight::new(pins_b.b6);
    let lcd_parts = k5board::lcd::Parts {
        spi: p.SPI0,
        gate: power.gates.spi0,
        cs: pins_b.b7.into_mode(),
        clk: pins_b.b8.into_mode(),
        a0: pins_b.b9.into_mode(),
        mosi: pins_b.b10.into_mode(),
        res: pins_b.b11.into_mode(),
    };
    let mut lcd = k5board::lcd::new(&mut delay, lcd_parts).unwrap();

    // draw some text
    let font = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::with_alignment(
        "Hello, UV-K5!",
        lcd.bounding_box().center(),
        font,
        Alignment::Center,
    )
    .draw(&mut lcd)
    .unwrap();
    lcd.flush().unwrap();

    // turn on the backlight
    backlight.on();

    // do nothing forever so we can admire our lcd
    loop {
        cortex_m::asm::wfi();
    }
}
