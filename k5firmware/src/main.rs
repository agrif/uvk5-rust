#![no_std]
#![no_main]

use dp32g030_hal as hal;
use panic_halt as _;

use hal::prelude::*;

use hal::gpio::InputOutputPin;
use hal::time::Hertz;

hal::version!(env!("CARGO_PKG_VERSION"));

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

struct DisplaySpec;

impl st7565::DisplaySpecs<128, 64, 8> for DisplaySpec {
    // original firmware initialization sequence:

    // 0xE2: Reset

    // delay 120 ms

    // 0xA2: LCD Bias Set 0 (BIAS_MODE_1 = false)
    // 0xC0: Common Mode Output Select Normal (ROW_FLIP = false)
    // 0xA1: ADC Select Reverse (COLUMN_FLIP = true)
    // 0xA6: Display Normal (INVERTED = false)
    // 0xA4: Display All Points Off
    // 0x24: Voltage Internal Resistor Ratio set to 0x4
    // 0x81: Electronic Volume Mode Set Byte 1
    // 0x1F: Electronic Volume Mode Set Byte 2, set to 0x1f
    // 0x2B: 0b0010 0b1011 booster off, regulator on, follower on

    // delay 1 ms

    // 0x2E: 0b0010 0b1110 booster on, regulator on, follower off

    // delay 1 ms

    // 0x2F: 0b0010 0b1111 booster on, regulator on, follower on
    // 0x2F: **
    // 0x2F: **

    // delay 40 ms

    // 0x40: Display Line Start Set 0
    // 0xAF: Display ON

    // end of sequence

    // 0xC0 | (flip_rows << 3), send 0xC0 means false
    const FLIP_ROWS: bool = false;

    // 0xA0 | flip_columns, send 0xA1 means true
    const FLIP_COLUMNS: bool = true;

    // 0xA6 | inverted, send 0xA6 means false
    const INVERTED: bool = false;

    // 0xA2 | bias_mode_1, send 0xA2 means false
    const BIAS_MODE_1: bool = false;

    // 0x28 | (booster << 2) | (regulator << 1) | follower
    // send 0x2f means booster, regulator, follower all 1
    const POWER_CONTROL: st7565::types::PowerControlMode = st7565::types::PowerControlMode {
        booster_circuit: true,
        voltage_regulator_circuit: true,
        voltage_follower_circuit: true,
    };

    // 0x20 | (resistor_ratio & 0b111), send 0x24 means 0x4
    const VOLTAGE_REGULATOR_RESISTOR_RATIO: u8 = 0x4;

    // 0x81 (volume & 0b0011_1111), send 0x81 0x1f means 0x1f
    const ELECTRONIC_VOLUME: u8 = 0x1f;

    // 0xF8 booster_ratio, send .. nothing. hm.
    // this appears to be an internal command??
    // go with the most 0 one
    const BOOSTER_RATIO: st7565::types::BoosterRatio = st7565::types::BoosterRatio::StepUp2x3x4x;
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU);

    let clocks = power.clocks.sys_internal_48mhz().freeze();

    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_a = ports.port_a.enable(power.gates.gpio_a);
    let pins_b = ports.port_b.enable(power.gates.gpio_b);
    let pins_c = ports.port_c.enable(power.gates.gpio_c);

    // PA3 keypad column 1
    // PA4 keypad column 2
    // PA5 keypad column 3
    // PA6 keypad column 4

    // PA7 uart1 tx
    let uart_tx = pins_a.a7.into();
    // PA8 uart1 rx
    let uart_rx = pins_a.a8.into();

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
    let mut backlight = pins_b.b6.into_push_pull_output();

    // PB7 ST7565 ??? P10
    let lcd_cs = pins_b.b7.into_push_pull_output();
    // PB8 ST7565 clk
    let lcd_clk = pins_b.b8.into_push_pull_output();
    // PB9 ST7565 a0
    let lcd_a0 = pins_b.b9.into_push_pull_output();
    // PB10 ST7565 si
    let lcd_mosi = pins_b.b10.into_push_pull_output();
    // PB11 ST7565 res / swdio / tp14
    let mut lcd_res = pins_b.b11.into_push_pull_output();

    // PB14 BK4819 gpio2 / swclk / tp13
    // PB15 BK1080 rf on

    // PC0 BK4819 scn
    // PC1 BK4819 scl
    // PC2 BK4819 sda

    // PC3 flashlight
    let mut flashlight = pins_c.c3.into_push_pull_output();
    // PC4 speaker amp on
    // PC5 ptt
    let ptt = pins_c.c5.into_pull_up_input();

    // initialize uart
    let mut uart = hal::uart::new(p.UART1, power.gates.uart1, &clocks, 38_400.Hz())
        .unwrap()
        .port(uart_rx, uart_tx);

    // get a timer going at 100kHz for delays and i2c
    let timer100k = hal::timer::new(p.TIMER_BASE0, power.gates.timer_base0)
        .frequency::<{ Hertz::kHz(100).to_Hz() }>(&clocks)
        .unwrap()
        .split(&clocks);

    // get a timer going at 1MHz for SPI
    // I *think* SPI can run at 8MHz or higher, but this is bit banged
    // so lets go slow
    let timer1m = hal::timer::new(p.TIMER_BASE1, power.gates.timer_base1)
        .frequency::<{ Hertz::MHz(1).to_Hz() }>(&clocks)
        .unwrap()
        .split(&clocks);

    // bitbang eeprom i2c at 50kHz (half the timer frequency)
    let mut i2c_timer = timer100k.low.counter();
    i2c_timer.start_rate(100.kHz()).unwrap();
    let i2c = bitbang_hal::i2c::I2cBB::new(eeprom_scl, eeprom_sda, i2c_timer);
    let mut eeprom = eeprom24x::Eeprom24x::new_24x64(i2c, eeprom24x::SlaveAddr::default());

    // delay timer
    let mut delay = timer100k.high.counter();

    // bitbang spi at 500kHz (half the timer frequency)
    let mut spi_timer = timer1m.low.counter();
    spi_timer.start_rate(1.MHz()).unwrap();
    let spi = bitbang_hal::spi::SPI::new(
        bitbang_hal::spi::MODE_3,
        NoPin,
        lcd_mosi,
        lcd_clk,
        spi_timer,
    );

    // LCD setup
    let lcd_interface = display_interface_spi::SPIInterface::new(spi, lcd_a0, lcd_cs);
    let mut page_buffer = st7565::GraphicsPageBuffer::new();
    let mut lcd =
        st7565::ST7565::new(lcd_interface, DisplaySpec).into_graphics_mode(&mut page_buffer);
    lcd.reset(&mut lcd_res, &mut delay).unwrap();
    lcd.flush().unwrap();
    lcd.set_display_on(true).unwrap();

    // draw a thing
    use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics::prelude::*;
    use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
    use embedded_graphics::text::{Alignment, Text};

    let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let thick_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 2);
    let font = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Circle::new(Point::new(50, 30), 20)
        .into_styled(thin_stroke)
        .draw(&mut lcd)
        .unwrap();
    Rectangle::new(Point::new(80, 30), Size::new(20, 16))
        .into_styled(thick_stroke)
        .draw(&mut lcd)
        .unwrap();
    Text::with_alignment(
        "Hello, UV-K5!",
        lcd.bounding_box().center() + Point::new(0, -20),
        font,
        Alignment::Center,
    )
    .draw(&mut lcd)
    .unwrap();

    lcd.flush().unwrap();

    // turn off flashlight
    flashlight.set_low();

    // turn on backlight
    backlight.set_high();

    // set the timer to complete every 500ms
    delay.start(500.millis()).unwrap();

    // a buffer in which to store our serial data
    let mut line_buf = [0; 0x100];
    let mut line_size = 0;

    loop {
        // ptt pressed means ptt low
        // ptt pressed means toggle flashlight
        if ptt.is_low() {
            flashlight.toggle();
        }

        // handle serial until the timer expires
        use core::fmt::Write;
        while !matches!(delay.wait(), Ok(())) {
            if let Ok(c) = uart.rx.read_one() {
                if c == b'\r' {
                    uart.tx.write_all(b"\r\n".as_ref()).unwrap();
                    let line = core::str::from_utf8(&line_buf[..line_size]);
                    line_size = 0;

                    // read the given address
                    if let Some(addr) = line.ok().and_then(|s| u32::from_str_radix(s, 16).ok()) {
                        let mut eeprom_data = [0; 16];
                        eeprom.read_data(addr, &mut eeprom_data[..]).unwrap();
                        writeln!(uart.tx, "eeprom data: {:x?}", eeprom_data).unwrap();
                    }
                    continue;
                }

                if line_size >= line_buf.len() {
                    writeln!(uart.tx, "overrun!").unwrap();
                    line_size = 0;
                }

                uart.tx.write_all(&[c]).unwrap();
                line_buf[line_size] = c;
                line_size += 1;
            }
        }
    }
}
