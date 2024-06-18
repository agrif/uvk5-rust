#![no_std]
#![no_main]

use critical_section::CriticalSection;
use dp32g030::flash_ctrl::cfg::MODE_A;
use panic_halt as _;

dp32g030_hal_flash::header! {
    init,
    read_nvr,
    erase,
    program_word,
    program,
    read_nvr_apb,
}

// safety: see Code in lib.rs
pub unsafe fn init(cs: CriticalSection, clock_mhz: u8) {
    let mut flash = Flash::get(cs);

    flash.leave_low_power_and_wait_init();
    flash.set_mode(MODE_A::Normal);
    flash.set_read_md(clock_mhz);
    flash.set_erasetime(clock_mhz);
    flash.set_progtime(clock_mhz);
    flash.lock();
}

// safety: see Code in lib.rs
pub unsafe fn read_nvr(cs: CriticalSection, src: u16, dest: &mut [u8]) {
    let mut flash = Flash::get(cs);

    flash.with_nvr(true, |_flash| unsafe {
        let src = core::slice::from_raw_parts(src as *const u8, dest.len());
        dest.copy_from_slice(src);
    })
}

// safety: see Code in lib.rs
pub unsafe fn erase(cs: CriticalSection, sector: *mut u32) {
    let mut flash = Flash::get(cs);

    flash.execute(
        |flash| {
            flash.set_mode(MODE_A::Erase);
            flash.set_address(sector);
        },
        |_| {},
        |_| {},
    );
}

// safety: see Code in lib.rs
pub unsafe fn program_word(cs: CriticalSection, word: u32, dest: *mut u32) {
    let mut flash = Flash::get(cs);

    flash.execute(
        |flash| {
            flash.set_mode(MODE_A::Program);
            flash.set_address(dest);
            flash.set_wdata(word);
        },
        |_| {},
        |_| {},
    )
}

// safety: see Code in lib.rs
pub unsafe fn program(cs: CriticalSection, src: &[u32], dest: *mut u32) {
    // empty succeeds automatically
    if src.is_empty() {
        return;
    }

    let mut flash = Flash::get(cs);

    flash.execute(
        |flash| {
            flash.set_mode(MODE_A::Program);
            flash.set_address(dest);
            flash.set_wdata(src[0]);
        },
        |flash| {
            for word in &src[1..] {
                flash.wait_prog_buf_empty();
                flash.set_wdata(*word);
            }
        },
        |_| {},
    );
}

// safety: see Code in lib.rs
pub unsafe fn read_nvr_apb(cs: CriticalSection, src: u16) -> u32 {
    let mut flash = Flash::get(cs);

    flash.with_nvr(true, |flash| {
        flash.execute(
            |flash| {
                flash.set_mode(MODE_A::ReadApb);
                flash.set_address(src as *mut u32);
            },
            |_| {},
            |flash| flash.rdata(),
        )
    })
}

pub struct Flash {
    ctrl: dp32g030::FLASH_CTRL,
}

impl Flash {
    pub fn get(_cs: critical_section::CriticalSection) -> Self {
        // safety: we have a critical section, only we can be talking to flash
        Flash {
            ctrl: unsafe { dp32g030::FLASH_CTRL::steal() },
        }
    }

    pub fn leave_low_power_and_wait_init(&mut self) {
        self.ctrl.cfg().modify(|_, w| w.deep_pd().normal());
        while self.ctrl.status().read().init_busy().is_busy() {
            core::hint::spin_loop();
        }
    }

    pub fn wait_ready(&self) {
        while self.ctrl.status().read().busy().is_busy() {
            core::hint::spin_loop();
        }
    }

    pub fn wait_prog_buf_empty(&self) {
        while !self.ctrl.status().read().prog_buf_empty().is_empty() {
            core::hint::spin_loop();
        }
    }

    pub fn set_mode(&mut self, mode: MODE_A) {
        self.ctrl.cfg().modify(|_, w| w.mode().variant(mode));
    }

    pub fn set_address(&mut self, address: *mut u32) {
        self.ctrl
            .addr()
            .write(|w| w.addr().set((address as u16) >> 2))
    }

    pub fn with_nvr<R>(&mut self, nvr: bool, f: impl FnOnce(&mut Self) -> R) -> R {
        if nvr {
            self.ctrl.cfg().modify(|_, w| w.nvr_sel().nvr());
        } else {
            self.ctrl.cfg().modify(|_, w| w.nvr_sel().main());
        }
        let r = f(self);
        self.ctrl.cfg().modify(|_, w| w.nvr_sel().main());
        r
    }

    pub fn set_read_md(&mut self, clock_mhz: u8) {
        if clock_mhz >= 56 {
            self.ctrl.cfg().modify(|_, w| w.read_md().wait2());
        } else {
            self.ctrl.cfg().modify(|_, w| w.read_md().wait1());
        }
    }

    pub fn set_erasetime(&mut self, clock_mhz: u8) {
        self.ctrl.erasetime().write(|w| {
            // terase = 3.6ms = 3600ns
            w.terase()
                .set(3600 * clock_mhz as u32)
                // trcv = 52ns
                .trcv()
                .set(52 * clock_mhz as u16)
        })
    }

    pub fn set_progtime(&mut self, clock_mhz: u8) {
        self.ctrl.progtime().write(|w| {
            // tprog = 18ns
            w.tprog()
                .set(18 * clock_mhz as u16)
                // tpgs = 22ns
                .tpgs()
                .set(22 * clock_mhz as u16)
        })
    }

    pub fn lock(&mut self) {
        self.ctrl.lock().write(|w| w.lock().locked())
    }

    pub fn unlock(&mut self) {
        self.ctrl.unlock().write(|w| w.unlock().unlocked())
    }

    pub fn start(&mut self) {
        self.unlock();
        self.ctrl.start().write(|w| w.start().started())
    }

    pub fn set_wdata(&mut self, word: u32) {
        self.ctrl.wdata().write(|w| w.word().set(word))
    }

    pub fn rdata(&self) -> u32 {
        self.ctrl.rdata().read().word().bits()
    }

    pub fn execute<R>(
        &mut self,
        start: impl FnOnce(&mut Self),
        between: impl FnOnce(&mut Self),
        end: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.wait_ready();
        start(self);

        self.start();
        between(self);

        self.wait_ready();
        let r = end(self);

        self.set_mode(MODE_A::Normal);
        self.lock();

        r
    }
}
