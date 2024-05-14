#![no_std]
#![no_main]

use dp32g030_hal as hal;
use panic_halt as _;

use hal::prelude::*;
use hal::time::Hertz;

hal::version!(env!("CARGO_PKG_VERSION"));

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU);

    let clocks = power.clocks.sys_internal_48mhz().freeze();

    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_a = ports.port_a.enable(power.gates.gpio_a);
    let pins_c = ports.port_c.enable(power.gates.gpio_c);

    // flashlight is GPIO C3
    let mut light = pins_c.c3.erase().into_push_pull_output();

    // ptt button is GPIO C5
    let ptt = pins_c.c5.erase().into_pull_up_input();

    // uart1 tx is A7, uart1 rx is A8
    let mut uart = hal::uart::new(p.UART1, power.gates.uart1, &clocks, 38_400.Hz())
        .unwrap()
        .port(pins_a.a8.into(), pins_a.a7.into());

    // timer test
    let mut timer = hal::timer::new(p.TIMER_BASE0, power.gates.timer_base0)
        .frequency::<{ Hertz::kHz(1).to_Hz() }>(&clocks)
        .unwrap()
        .split(&clocks)
        .high
        .counter();

    // turn on flashlight
    light.set_high();

    // set the timer to complete every 500ms
    timer.start(500.millis()).unwrap();

    // a buffer in which to store our serial data
    let mut line_buf = [0; 0x100];
    let mut line_size = 0;

    loop {
        // ptt pressed means ptt low
        // ptt pressed means toggle light
        if ptt.is_low() {
            light.toggle();
        }

        // handle serial until the timer expires
        use core::fmt::Write;
        while !matches!(timer.wait(), Ok(())) {
            if let Ok(c) = uart.rx.read_one() {
                if c == b'\r' {
                    uart.tx.write_all(b"\r\n".as_ref()).unwrap();
                    let line = core::str::from_utf8(&line_buf[..line_size]);
                    writeln!(uart.tx, "got line: {:?}", line).unwrap();
                    line_size = 0;
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
