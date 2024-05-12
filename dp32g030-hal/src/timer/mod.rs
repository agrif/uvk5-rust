//! Interfaces for hardware timers.

use crate::pac;

use crate::power::Clocks;
use crate::time::Hertz;

mod config;
pub use config::*;

mod counter;
pub use counter::*;

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

/// The low half of a timer.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Low<Timer, const HZ: u32> {
    timer: Timer,
    input_clk: Hertz,
}

/// The high half of a timer.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct High<Timer, const HZ: u32> {
    timer: Timer,
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
        /// Create a Counter from this timer, with specific precision.
        ///
        /// This discards any statically-known frequency of the base
        /// timer, and instead calculates ticks from the dynamic
        /// sys_clk value. This *might* be more accurate, at the cost
        /// of runtime.
        #[inline(always)]
        pub fn counter_hz<const C_HZ: u32>(self) -> Counter<Self, C_HZ, true> {
            Counter::new(self)
        }

        /// Create a Counter with nanosecond precision with this timer.
        #[inline(always)]
        pub fn counter_ns(self) -> CounterNs<Self> {
            self.counter_hz()
        }

        /// Create a Counter with microsecond precision with this timer.
        #[inline(always)]
        pub fn counter_us(self) -> CounterUs<Self> {
            self.counter_hz()
        }

        /// Create a Counter with millisecond precision with this timer.
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

impl<Timer, const HZ: u32> Low<Timer, HZ>
where
    Timer: Base,
{
    counter_methods!(native);
}

impl<Timer, const HZ: u32> High<Timer, HZ>
where
    Timer: Base,
{
    counter_methods!(native);
}

/// Create the system timer from the SYST register;
#[inline(always)]
pub fn new_system(syst: pac::SYST, clocks: &Clocks) -> System {
    System::new(syst, clocks)
}

impl System {
    /// Create the system timer from the SYST register.
    #[inline(always)]
    pub fn new(syst: pac::SYST, clocks: &Clocks) -> Self {
        Self {
            timer: syst,
            input_clk: clocks.sys_clk(),
        }
    }

    /// Recover the SYST register from this timer.
    #[inline(always)]
    pub fn free(self) -> pac::SYST {
        // safety: self owns SYST, and we're dropping self here
        unsafe { pac::CorePeripherals::steal().SYST }
    }

    counter_methods!();
}
