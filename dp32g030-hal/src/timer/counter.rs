use crate::pac;

use crate::block;
use crate::time::{TimerDuration, TimerInstant};

use super::{Base, High, Low, System};

/// Timers that can be used in Counter.
#[allow(private_bounds)]
pub trait Count<const HZ: u32>: CountSealed<HZ> {}

/// Timers that can be used in Counter.
trait CountSealed<const HZ: u32> {
    /// What is the current count?
    fn now(&mut self) -> TimerInstant<HZ>;

    /// Start the count.
    fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), ()>;

    /// Cancel the count.
    fn cancel(&mut self) -> Result<(), ()>;

    /// Wait for the count to end.
    fn wait(&mut self) -> block::Result<(), ()>;
}

macro_rules! impl_count {
    ($highlow:ident, $highbool:expr) => {
        impl<Timer, const HZ: u32> Count<HZ> for $highlow<Timer> where Timer: Base {}

        impl<Timer, const HZ: u32> CountSealed<HZ> for $highlow<Timer>
        where
            Timer: Base,
        {
            #[inline(always)]
            fn now(&mut self) -> TimerInstant<HZ> {
                let clocks = self.timer.get_count($highbool);
                let ticks = clocks as u64 * HZ as u64 / self.input_clk.to_Hz() as u64;
                TimerInstant::from_ticks(ticks as u32)
            }

            #[inline(always)]
            fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), ()> {
                let clocks = duration.ticks() as u64 * self.input_clk.to_Hz() as u64 / HZ as u64;
                let clocks = clocks - 1;
                if clocks > 0xffff {
                    return Err(());
                }

                // unsafe: we are the owners of this half of the timer
                unsafe {
                    self.timer.set_enabled($highbool, false);
                    self.timer.clear_flag($highbool);
                    self.timer.set_load($highbool, clocks as u16);
                    self.timer.set_enabled($highbool, true);
                }
                Ok(())
            }

            #[inline(always)]
            fn cancel(&mut self) -> Result<(), ()> {
                if self.timer.get_enabled($highbool) {
                    // unsafe: we are the owners of this half of the timer
                    unsafe {
                        self.timer.set_enabled($highbool, false);
                    }
                    Ok(())
                } else {
                    Err(())
                }
            }

            #[inline(always)]
            fn wait(&mut self) -> block::Result<(), ()> {
                if self.timer.get_enabled($highbool) {
                    if self.timer.get_flag($highbool) {
                        // safety: we are the owners of this half of the timer
                        unsafe {
                            self.timer.clear_flag($highbool);
                        }
                        Ok(())
                    } else {
                        Err(block::Error::WouldBlock)
                    }
                } else {
                    Err(block::Error::Other(()))
                }
            }
        }
    };
}

impl_count!(Low, false);
impl_count!(High, true);

impl<const HZ: u32> Count<HZ> for System {}

impl<const HZ: u32> CountSealed<HZ> for System {
    #[inline(always)]
    fn now(&mut self) -> TimerInstant<HZ> {
        let clocks = pac::SYST::get_reload() - pac::SYST::get_current();
        let ticks = clocks as u64 * HZ as u64 / self.input_clk.to_Hz() as u64;
        TimerInstant::from_ticks(ticks as u32)
    }

    #[inline(always)]
    fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), ()> {
        let clocks = duration.ticks() as u64 * self.input_clk.to_Hz() as u64 / HZ as u64;
        let clocks = clocks - 1;
        if clocks > 0x00ffffff {
            return Err(());
        }

        self.timer.disable_counter();
        self.timer.set_reload(clocks as u32);
        self.timer.clear_current();
        self.timer.has_wrapped();
        self.timer.enable_counter();

        Ok(())
    }

    #[inline(always)]
    fn cancel(&mut self) -> Result<(), ()> {
        if self.timer.is_counter_enabled() {
            self.timer.disable_counter();
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline(always)]
    fn wait(&mut self) -> block::Result<(), ()> {
        let has_wrapped = self.timer.has_wrapped();
        if self.timer.is_counter_enabled() {
            if has_wrapped {
                Ok(())
            } else {
                Err(block::Error::WouldBlock)
            }
        } else {
            Err(block::Error::Other(()))
        }
    }
}

/// A counter that can count up a duration.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Counter<Timer, const HZ: u32> {
    timer: Timer,
}

/// A counter with nanosecond precision.
pub type CounterNs<Timer> = Counter<Timer, 1_000_000_000>;

/// A counter with microsecond precision.
pub type CounterUs<Timer> = Counter<Timer, 1_000_000>;

/// A counter with millisecond precision.
pub type CounterMs<Timer> = Counter<Timer, 1_000>;

impl<Timer, const HZ: u32> Counter<Timer, HZ>
where
    Timer: Count<HZ>,
{
    /// Create a new Counter.
    #[inline(always)]
    pub fn new(timer: Timer) -> Self {
        Self { timer }
    }

    /// Free the Counter and return the underlying timer.
    #[inline(always)]
    pub fn free(self) -> Timer {
        self.timer
    }

    /// What is the current count?
    #[inline(always)]
    pub fn now(&mut self) -> TimerInstant<HZ> {
        self.timer.now()
    }

    /// Start the count.
    #[inline(always)]
    pub fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), ()> {
        self.timer.start(duration)
    }

    /// Cancel the count.
    #[inline(always)]
    pub fn cancel(&mut self) -> Result<(), ()> {
        self.timer.cancel()
    }

    /// Wait for the count to end.
    #[inline(always)]
    pub fn wait(&mut self) -> block::Result<(), ()> {
        self.timer.wait()
    }
}
