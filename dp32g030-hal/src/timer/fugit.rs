use crate::block;
use crate::time::{TimerDuration, TimerInstant};

use super::{Error, TimingInstance, TimingMode};

impl<Timer, const HZ: u32, const FORCED: bool> fugit_timer::Timer<HZ>
    for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    type Error = Error;

    #[inline(always)]
    fn now(&mut self) -> TimerInstant<HZ> {
        TimingMode::now(self)
    }

    #[inline(always)]
    fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), Self::Error> {
        TimingMode::start(self, duration)
    }

    #[inline(always)]
    fn cancel(&mut self) -> Result<(), Self::Error> {
        TimingMode::cancel(self)
    }

    #[inline(always)]
    fn wait(&mut self) -> block::Result<(), Self::Error> {
        TimingMode::wait(self)
    }
}

impl<Timer, const HZ: u32, const FORCED: bool> fugit_timer::Delay<HZ>
    for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    type Error = Error;

    #[inline(always)]
    fn delay(&mut self, duration: TimerDuration<HZ>) -> Result<(), Self::Error> {
        TimingMode::delay(self, duration)
    }
}
