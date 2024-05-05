//! Interfaces for power and clock control.

use crate::pac;

mod chip_id;
pub use chip_id::*;

mod clocks;
pub use clocks::*;

mod dev_gate;
pub use dev_gate::*;

#[inline(always)]
/// Split the SYSCON and PMU registers into usable parts.
pub fn new(syscon: pac::SYSCON, pmu: pac::PMU) -> Power {
    Power::new(syscon, pmu)
}

/// Peripherals that control power and the clock.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Power {
    pub chip_id: ChipId,
    pub clocks: ClockConfig,
    pub dev_gate: DevGate,
}

impl Power {
    #[inline(always)]
    /// Split the SYSCON and PMU registers into usable parts.
    pub fn new(_syscon: pac::SYSCON, _pmu: pac::PMU) -> Self {
        // safety: all of these operate on disjoint registers of these blocks
        // which we are now owners of
        unsafe {
            Self {
                chip_id: ChipId::steal(),
                clocks: ClockConfig::steal(),
                dev_gate: DevGate::steal(),
            }
        }
    }
}
