use muldiv::MulDiv;

use crate::pac;

use crate::block;
use crate::time::{TimerDuration, TimerInstant};

use super::{static_assert_timer_hz_not_zero, Base, Error, High, Low, System};

/// Timers that can be used in Counter.
#[allow(private_bounds)]
pub trait Count<const HZ: u32, const DYN: bool>: CountSealed<HZ, DYN> {}

/// Timers that can be used in Counter.
trait CountSealed<const HZ: u32, const DYN: bool> {
    /// What is the current count? At minimum this time has passed.
    fn now(&mut self) -> TimerInstant<HZ>;

    /// Start the count, waiting at least duration.
    fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), Error>;

    /// Cancel the count.
    fn cancel(&mut self) -> Result<(), Error>;

    /// Wait for the count to end.
    fn wait(&mut self) -> block::Result<(), Error>;
}

/// Helper for making sure either DYN is set, or T_HZ matches C_HZ
#[allow(path_statements)]
const fn static_assert_dyn_or_hz_same<const T_HZ: u32, const C_HZ: u32, const DYN: bool>() {
    // this should be a simple static_assert!
    // but rust does not like that
    struct Assert<const T_HZ: u32, const C_HZ: u32, const DYN: bool>;

    impl<const T_HZ: u32, const C_HZ: u32, const DYN: bool> Assert<T_HZ, C_HZ, DYN> {
        const DYN_OR_HZ_SAME: () = assert!(DYN || T_HZ == C_HZ);
    }

    #[allow(clippy::no_effect)]
    Assert::<T_HZ, C_HZ, DYN>::DYN_OR_HZ_SAME; // This error means a timer is not DYN but has un-matching HZ
}

macro_rules! impl_count {
    ($highlow:ident, $highbool:expr) => {
        impl<Timer, const T_HZ: u32, const C_HZ: u32, const DYN: bool> Count<C_HZ, DYN>
            for $highlow<Timer, T_HZ>
        where
            Timer: Base,
        {
        }

        impl<Timer, const T_HZ: u32, const C_HZ: u32, const DYN: bool> CountSealed<C_HZ, DYN>
            for $highlow<Timer, T_HZ>
        where
            Timer: Base,
        {
            #[inline(always)]
            fn now(&mut self) -> TimerInstant<C_HZ> {
                static_assert_dyn_or_hz_same::<T_HZ, C_HZ, DYN>();

                let clocks = self.timer.get_count($highbool) as u32;
                if DYN {
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
                static_assert_dyn_or_hz_same::<T_HZ, C_HZ, DYN>();

                let clocks = if DYN {
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
                    .checked_sub(1)
                    .ok_or(Error::OutOfRange)?
                    .try_into()
                    .map_err(|_| Error::OutOfRange)?;

                // unsafe: we are the owners of this half of the timer
                unsafe {
                    self.timer.set_enabled($highbool, false);
                    self.timer.clear_flag($highbool);
                    self.timer.set_load($highbool, clocks);
                    self.timer.set_enabled($highbool, true);
                }
                Ok(())
            }

            #[inline(always)]
            fn cancel(&mut self) -> Result<(), Error> {
                if self.timer.get_enabled($highbool) {
                    // unsafe: we are the owners of this half of the timer
                    unsafe {
                        self.timer.set_enabled($highbool, false);
                    }
                    Ok(())
                } else {
                    Err(Error::NotStarted)
                }
            }

            #[inline(always)]
            fn wait(&mut self) -> block::Result<(), Error> {
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
                    Err(block::Error::Other(Error::NotStarted))
                }
            }
        }
    };
}

impl_count!(Low, false);
impl_count!(High, true);

impl<const C_HZ: u32> Count<C_HZ, true> for System {}

impl<const C_HZ: u32> CountSealed<C_HZ, true> for System {
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
            .checked_sub(1)
            .ok_or(Error::OutOfRange)?;

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

/// A counter that can count up a duration.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Counter<Timer, const HZ: u32, const DYN: bool = false> {
    timer: Timer,
}

/// A counter with nanosecond precision.
pub type CounterNs<Timer> = Counter<Timer, 1_000_000_000, true>;

/// A counter with microsecond precision.
pub type CounterUs<Timer> = Counter<Timer, 1_000_000, true>;

/// A counter with millisecond precision.
pub type CounterMs<Timer> = Counter<Timer, 1_000, true>;

impl<Timer, const HZ: u32, const DYN: bool> Counter<Timer, HZ, DYN>
where
    Timer: Count<HZ, DYN>,
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

    /// Start the count.
    #[inline(always)]
    pub fn start(&mut self, duration: TimerDuration<HZ>) -> Result<(), Error> {
        self.timer.start(duration)
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
}
