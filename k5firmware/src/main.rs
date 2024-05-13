#![no_std]
#![no_main]

use dp32g030_hal as hal;
use panic_halt as _;

use hal::prelude::*;
use hal::time::Hertz;

hal::version!(env!("CARGO_PKG_VERSION"));

struct UartFmt<UART>(UART);

impl<UART> core::fmt::Write for UartFmt<UART>
where
    UART: core::ops::Deref<Target = hal::pac::uart0::RegisterBlock>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.as_bytes() {
            while self.0.if_().read().txfifo_full().is_full() {}
            self.0.tdr().write(|w| w.data().set(*b));
        }
        Ok(())
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let _cp = hal::pac::CorePeripherals::take().unwrap();
    let p = hal::pac::Peripherals::take().unwrap();
    let mut power = hal::power::new(p.SYSCON, p.PMU);

    let clocks = power.clocks.sys_internal_48mhz().freeze();

    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_a = ports.port_a.enable(power.gates.gpio_a);
    let pins_c = ports.port_c.enable(power.gates.gpio_c);

    // flashlight is GPIO C3
    let mut light = pins_c.c3.erase().into_push_pull_output();

    // ptt button is GPIO C5
    let ptt = pins_c.c5.erase().into_pull_up_input();

    // uart1 tx is A7, uart1 rx is A8
    const ALT_TX: u8 = hal::pac::portcon::porta_sel0::PORTA7_A::Uart1Tx as u8;
    const ALT_RX: u8 = hal::pac::portcon::porta_sel1::PORTA8_A::Uart1Rx as u8;
    let tx = pins_a.a7.into_push_pull_output().into_alternate::<ALT_TX>();
    let rx = pins_a.a8.into_floating_input().into_alternate::<ALT_RX>();

    // power on the uart
    power.gates.uart1.enable();

    // disable uart to configure it
    p.UART1.ctrl().modify(|_, w| w.uarten().disabled());

    // set our baud to.. 39053 ?
    p.UART1
        .baud()
        .write(|w| w.baud().set((clocks.sys_clk() / Hertz::Hz(39053)) as u16));

    // enable rx and tx
    p.UART1.ctrl().write(|w| {
        w.rxen().enabled();
        w.txen().enabled()
    });

    // we don't use these pins yet.
    drop(tx);
    drop(rx);

    // reset a lot
    p.UART1.rxto().reset();
    p.UART1.fc().reset();
    p.UART1.ie().reset();

    // clear our fifos
    p.UART1.fifo().write(|w| {
        w.rf_clr().clear();
        w.tf_clr().clear()
    });

    // turn on the uart
    p.UART1.ctrl().modify(|_, w| w.uarten().enabled());

    // make a formatter
    let mut uart1 = UartFmt(p.UART1);

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
        writeln!(&mut uart1, "Hello, {}!", "UV-K5").unwrap();
        writeln!(&mut uart1, "PTT is {:?} {:?}", ptt, ptt.read()).unwrap();
        writeln!(&mut uart1, "Light is {:?} {:?}", light, light.get_state()).unwrap();
    }
}
