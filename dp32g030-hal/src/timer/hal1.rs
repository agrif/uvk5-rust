use embedded_hal_1::delay as hal1;

use crate::time::DurationExtU32;

use super::{Count, Counter};

impl<Timer, const HZ: u32, const DYN: bool> hal1::DelayNs for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    #[inline(always)]
    fn delay_ns(&mut self, ns: u32) {
        // we have nothing smaller to try if this don't work
        self.delay(ns.nanos()).unwrap()
    }

    #[inline]
    fn delay_us(&mut self, mut us: u32) {
        const NANOS_PER_MICRO: u32 = 1_000;
        const MAX_MICROS: u32 = u32::MAX / NANOS_PER_MICRO;

        // try directly first
        if self.delay(us.micros()).is_err() {
            // assume it's too long, try to break it up
            while us > MAX_MICROS {
                us -= MAX_MICROS;
                self.delay_ns(MAX_MICROS * NANOS_PER_MICRO);
            }

            self.delay_ns(us * NANOS_PER_MICRO);
        }
    }

    #[inline]
    fn delay_ms(&mut self, mut ms: u32) {
        const NANOS_PER_MILLI: u32 = 1_000_000;
        const MAX_MILLIS: u32 = u32::MAX / NANOS_PER_MILLI;

        // try directly first
        if self.delay(ms.millis()).is_err() {
            // assume it's too long, try to break it up
            while ms > MAX_MILLIS {
                ms -= MAX_MILLIS;
                self.delay_ns(MAX_MILLIS * NANOS_PER_MILLI);
            }

            self.delay_ns(ms * NANOS_PER_MILLI);
        }
    }
}
