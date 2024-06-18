#![no_std]
#![no_main]

use panic_halt as _;

use k5board::hal;
use k5board::prelude::*;

use hal::time::Hertz;

k5board::version!(concat!(env!("CARGO_PKG_VERSION"), "blink"));

#[cortex_m_rt::entry]
fn main() -> ! {
    // grab peripherals and initialize the clock
    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU)
        .sys_internal_24mhz()
        .freeze();

    // turn on GPIOC and grab our flashlight on pin C3
    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_c = ports.port_c.enable(power.gates.gpio_c);
    let mut flashlight = k5board::flashlight::new(pins_c.c3.into_mode());

    // turn TIMER_BASE0 into a 1kHz resolution timer, and use the Low half
    let mut timer = hal::timer::new(p.TIMER_BASE0, power.gates.timer_base0)
        .frequency::<{ Hertz::kHz(1).to_Hz() }>()
        .unwrap()
        .split()
        .low
        .timing();

    // it's blinkin' time
    loop {
        // wait half a second
        timer.delay(500.millis()).unwrap();

        // blink flashlight
        flashlight.toggle();
    }
}
