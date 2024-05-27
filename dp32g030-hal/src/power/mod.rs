//! Interfaces for power and clock control.

use crate::pac;

mod chip_id;
pub use chip_id::*;

mod clocks;
pub use clocks::*;

mod gate;
pub use gate::*;

/// Split the [pac::SYSCON] and [pac::PMU] registers into usable parts.
pub fn new(syscon: pac::SYSCON, pmu: pac::PMU) -> Power {
    Power::new(syscon, pmu)
}

/// Peripherals that control power and the clock.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Power {
    pub chip_id: ChipId,
    pub clocks: ClockConfig,
    pub gates: Gates,
}

impl Power {
    /// # Safety
    /// This accesses [pac::SYSCON] and [pac::PMU] registers. Notably,
    /// having this allows you to change the clock speed out from under
    /// all other peripherals.
    unsafe fn steal() -> Self {
        Self {
            chip_id: ChipId::steal(),
            clocks: ClockConfig::steal(),
            gates: Gates::steal(),
        }
    }

    /// Split the [pac::SYSCON] and [pac::PMU] registers into usable parts.
    pub fn new(_syscon: pac::SYSCON, _pmu: pac::PMU) -> Self {
        // safety: all of these operate on disjoint registers of these blocks
        // which we are now owners of
        unsafe { Self::steal() }
    }
}
