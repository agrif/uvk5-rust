use embedded_hal_02::blocking::delay as hal02blocking;
use embedded_hal_02::timer as hal02;

use crate::block;
use crate::time::{DurationExtU32, TimerDuration};

use super::{Count, Counter, Error};

impl<Timer, const HZ: u32, const DYN: bool> hal02::CountDown for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    type Time = TimerDuration<HZ>;

    #[inline(always)]
    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        Counter::start(self, count.into()).unwrap()
    }

    #[inline(always)]
    fn wait(&mut self) -> block::Result<(), void::Void> {
        Counter::wait(self).map_err(|e| match e {
            block::Error::WouldBlock => block::Error::WouldBlock,
            // not great, but panicing is the best we can do
            block::Error::Other(e) => panic!("{:?}", e),
        })
    }
}

impl<Timer, const HZ: u32, const DYN: bool> hal02::Cancel for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    type Error = Error;

    #[inline(always)]
    fn cancel(&mut self) -> Result<(), Self::Error> {
        Counter::cancel(self)
    }
}

impl<Timer, const HZ: u32, const DYN: bool> hal02::Periodic for Counter<Timer, HZ, DYN> where
    Timer: Count<HZ, DYN>
{
}

impl<Timer, const HZ: u32, const DYN: bool> hal02blocking::DelayUs<u32> for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    #[inline(always)]
    fn delay_us(&mut self, us: u32) {
        self.delay(us.micros()).unwrap()
    }
}

impl<Timer, const HZ: u32, const DYN: bool> hal02blocking::DelayUs<u16> for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    #[inline(always)]
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32)
    }
}

impl<Timer, const HZ: u32, const DYN: bool> hal02blocking::DelayUs<u8> for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    #[inline(always)]
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32)
    }
}

impl<Timer, const HZ: u32, const DYN: bool> hal02blocking::DelayMs<u32> for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    #[inline(always)]
    fn delay_ms(&mut self, ms: u32) {
        self.delay(ms.millis()).unwrap()
    }
}

impl<Timer, const HZ: u32, const DYN: bool> hal02blocking::DelayMs<u16> for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    #[inline(always)]
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32)
    }
}

impl<Timer, const HZ: u32, const DYN: bool> hal02blocking::DelayMs<u8> for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    #[inline(always)]
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32)
    }
}
