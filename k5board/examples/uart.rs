#![no_std]
#![no_main]

use k5board::hal;
use panic_halt as _;

use k5board::prelude::*;

k5board::version!(concat!(env!("CARGO_PKG_VERSION"), "uart"));

#[cortex_m_rt::entry]
fn main() -> ! {
    // grab peripherals and initialize the clock
    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU, p.FLASH_CTRL)
        .sys_internal_24mhz()
        .freeze();

    // turn on GPIOA
    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_a = ports.port_a.enable(power.gates.gpio_a);

    // set up the uart and install it globally
    let uart_parts = k5board::uart::Parts {
        uart: p.UART1,
        gate: power.gates.uart1,
        tx: pins_a.a7.into_mode(),
        rx: pins_a.a8.into_mode(),
    };
    let uart = k5board::uart::new(38_400.Hz(), uart_parts).unwrap();
    k5board::uart::install(uart);

    let mut counter = 0;
    loop {
        println!("counter is: {:?}", counter);
        counter += 1;

        // delay a bit
        cortex_m::asm::delay(power.clocks.sys_clk().to_Hz() / 2);
    }
}
