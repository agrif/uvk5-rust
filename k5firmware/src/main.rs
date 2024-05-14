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
    let mut uart = hal::uart::new(p.UART1, power.gates.uart1)
        .baud(&clocks, 38_400.Hz())
        .unwrap()
        .port(pins_a.a8.into(), pins_a.a7.into())
        .tx;

    // timer test
    let mut timer = hal::timer::new(p.TIMER_BASE0, power.gates.timer_base0)
        .frequency::<{ Hertz::kHz(1).to_Hz() }>(&clocks)
        .unwrap()
        .split(&clocks)
        .high
        .counter();

    // turn on flashlight
    light.set_high();

    loop {
        timer.delay(500.millis()).unwrap();

        // ptt pressed means ptt low
        // ptt pressed means toggle light
        if ptt.is_low() {
            light.toggle();
        }

        use core::fmt::Write;
        writeln!(&mut uart, "Hello, {}!", "UV-K5").unwrap();
        writeln!(&mut uart, "PTT is {:?} {:?}", ptt, ptt.read()).unwrap();
        writeln!(&mut uart, "Light is {:?} {:?}", light, light.get_state()).unwrap();
    }
}
