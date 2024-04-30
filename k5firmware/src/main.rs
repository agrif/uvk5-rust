#![no_std]
#![no_main]

use core::cell::Cell;

use cortex_m_rt::exception;
use critical_section::Mutex;
use panic_halt as _;

#[no_mangle]
pub static VERSION: &core::ffi::CStr = c"*0.0test";

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

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut cp = dp32g030::CorePeripherals::take().unwrap();
    let p = dp32g030::Peripherals::take().unwrap();

    // tick every 10ms. There are 100x 10ms in 1s, and our clock is 48MHz.
    cp.SYST.set_reload(48_000_000 / 100);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();

    // flashlight is GPIO C3
    // ptt button is GPIO C5

    // turn on GPIOC
    p.SYSCON
        .dev_clk_gate()
        .modify(|_, w| w.gpioc_clk_gate().enabled());

    // set our pins to be GPIO
    p.PORTCON.portc_sel0().modify(|_, w| {
        w.portc3().gpioc3();
        w.portc5().gpioc5()
    });

    // turn on input for ptt
    p.PORTCON.portc_ie().modify(|_, w| {
        w.portc3_ie().disabled();
        w.portc5_ie().enabled()
    });

    // turn on pull-up for ptt
    p.PORTCON.portc_pu().modify(|_, w| {
        w.portc3_pu().disabled();
        w.portc5_pu().enabled()
    });

    // disable all pull-downs
    p.PORTCON.portc_pd().modify(|_, w| {
        w.portc3_pd().disabled();
        w.portc5_pd().disabled()
    });

    // turn on open drain for ptt (?)
    p.PORTCON.portc_od().modify(|_, w| {
        w.portc3_od().disabled();
        w.portc5_od().enabled()
    });

    // flashlight is output, ptt is input
    p.GPIOC.dir().modify(|_, w| {
        w.dir3().output();
        w.dir5().input()
    });

    // turn on flashlight
    p.GPIOC.data().modify(|_, w| w.data3().high());

    let mut state = false;
    loop {
        // ptt pressed means ptt low
        // ptt pressed means toggle light
        let ptt = p.GPIOC.data().read().data5().is_low();
        if ptt {
            state = !state;
        }

        p.GPIOC.data().modify(|_, w| {
            if state {
                w.data3().low()
            } else {
                w.data3().high()
            }
        });

        delay_ms(500);
    }
}
