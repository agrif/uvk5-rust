use crate::block;
use crate::time::{TimerDuration, TimerInstant};

use super::{Count, Counter, Error};

impl<Timer, const HZ: u32, const DYN: bool> fugit_timer::Timer<HZ> for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    type Error = Error;

    #[inline(always)]
    fn now(&mut self) -> TimerInstant<HZ> {
        Counter::now(self)
    }

    #[inline(always)]
    fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), Self::Error> {
        Counter::start(self, duration)
    }

    #[inline(always)]
    fn cancel(&mut self) -> Result<(), Self::Error> {
        Counter::cancel(self)
    }

    #[inline(always)]
    fn wait(&mut self) -> block::Result<(), Self::Error> {
        Counter::wait(self)
    }
}

impl<Timer, const HZ: u32, const DYN: bool> fugit_timer::Delay<HZ> for Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
{
    type Error = Error;

    #[inline(always)]
    fn delay(&mut self, duration: TimerDuration<HZ>) -> Result<(), Self::Error> {
        Counter::delay(self, duration)
    }
}
