#![no_std]
#![no_main]

use panic_halt as _;
use dp32g030 as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    cortex_m::asm::nop();

    loop {
        cortex_m::asm::nop();
    }
}
