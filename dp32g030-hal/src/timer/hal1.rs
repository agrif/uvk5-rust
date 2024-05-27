use embedded_hal_1::delay as hal1;

use crate::time::DurationExtU32;

use super::{TimingInstance, TimingMode};

impl<Timer, const HZ: u32, const FORCED: bool> hal1::DelayNs for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    fn delay_ns(&mut self, ns: u32) {
        self.delay(ns.nanos()).unwrap()
    }

    fn delay_us(&mut self, us: u32) {
        self.delay(us.micros()).unwrap()
    }

    fn delay_ms(&mut self, ms: u32) {
        self.delay(ms.millis()).unwrap()
    }
}
