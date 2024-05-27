#![no_std]
#![no_main]

use k5board::hal;
use panic_halt as _;

use hal::prelude::*;
use hal::time::Hertz;

hal::version!(concat!(env!("CARGO_PKG_VERSION"), "blink"));

#[cortex_m_rt::entry]
fn main() -> ! {
    // grab peripherals and initialize the clock
    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU);
    let clocks = power.clocks.sys_internal_24mhz().freeze();

    // turn on GPIOC and grab our LED on pin C3 as an output
    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_c = ports.port_c.enable(power.gates.gpio_c);
    let mut led = pins_c.c3.into_push_pull_output();

    // turn TIMER_BASE0 into a 1kHz resolution timer, and use the Low half
    let mut timer = hal::timer::new(p.TIMER_BASE0, power.gates.timer_base0)
        .frequency::<{ Hertz::kHz(1).to_Hz() }>(&clocks)
        .unwrap()
        .split(&clocks)
        .low
        .timing();

    // it's blinkin' time
    loop {
        // wait half a second
        timer.delay(500.millis()).unwrap();

        // blink led
        led.toggle();
    }
}
