use crate::block;
use crate::time::{TimerDuration, TimerInstant};

use super::{Error, TimingInstance, TimingMode};

impl<Timer, const HZ: u32, const FORCED: bool> fugit_timer::Timer<HZ>
    for TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    type Error = Error;

    fn now(&mut self) -> TimerInstant<HZ> {
        TimingMode::now(self)
    }

    fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), Self::Error> {
        TimingMode::start(self, duration)
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        TimingMode::cancel(self)
    }

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

    fn delay(&mut self, duration: TimerDuration<HZ>) -> Result<(), Self::Error> {
        TimingMode::delay(self, duration)
    }
}
