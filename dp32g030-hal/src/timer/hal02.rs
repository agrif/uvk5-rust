use embedded_hal_02::blocking::delay as hal02blocking;
use embedded_hal_02::timer as hal02;

use crate::block;
use crate::time::{DurationExtU32, TimerDuration};

use super::{Error, TimingInstance, TimingMode};

impl<Timer, const HZ: u32, const FORCED: bool> hal02::CountDown for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    type Time = TimerDuration<HZ>;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        TimingMode::start(self, count.into()).unwrap()
    }

    fn wait(&mut self) -> block::Result<(), void::Void> {
        TimingMode::wait(self).map_err(|e| match e {
            block::Error::WouldBlock => block::Error::WouldBlock,
            // not great, but panicing is the best we can do
            block::Error::Other(e) => panic!("{:?}", e),
        })
    }
}

impl<Timer, const HZ: u32, const FORCED: bool> hal02::Cancel for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    type Error = Error;

    fn cancel(&mut self) -> Result<(), Self::Error> {
        TimingMode::cancel(self)
    }
}

impl<Timer, const HZ: u32, const FORCED: bool> hal02::Periodic for TimingMode<Timer, HZ, FORCED> where
    Timer: TimingInstance<HZ, FORCED>
{
}

impl<Timer, const HZ: u32, const FORCED: bool> hal02blocking::DelayUs<u32>
    for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    fn delay_us(&mut self, us: u32) {
        self.delay(us.micros()).unwrap()
    }
}

impl<Timer, const HZ: u32, const FORCED: bool> hal02blocking::DelayUs<u16>
    for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32)
    }
}

impl<Timer, const HZ: u32, const FORCED: bool> hal02blocking::DelayUs<u8>
    for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32)
    }
}

impl<Timer, const HZ: u32, const FORCED: bool> hal02blocking::DelayMs<u32>
    for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    fn delay_ms(&mut self, ms: u32) {
        self.delay(ms.millis()).unwrap()
    }
}

impl<Timer, const HZ: u32, const FORCED: bool> hal02blocking::DelayMs<u16>
    for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32)
    }
}

impl<Timer, const HZ: u32, const FORCED: bool> hal02blocking::DelayMs<u8>
    for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32)
    }
}
