//! Interfaces for power and clock control.

use crate::pac;

mod chip_id;
pub use chip_id::*;

mod clocks;
pub use clocks::*;

mod dev_gate;
pub use dev_gate::*;

/// Peripherals that control power and the clock.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerParts {
    pub chip_id: ChipId,
    pub clocks: ClockConfig,
    pub dev_gate: DevGate,
}

#[inline(always)]
/// Split the SYSCON and PMU registers into usable parts.
pub fn split(_syscon: pac::SYSCON, _pmu: pac::PMU) -> PowerParts {
    // safety: all of these operate on disjoint registers of these blocks
    // which we are now owners of
    unsafe {
        PowerParts {
            chip_id: ChipId::steal(),
            clocks: ClockConfig::steal(),
            dev_gate: DevGate::steal(),
        }
    }
}
