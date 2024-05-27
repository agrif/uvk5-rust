//! Interfaces for hardware timers.

use crate::pac;

use crate::power::Clocks;
use crate::time::Hertz;

mod config;
pub use config::*;

mod fugit;
mod hal02;
mod hal1;

mod peripherals;
pub use peripherals::*;

mod timing;
pub use timing::*;

/// A very simple timer error.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Requested duration or frequency is out of range for this timer.
    OutOfRange,
    /// Timer has not been started.
    NotStarted,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Timer Error {:?}", self)
    }
}

/// Helper for making sure HZ is nonzero in constructors that need it.
#[allow(path_statements)]
const fn static_assert_timer_hz_not_zero<const HZ: u32>() {
    // this should be a simple static_assert!
    // but rust does not like that
    struct Assert<const HZ: u32>;

    impl<const HZ: u32> Assert<HZ> {
        const HZ_NOT_ZERO: () = assert!(HZ > 0);
    }

    #[allow(clippy::no_effect)]
    Assert::<HZ>::HZ_NOT_ZERO; // This error means a timer has HZ = 0
}

/// The low half of a timer. (type marker)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Low;

/// The high half of a timer. (type marker)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct High;

/// A marker trait for the [Low] / [High] marker types.
#[allow(private_bounds)]
pub trait TimerHalf: TimerHalfSealed {
    const IS_HIGH: bool;
}

trait TimerHalfSealed {}

impl TimerHalf for Low {
    const IS_HIGH: bool = false;
}

impl TimerHalfSealed for Low {}

impl TimerHalf for High {
    const IS_HIGH: bool = true;
}

impl TimerHalfSealed for High {}

/// One half of a configured timer.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Timer<T, LowHigh, const HZ: u32> {
    timer: T,
    _half: LowHigh,
    input_clk: Hertz,
}

/// The system timer.
pub struct System {
    timer: pac::SYST,
    input_clk: Hertz,
}

impl core::fmt::Debug for System {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("System")
            .field("timer", &"SYST")
            .field("input_clk", &self.input_clk)
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for System {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "System {{ timer: SYST, input_clk: {} }}", self.input_clk)
    }
}

macro_rules! timing_methods {
    () => {
        /// Use this timer in [TimingMode], with specific forced precision.
        ///
        /// This discards any statically-known frequency of the base
        /// timer, and instead calculates ticks from the dynamic
        /// `sys_clk` value. This *might* be more accurate, at the cost
        /// of runtime and code size.
        pub fn timing_hz<const C_HZ: u32>(self) -> TimingMode<Self, C_HZ, true> {
            TimingMode::new(self)
        }

        /// Use this timer in [TimingMode] with forced nanosecond precision.
        pub fn timing_ns(self) -> TimingModeNs<Self> {
            self.timing_hz()
        }

        /// Use this timer in [TimingMode] with forced microsecond precision.
        pub fn timing_us(self) -> TimingModeUs<Self> {
            self.timing_hz()
        }

        /// Use this timer in [TimingMode] with forced millisecond precision.
        pub fn timing_ms(self) -> TimingModeMs<Self> {
            self.timing_hz()
        }
    };

    (native) => {
        /// Use this timer in [TimingMode] with native precision.
        pub fn timing(self) -> TimingMode<Self, HZ> {
            TimingMode::new(self)
        }

        timing_methods!();
    };
}

impl<T, LowHigh, const HZ: u32> Timer<T, LowHigh, HZ>
where
    T: BaseInstance,
    LowHigh: TimerHalf,
{
    timing_methods!(native);
}

/// Create the system timer from the [pac::SYST] register.
pub fn new_system(syst: pac::SYST, clocks: &Clocks) -> System {
    System::new(syst, clocks)
}

impl System {
    /// Create the system timer from the [pac::SYST] register.
    pub fn new(syst: pac::SYST, clocks: &Clocks) -> Self {
        Self {
            timer: syst,
            input_clk: clocks.sys_clk(),
        }
    }

    /// Recover the [pac::SYST] register from this timer.
    pub fn free(self) -> pac::SYST {
        // safety: self owns SYST, and we're dropping self here
        unsafe { pac::CorePeripherals::steal().SYST }
    }

    timing_methods!();
}
