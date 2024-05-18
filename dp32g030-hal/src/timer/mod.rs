//! Interfaces for hardware timers.

use crate::pac;

use crate::power::Clocks;
use crate::time::Hertz;

mod config;
pub use config::*;

mod counter;
pub use counter::*;

mod fugit;
mod hal02;
mod hal1;

mod peripherals;
pub use peripherals::*;

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
    #[allow(clippy::missing_inline_in_public_items)]
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
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("System")
            .field("timer", &"SYST")
            .field("input_clk", &self.input_clk)
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for System {
    #[allow(clippy::missing_inline_in_public_items)]
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "System {{ timer: SYST, input_clk: {} }}", self.input_clk)
    }
}

macro_rules! counter_methods {
    () => {
        /// Create a dynamic [Counter] from this timer, with specific precision.
        ///
        /// This discards any statically-known frequency of the base
        /// timer, and instead calculates ticks from the dynamic
        /// `sys_clk` value. This *might* be more accurate, at the cost
        /// of runtime and code size.
        #[inline(always)]
        pub fn counter_hz<const C_HZ: u32>(self) -> Counter<Self, C_HZ, true> {
            Counter::new(self)
        }

        /// Create a dynamic [Counter] with nanosecond precision
        /// with this timer.
        #[inline(always)]
        pub fn counter_ns(self) -> CounterNs<Self> {
            self.counter_hz()
        }

        /// Create a dynamic [Counter] with microsecond precision
        /// with this timer.
        #[inline(always)]
        pub fn counter_us(self) -> CounterUs<Self> {
            self.counter_hz()
        }

        /// Create a dynamic [Counter] with millisecond precision
        /// with this timer.
        #[inline(always)]
        pub fn counter_ms(self) -> CounterMs<Self> {
            self.counter_hz()
        }
    };

    (native) => {
        /// Create a Counter from this timer, using the native precision.
        #[inline(always)]
        pub fn counter(self) -> Counter<Self, HZ> {
            Counter::new(self)
        }

        counter_methods!();
    };
}

impl<T, LowHigh, const HZ: u32> Timer<T, LowHigh, HZ>
where
    T: BaseInstance,
    LowHigh: TimerHalf,
{
    counter_methods!(native);
}

/// Create the system timer from the [pac::SYST] register.
#[inline(always)]
pub fn new_system(syst: pac::SYST, clocks: &Clocks) -> System {
    System::new(syst, clocks)
}

impl System {
    /// Create the system timer from the [pac::SYST] register.
    #[inline(always)]
    pub fn new(syst: pac::SYST, clocks: &Clocks) -> Self {
        Self {
            timer: syst,
            input_clk: clocks.sys_clk(),
        }
    }

    /// Recover the [pac::SYST] register from this timer.
    #[inline(always)]
    pub fn free(self) -> pac::SYST {
        // safety: self owns SYST, and we're dropping self here
        unsafe { pac::CorePeripherals::steal().SYST }
    }

    counter_methods!();
}
