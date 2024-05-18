use muldiv::MulDiv;

use crate::pac;

use crate::block;
use crate::time::{TimerDuration, TimerInstant, TimerRate};

use super::{static_assert_timer_hz_not_zero, BaseInstance, Error, System, Timer, TimerHalf};

/// Timers that can be used in [TimingMode].
#[allow(private_bounds)]
pub trait TimingInstance<const HZ: u32, const FORCED: bool>:
    TimingInstanceSealed<HZ, FORCED>
{
}

/// Timers that can be used in [TimingMode].
trait TimingInstanceSealed<const HZ: u32, const FORCED: bool> {
    /// What is the current count? At minimum this time has passed.
    fn now(&mut self) -> TimerInstant<HZ>;

    /// Start the count, waiting at least duration.
    fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), Error>;

    /// Cancel the count.
    fn cancel(&mut self) -> Result<(), Error>;

    /// Wait for the count to end.
    fn wait(&mut self) -> block::Result<(), Error>;
}

/// Helper for making sure either FORCED is set, or T_HZ matches C_HZ
#[allow(path_statements)]
const fn static_assert_forced_or_hz_same<const T_HZ: u32, const C_HZ: u32, const FORCED: bool>() {
    // this should be a simple static_assert!
    // but rust does not like that
    struct Assert<const T_HZ: u32, const C_HZ: u32, const FORCED: bool>;

    impl<const T_HZ: u32, const C_HZ: u32, const FORCED: bool> Assert<T_HZ, C_HZ, FORCED> {
        const FORCED_OR_HZ_SAME: () = assert!(FORCED || T_HZ == C_HZ);
    }

    #[allow(clippy::no_effect)]
    Assert::<T_HZ, C_HZ, FORCED>::FORCED_OR_HZ_SAME; // This error means a timer is not FORCED but has un-matching HZ
}

impl<T, HighLow, const T_HZ: u32, const C_HZ: u32, const FORCED: bool> TimingInstance<C_HZ, FORCED>
    for Timer<T, HighLow, T_HZ>
where
    T: BaseInstance,
    HighLow: TimerHalf,
{
}

impl<T, HighLow, const T_HZ: u32, const C_HZ: u32, const FORCED: bool>
    TimingInstanceSealed<C_HZ, FORCED> for Timer<T, HighLow, T_HZ>
where
    T: BaseInstance,
    HighLow: TimerHalf,
{
    #[inline(always)]
    fn now(&mut self) -> TimerInstant<C_HZ> {
        static_assert_forced_or_hz_same::<T_HZ, C_HZ, FORCED>();

        let clocks = self.timer.get_count(HighLow::IS_HIGH) as u32;
        if FORCED {
            // use input_clk
            let ticks = clocks
                .mul_div_floor(C_HZ, self.input_clk.to_Hz())
                // 0 is a poor choice on failure, but it has to do
                // panicing here would be... odd
                .unwrap_or(0);
            TimerInstant::from_ticks(ticks)
        } else {
            // T_HZ == C_HZ
            TimerInstant::from_ticks(clocks)
        }
    }

    #[inline(always)]
    fn start(&mut self, duration: TimerDuration<C_HZ>) -> Result<(), Error> {
        static_assert_forced_or_hz_same::<T_HZ, C_HZ, FORCED>();

        let clocks = if FORCED {
            // use input_clk
            duration
                .ticks()
                .mul_div_ceil(self.input_clk.to_Hz(), C_HZ)
                .ok_or(Error::OutOfRange)?
        } else {
            // T_HZ == C_HZ
            duration.ticks()
        };

        let clocks = clocks
            .saturating_sub(1)
            .try_into()
            .map_err(|_| Error::OutOfRange)?;

        // unsafe: we are the owners of this half of the timer
        unsafe {
            self.timer.set_enabled(HighLow::IS_HIGH, false);
            self.timer.clear_flag(HighLow::IS_HIGH);
            self.timer.set_load(HighLow::IS_HIGH, clocks);
            self.timer.set_enabled(HighLow::IS_HIGH, true);
        }
        Ok(())
    }

    #[inline(always)]
    fn cancel(&mut self) -> Result<(), Error> {
        if self.timer.get_enabled(HighLow::IS_HIGH) {
            // unsafe: we are the owners of this half of the timer
            unsafe {
                self.timer.set_enabled(HighLow::IS_HIGH, false);
            }
            Ok(())
        } else {
            Err(Error::NotStarted)
        }
    }

    #[inline(always)]
    fn wait(&mut self) -> block::Result<(), Error> {
        if self.timer.get_enabled(HighLow::IS_HIGH) {
            if self.timer.get_flag(HighLow::IS_HIGH) {
                // safety: we are the owners of this half of the timer
                unsafe {
                    self.timer.clear_flag(HighLow::IS_HIGH);
                }
                Ok(())
            } else {
                Err(block::Error::WouldBlock)
            }
        } else {
            Err(block::Error::Other(Error::NotStarted))
        }
    }
}

impl<const C_HZ: u32> TimingInstance<C_HZ, true> for System {}

impl<const C_HZ: u32> TimingInstanceSealed<C_HZ, true> for System {
    #[inline(always)]
    fn now(&mut self) -> TimerInstant<C_HZ> {
        // get_current should always be <= get_reload
        let clocks = pac::SYST::get_reload() - pac::SYST::get_current();
        let ticks = clocks
            .mul_div_floor(C_HZ, self.input_clk.to_Hz())
            // 0 is a poor choice on failure, but it has to do
            // panicing here would be... odd
            .unwrap_or(0);
        TimerInstant::from_ticks(ticks)
    }

    #[inline(always)]
    fn start(&mut self, duration: TimerDuration<C_HZ>) -> Result<(), Error> {
        let clocks = duration
            .ticks()
            .mul_div_ceil(self.input_clk.to_Hz(), C_HZ)
            .ok_or(Error::OutOfRange)?
            .saturating_sub(1);

        if clocks > 0x00ffffff {
            return Err(Error::OutOfRange);
        }

        self.timer.disable_counter();
        self.timer.set_reload(clocks);
        self.timer.clear_current();
        self.timer.has_wrapped();
        self.timer.enable_counter();

        Ok(())
    }

    #[inline(always)]
    fn cancel(&mut self) -> Result<(), Error> {
        if self.timer.is_counter_enabled() {
            self.timer.disable_counter();
            Ok(())
        } else {
            Err(Error::NotStarted)
        }
    }

    #[inline(always)]
    fn wait(&mut self) -> block::Result<(), Error> {
        let has_wrapped = self.timer.has_wrapped();
        if self.timer.is_counter_enabled() {
            if has_wrapped {
                Ok(())
            } else {
                Err(block::Error::WouldBlock)
            }
        } else {
            Err(block::Error::Other(Error::NotStarted))
        }
    }
}

/// A timer in TimingMode, that can wait out durations.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimingMode<Timer, const HZ: u32, const FORCED: bool = false> {
    timer: Timer,
}

/// A forced [TimingMode] with nanosecond precision.
pub type TimingModeNs<Timer> = TimingMode<Timer, 1_000_000_000, true>;

/// A forced [TimingMode] with microsecond precision.
pub type TimingModeUs<Timer> = TimingMode<Timer, 1_000_000, true>;

/// A forced [TimingMode] with millisecond precision.
pub type TimingModeMs<Timer> = TimingMode<Timer, 1_000, true>;

impl<Timer, const HZ: u32, const FORCED: bool> TimingMode<Timer, HZ, FORCED>
where
    Timer: TimingInstance<HZ, FORCED>,
{
    /// Create a new Counter.
    #[inline(always)]
    pub fn new(timer: Timer) -> Self {
        static_assert_timer_hz_not_zero::<HZ>();
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

    /// Start the count, lasting for the given duration.
    #[inline(always)]
    pub fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), Error> {
        self.timer.start(duration)
    }

    /// Start the count, rolling over at the given rate.
    #[inline(always)]
    pub fn start_rate(&mut self, rate: TimerRate<HZ>) -> Result<(), Error> {
        self.start(rate.into_duration())
    }

    /// Start the count, rolling over at the native timer frequency.
    #[inline(always)]
    pub fn start_native(&mut self) -> Result<(), Error> {
        self.start_rate(TimerRate::Hz(HZ))
    }

    /// Cancel the count.
    #[inline(always)]
    pub fn cancel(&mut self) -> Result<(), Error> {
        self.timer.cancel()
    }

    /// Wait for the count to end.
    #[inline(always)]
    pub fn wait(&mut self) -> block::Result<(), Error> {
        self.timer.wait()
    }

    /// Blocking wait for a duration.
    #[inline(always)]
    pub fn delay(&mut self, duration: TimerDuration<HZ>) -> Result<(), Error> {
        self.timer.start(duration)?;
        block::block!(self.timer.wait())
    }
}
