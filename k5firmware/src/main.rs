#![no_std]
#![no_main]

use dp32g030_hal as hal;
use panic_halt as _;

use hal::prelude::*;

use hal::gpio::InputOutputPin;
use hal::time::Hertz;

hal::version!(env!("CARGO_PKG_VERSION"));

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
    // PB8 ST7565 clk
    // PB9 ST7565 a0
    // PB10 ST7565 si
    // PB11 ST7565 res / swdio / tp14

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
    let timer = hal::timer::new(p.TIMER_BASE0, power.gates.timer_base0)
        .frequency::<{ Hertz::kHz(100).to_Hz() }>(&clocks)
        .unwrap()
        .split(&clocks);

    // bitbang eeprom i2c at 100kHz
    let mut i2c_timer = timer.low.counter();
    i2c_timer.start(Hertz::kHz(100).into_duration()).unwrap();
    let i2c = bitbang_hal::i2c::I2cBB::new(eeprom_scl, eeprom_sda, i2c_timer);
    let mut eeprom = eeprom24x::Eeprom24x::new_24x64(i2c, eeprom24x::SlaveAddr::default());

    // delay timer
    let mut delay = timer.high.counter();

    // turn on flashlight
    flashlight.set_high();

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
