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

/// The low half of a timer.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Low<Timer> {
    timer: Timer,
    input_clk: Hertz,
}

/// The high half of a timer.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct High<Timer> {
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
        /// Create a Counter from this timer.
        #[inline(always)]
        pub fn counter<const HZ: u32>(self) -> Counter<Self, HZ> {
            Counter::new(self)
        }

        /// Create a Counter with nanosecond precision with this timer.
        #[inline(always)]
        pub fn counter_ns(self) -> CounterNs<Self> {
            self.counter()
        }

        /// Create a Counter with microsecond precision with this timer.
        #[inline(always)]
        pub fn counter_us(self) -> CounterUs<Self> {
            self.counter()
        }

        /// Create a Counter with millisecond precision with this timer.
        #[inline(always)]
        pub fn counter_ms(self) -> CounterMs<Self> {
            self.counter()
        }
    };
}

impl<Timer> Low<Timer>
where
    Timer: Base,
{
    counter_methods!();
}

impl<Timer> High<Timer>
where
    Timer: Base,
{
    counter_methods!();
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
