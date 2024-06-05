//! A [defmt] logger using [k5lib] messages.

// mostly copying defmt-semihosting here

use core::sync::atomic::{AtomicBool, Ordering};

#[defmt::global_logger]
struct Logger;

static TAKEN: AtomicBool = AtomicBool::new(false);
static INTERRUPTS_ACTIVE: AtomicBool = AtomicBool::new(false);
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

// don't go too crazy here
const BUFFER_LEN: usize = 0x100;
static mut BUFFER: [u8; BUFFER_LEN] = [0; BUFFER_LEN];
static mut BUFFER_NEXT: usize = 0;

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        let primask = cortex_m::register::primask::read();
        cortex_m::interrupt::disable();

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger acquired twice");
        }

        TAKEN.store(true, Ordering::Relaxed);

        INTERRUPTS_ACTIVE.store(primask.is_active(), Ordering::Relaxed);

        // safety: we disabled interrupts, and only we ever access this
        unsafe { ENCODER.start_frame(do_write) }
    }

    unsafe fn flush() {
        flush();
    }

    unsafe fn release() {
        // safety: we disabled interrupts in acquire, and only we ever
        // access this
        ENCODER.end_frame(do_write);
        flush();

        TAKEN.store(false, Ordering::Relaxed);
        if INTERRUPTS_ACTIVE.load(Ordering::Relaxed) {
            cortex_m::interrupt::enable();
        }
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: we disabled interrupts and only we ever access this
        ENCODER.write(bytes, do_write);
    }
}

fn do_write(mut bytes: &[u8]) {
    while !bytes.is_empty() {
        unsafe {
            // safety: interrupts are disabled and only we ever access these
            let amt = bytes.len().min(BUFFER_LEN - BUFFER_NEXT);
            BUFFER[BUFFER_NEXT..BUFFER_NEXT + amt].copy_from_slice(&bytes[..amt]);
            BUFFER_NEXT += amt;
            bytes = &bytes[amt..];

            if BUFFER_NEXT >= BUFFER_LEN {
                flush();
            }
        }
    }
}

unsafe fn flush() {
    // safety: interrupts are disabled and only we ever access these
    let data = &BUFFER[..BUFFER_NEXT];
    if data.is_empty() {
        return;
    }

    if let Some(mut tx) = crate::uart::try_tx() {
        let msg = k5lib::protocol::messages::custom::DebugOutput { defmt: true, data };
        let crc = k5lib::protocol::crc::CrcConstantIgnore(0xffff);
        let mut ser = k5lib::protocol::serialize::SerializerWrap::new(&mut *tx);
        let _ = k5lib::protocol::serialize(&crc, &mut ser, &msg);
        BUFFER_NEXT = 0;
    }
}
