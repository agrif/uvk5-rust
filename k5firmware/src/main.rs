#![no_std]
#![no_main]

use panic_halt as _;

#[no_mangle]
pub static VERSION: &core::ffi::CStr = c"*0.0test";

pub static mut TESTRW: u8 = 1;

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = dp32g030::Peripherals::take().unwrap();

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

    while unsafe { TESTRW > 0 } {
        // ptt pressed means ptt low
        // ptt pressed means turn on light
        let ptt = p.GPIOC.data().read().data5().is_low();
        p.GPIOC.data().modify(|_, w| {
            if ptt {
                w.data3().low()
            } else {
                w.data3().high()
            }
        });
        cortex_m::asm::nop();

        unsafe {
            TESTRW += 1;
        }
    }

    loop {}
}
