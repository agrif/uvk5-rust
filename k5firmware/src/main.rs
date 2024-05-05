#![no_std]
#![no_main]

use core::cell::Cell;

use cortex_m_rt::exception;
use critical_section::Mutex;
use dp32g030_hal as hal;
use panic_halt as _;

use hal::pac;

hal::version!(env!("CARGO_PKG_VERSION"));

pub static TICKMS: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

#[cortex_m_rt::exception]
fn SysTick() {
    critical_section::with(|cs| {
        let tick = TICKMS.borrow(cs);
        // each tick is 10ms
        tick.set(tick.get() + 10);
    });
}

fn delay_ms(ms: usize) {
    let end = critical_section::with(|cs| TICKMS.borrow(cs).get() + ms as u64);
    loop {
        let now = critical_section::with(|cs| TICKMS.borrow(cs).get());
        if now >= end {
            break;
        } else {
            cortex_m::asm::wfi();
        }
    }
}

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
    let mut cp = hal::pac::CorePeripherals::take().unwrap();
    let p = hal::pac::Peripherals::take().unwrap();
    let mut power = hal::power::new(p.SYSCON, p.PMU);

    let clocks = power.clocks.sys_internal_48mhz().freeze();

    // turn on GPIOA, GPIOC and UART1
    // important! must be turned on before configured.
    power.dev_gate.enable_gpioa().enable_gpioc().enable_uart1();

    // tick every 10ms. There are 100x 10ms in 1s.
    // to make the time wrap every N ticks, set reload to N - 1.
    cp.SYST.set_reload((clocks.sys_clk() / 100) - 1);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();

    let pins = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);

    // flashlight is GPIO C3
    let mut light = pins.port_c.c3.into_push_pull_output();

    // ptt button is GPIO C5
    let ptt = pins.port_c.c5.into_pull_up_input();

    // uart1 tx is A7, uart1 rx is A8
    const ALT_TX: u8 = pac::portcon::porta_sel0::PORTA7_A::Uart1Tx as u8;
    const ALT_RX: u8 = pac::portcon::porta_sel1::PORTA8_A::Uart1Rx as u8;
    let tx = pins
        .port_a
        .a7
        .into_push_pull_output()
        .into_alternate::<ALT_TX>();
    let rx = pins
        .port_a
        .a8
        .into_floating_input()
        .into_alternate::<ALT_RX>();

    // disable uart to configure it
    p.UART1.ctrl().modify(|_, w| w.uarten().disabled());

    // set our baud to.. 39053 ?
    p.UART1
        .baud()
        .write(|w| w.baud().set((clocks.sys_clk() / 39053) as u16));

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

    // turn on flashlight
    light.set_high();

    loop {
        // ptt pressed means ptt low
        // ptt pressed means toggle light
        if ptt.is_low() {
            light.toggle();
        }

        delay_ms(500);

        use core::fmt::Write;
        writeln!(&mut uart1, "Hello, {}!", "UV-K5").unwrap();
        writeln!(&mut uart1, "PTT is {:?}", ptt.read()).unwrap();
        writeln!(&mut uart1, "Light is {:?}", light.get_state()).unwrap();
    }
}
