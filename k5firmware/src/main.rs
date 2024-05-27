#![no_std]
#![no_main]

extern crate alloc;
use panic_halt as _;

use core::fmt::Write;

use k5board::hal;
use k5board::prelude::*;

use hal::gpio::InputOutputPin;
use hal::time::Hertz;

pub mod bk1080;

k5board::version!(env!("CARGO_PKG_VERSION"));

#[global_allocator]
static ALLOCATOR: alloc_cortex_m::CortexMHeap = alloc_cortex_m::CortexMHeap::empty();
const HEAP_SIZE: usize = 1024;

static UART_RX: spin::Once<spin::Mutex<hal::uart::Rx<hal::pac::UART1>>> = spin::Once::new();
static UART_TX: spin::Once<spin::Mutex<hal::uart::Tx<hal::pac::UART1>>> = spin::Once::new();

macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

macro_rules! print {
    ($($arg:tt)*) => {
        // only try to lock.. if it's locked, it's likely never to unlock
        // this is a best effort deal
        if let Some(mutex) = UART_TX.get() {
            if let Some(mut guard) = mutex.try_lock() {
                write!(guard, "{}", format_args!($($arg)*)).unwrap();
            }
        }
    }
}

struct NoPin;

impl embedded_hal_02::digital::v2::InputPin for NoPin {
    type Error = core::convert::Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringError(alloc::string::String);

impl StringError {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for StringError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<E> From<E> for StringError
where
    E: core::fmt::Debug,
{
    fn from(value: E) -> Self {
        StringError(alloc::format!("{:?}", value))
    }
}

impl core::ops::Deref for StringError {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

fn reset() -> ! {
    println!("!!! reset !!!");
    if let Some(txmutex) = UART_TX.get() {
        if let Some(mut txguard) = txmutex.try_lock() {
            hal::block::block!(txguard.flush()).unwrap();
        }
    }
    cortex_m::peripheral::SCB::sys_reset();
}

#[cortex_m_rt::entry]
fn main() -> ! {
    match go() {
        Ok(()) => loop {
            cortex_m::asm::wfi();
        },
        Err(e) => loop {
            println!("error in go(), press any key to reset");
            println!("{}", e);

            // reset if character received
            if let Some(mutex) = UART_RX.get() {
                if let Some(mut guard) = mutex.try_lock() {
                    if guard.read_one().is_ok() {
                        reset();
                    }
                }
            }

            // don't spam the uart
            cortex_m::asm::delay(10_000_000);
        },
    }
}

fn go() -> Result<(), StringError> {
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU);

    let clocks = power.clocks.sys_internal_48mhz().freeze();

    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_a = ports.port_a.enable(power.gates.gpio_a);
    let pins_b = ports.port_b.enable(power.gates.gpio_b);
    let pins_c = ports.port_c.enable(power.gates.gpio_c);

    // fast track the uart (tx A7, rx A8)
    let uart_tx = pins_a.a7.into();
    let uart_rx = pins_a.a8.into();
    let uart =
        hal::uart::new(p.UART1, power.gates.uart1, &clocks, 38_400.Hz())?.port(uart_rx, uart_tx);
    UART_RX.call_once(|| uart.rx.into());
    UART_TX.call_once(|| uart.tx.into());

    // PA3 keypad column 1
    let col1 = pins_a.a3.into_pull_up_input();
    // PA4 keypad column 2
    let col2 = pins_a.a4.into_pull_up_input();
    // PA5 keypad column 3
    // PA6 keypad column 4

    // PA9 battery voltage

    // PA10 keypad row 1 / eeprom scl
    let eeprom_scl = pins_a.a10.into_open_drain_output();
    // PA11 keypad row 2 / eeprom sda
    let eeprom_sda = InputOutputPin::new_from_output(pins_a.a11.into_open_drain_output(), |p| {
        p.into_floating_input()
    })
    .default_high();
    // PA12 keypad row 3 / voice 0
    // PA13 keypad row 4 / voice 1

    // PA14 battery current

    // PB6 backlight
    let mut backlight = k5board::backlight::new(pins_b.b6.into_mode());

    let lcd_parts = k5board::lcd::Parts {
        spi: p.SPI0,
        gate: power.gates.spi0,
        // PB7 ST7565 cs
        cs: pins_b.b7.into_mode(),
        // PB8 ST7565 clk
        clk: pins_b.b8.into_mode(),
        // PB9 ST7565 a0
        a0: pins_b.b9.into_mode(),
        // PB10 ST7565 si
        mosi: pins_b.b10.into_mode(),
        // PB11 ST7565 res / swdio / tp14
        res: pins_b.b11.into_mode(),
    };

    // PB14 BK4819 gpio2 / swclk / tp13
    // PB15 BK1080 rf on
    let mut fm_enable = pins_b.b15.into_push_pull_output();

    // PC0 BK4819 scn
    // PC1 BK4819 scl
    // PC2 BK4819 sda

    // PC3 flashlight
    let mut flashlight = k5board::flashlight::new(pins_c.c3.into_mode());
    // PC4 speaker amp on
    let mut speaker_enable = pins_c.c4.into_push_pull_output();
    // PC5 ptt
    let ptt = k5board::ptt::new(pins_c.c5.into_mode());

    // get a timer going at 200kHz i2c
    let timer200k = hal::timer::new(p.TIMER_BASE0, power.gates.timer_base0)
        .frequency::<{ Hertz::kHz(200).to_Hz() }>(&clocks)?
        .split(&clocks);
    let mut delay = timer200k.high.timing();

    // get a timer going at 1kHz for blinks and frames
    let timer1k = hal::timer::new(p.TIMER_BASE2, power.gates.timer_base2)
        .frequency::<{ Hertz::kHz(1).to_Hz() }>(&clocks)?
        .split(&clocks);

    // bitbang eeprom i2c at 100kHz (half the timer frequency)
    let mut i2c_timer = timer200k.low.timing();
    i2c_timer.start_native()?;
    let mut i2c = bitbang_hal::i2c::I2cBB::new(eeprom_scl, eeprom_sda, i2c_timer);
    let mut fm = bk1080::Bk1080::new(&mut i2c)?;
    //let mut eeprom = eeprom24x::Eeprom24x::new_24x64(i2c, eeprom24x::SlaveAddr::default());

    // the lcd display
    let mut lcd = k5board::lcd::new(&mut delay, lcd_parts)?;

    // draw a thing
    use embedded_graphics::mono_font::{ascii::FONT_4X6, ascii::FONT_6X10, MonoTextStyle};
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
    use embedded_graphics::text::{Alignment, Text};

    let font = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let fontsmall = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);
    Text::with_alignment(
        "Hello, UV-K5!",
        lcd.bounding_box().center() + Point::new(0, -20),
        font,
        Alignment::Center,
    )
    .draw(&mut lcd)?;

    lcd.flush()?;

    // turn off flashlight
    flashlight.off();

    // turn on backlight
    backlight.on();

    // turn on radio
    fm_enable.set_low(); // active low
    fm.enable()?;
    speaker_enable.set_high();

    let mut freq = 0;
    fm.tune(freq)?;

    let mut led_blink = timer1k.low.timing();
    led_blink.start_frequency(2.Hz())?;

    let mut update_display = timer1k.high.timing();
    update_display.start_frequency(30.Hz())?;

    // a buffer in which to store our serial data
    let line_buf = cortex_m::singleton!(LINE_BUF: [u8; 0x100] = [0; 0x100]).unwrap();
    let mut line_size = 0;

    // a snaking dot that moves across the screen
    let mut snake = 0;
    const SNAKE_LEN: i32 = 50;

    // last status text
    let mut last_text: Option<Rectangle> = None;

    // calculate a Point based on a snake value
    fn snake_point(border: Rectangle, mut snake: i32) -> Point {
        let (left, top) = border.top_left.into();
        let (width, height) = border.size.into();
        let width = width as i32 - 1;
        let height = height as i32 - 1;
        let right = left + width;
        let bottom = top + height;

        snake %= 2 * width + 2 * height;

        if snake < width {
            Point::new(left + snake, top)
        } else if snake < width + height {
            snake -= width;
            Point::new(right, top + snake)
        } else if snake < 2 * width + height {
            snake -= width + height;
            Point::new(right - snake, bottom)
        } else {
            snake -= 2 * width + height;
            Point::new(left, bottom - snake)
        }
    }

    // turn a string into a pin state
    fn pin_state(name: &str) -> Option<hal::gpio::PinState> {
        use hal::gpio::PinState::*;
        match name {
            "low" => Some(Low),
            "high" => Some(High),
            _ => None,
        }
    }

    loop {
        if let Ok(()) = led_blink.wait() {
            // ptt pressed means toggle flashlight
            if ptt.is_pressed() {
                flashlight.toggle();
                fm_enable.toggle();
            }

            // button 1 is pressed
            if col1.is_low() {
                freq += 1;
                fm.tune(freq)?;
            }

            // button 2 is pressed
            if col2.is_low() {
                freq -= 1;
                fm.tune(freq)?;
            }
        }

        if let Ok(()) = update_display.wait() {
            let text = alloc::format!(
                "freq {:?} rssi {:04x?}",
                875 + 2 * freq,
                0x0000 // fm.read(bk1080::REG_RSSI)?,
            );

            let text = Text::with_alignment(
                &text,
                lcd.bounding_box().center() + Point::new(0, 20),
                fontsmall,
                Alignment::Center,
            );

            if let Some(last_text) = last_text.take() {
                last_text
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                    .draw(&mut lcd)?;
            }

            text.draw(&mut lcd)?;
            last_text = Some(text.bounding_box());

            snake += 1;
            Pixel(snake_point(lcd.bounding_box(), snake), BinaryColor::On).draw(&mut lcd)?;
            Pixel(
                snake_point(lcd.bounding_box(), snake - SNAKE_LEN),
                BinaryColor::Off,
            )
            .draw(&mut lcd)?;
            lcd.flush()?;
        }

        if let Ok(c) = UART_RX.wait().lock().read_one() {
            if c == b'\r' {
                print!("\r\n");

                let line = core::str::from_utf8(&line_buf[..line_size]);
                line_size = 0;

                let Some(line) = line.ok() else {
                    continue;
                };

                let cmd = line
                    .split_once(|c: char| c.is_whitespace())
                    .unwrap_or((line, ""));

                match cmd {
                    ("hello", _) => {
                        println!("Hello!");
                    }
                    ("reset", _) => {
                        reset();
                    }
                    ("speaker", state) => {
                        let Some(state) = pin_state(state) else {
                            continue;
                        };
                        speaker_enable.set_state(state);
                    }
                    ("fm", "enable") => {
                        fm_enable.set_low();
                    }
                    ("fm", "disable") => {
                        fm_enable.set_high();
                    }
                    ("fm", "init") => {
                        println!("init: {:x?}", fm.enable());
                    }
                    ("tune", val) => {
                        let Ok(val) = val.parse::<u16>() else {
                            continue;
                        };
                        println!("tune: {:x?}", fm.tune(val));
                    }
                    ("fm", "") => {
                        let all = fm.update(..);
                        if let Ok(all) = all {
                            for (a, v) in all.iter().enumerate() {
                                println!("fm {:02x}: {:x?}", a, v);
                            }
                        } else {
                            println!("fm {:?}", all);
                        }
                    }
                    ("read", addr) => {
                        let Ok(addr) = u32::from_str_radix(addr, 16) else {
                            continue;
                        };
                        let mut eeprom_data = [0; 16];
                        //eeprom.read_data(addr, &mut eeprom_data[..])?;
                        println!("eeprom data: {:x?}", eeprom_data);
                    }
                    _ => {}
                }
                continue;
            }

            if line_size >= line_buf.len() {
                println!("overrun!");
                line_size = 0;
            }

            UART_TX.wait().lock().write_all(&[c])?;
            line_buf[line_size] = c;
            line_size += 1;
        }
    }
}
