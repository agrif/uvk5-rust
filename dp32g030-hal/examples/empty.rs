#![no_std]
#![no_main]

use dp32g030_hal as hal;
use panic_halt as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    // grab peripherals and initialize the clock
    let p = hal::pac::Peripherals::take().unwrap();
    let _power = hal::power::new(p.SYSCON, p.PMU, p.FLASH_CTRL)
        .sys_internal_24mhz()
        .freeze();

    loop {
        cortex_m::asm::wfi();
    }
}
