//! Interfaces for power and clock control.

use crate::pac;

mod chip_id;
pub use chip_id::*;

mod clocks;
pub use clocks::*;

mod gate;
pub use gate::*;

/// Create a clock and power configurator from the relevant registers.
pub fn new(syscon: pac::SYSCON, pmu: pac::PMU) -> Config {
    Config::new(syscon, pmu)
}

/// Peripherals that control power and the clock.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Power {
    pub chip_id: ChipId,
    pub clocks: Clocks,
    pub gates: Gates,
}

impl Power {
    /// # Safety
    /// This will duplicate access to [pac::SYSCON] and
    /// [pac::PMU] unless those are known to not yet exist or have
    /// been dropped.
    unsafe fn steal(clocks: Clocks) -> Self {
        Self {
            chip_id: ChipId::steal(),
            clocks,
            gates: Gates::steal(),
        }
    }

    /// Freeze the configuration to obtain the HAL peripherals.
    pub fn new(config: Config) -> Self {
        config.freeze()
    }

    // hmm... free?
}
