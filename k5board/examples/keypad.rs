#![no_std]
#![no_main]

use k5board::hal;
use panic_halt as _;

use k5board::prelude::*;

k5board::version!(concat!(env!("CARGO_PKG_VERSION"), "keypad"));

#[cortex_m_rt::entry]
fn main() -> ! {
    // grab peripherals and initialize the clock
    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU)
        .sys_internal_24mhz()
        .freeze();

    // turn on GPIOA and GPIOC
    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_a = ports.port_a.enable(power.gates.gpio_a);
    let pins_c = ports.port_c.enable(power.gates.gpio_c);

    // set up the uart and install it globally so we can print keypresses
    let uart_parts = k5board::uart::Parts {
        uart: p.UART1,
        gate: power.gates.uart1,
        tx: pins_a.a7.into_mode(),
        rx: pins_a.a8.into_mode(),
    };
    let uart = k5board::uart::new(38_400.Hz(), uart_parts).unwrap();
    k5board::uart::install(uart);

    // set up the keypad
    let keypad_parts = k5board::keypad::Parts {
        ptt: pins_c.c5.into_mode(),
        row: (
            pins_a.a3.into_mode(),
            pins_a.a4.into_mode(),
            pins_a.a5.into_mode(),
            pins_a.a6.into_mode(),
        ),
        col: (
            pins_a.a10.into_mode().into(),
            pins_a.a11.into_mode().into(),
            pins_a.a12.into_mode().into(),
            pins_a.a13.into_mode().into(),
        ),
    };
    let mut keypad = k5board::keypad::new(keypad_parts);

    loop {
        let keys = keypad.poll();
        if !keypad.changed().is_empty() {
            println!("   down: {:?}", keys);
            println!("     up: {:?}", keypad.up());
            println!("pressed: {:?}", keypad.pressed());
            println!(" number: {:?}", keys.number());
            println!();
        }

        // delay a bit. Shoot for polling every 1ms, giving about 8ms latency.
        cortex_m::asm::delay(power.clocks.sys_clk().to_Hz() / 2000);
    }
}
