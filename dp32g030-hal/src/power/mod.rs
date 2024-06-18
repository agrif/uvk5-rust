//! Interfaces for power and clock control.

use dp32g030_hal_flash::Code;

use crate::pac;

mod chip_id;
pub use chip_id::*;

mod clocks;
pub use clocks::*;

mod gate;
pub use gate::*;

/// Create a clock and power configurator from the relevant registers.
///
/// This uses the built-in flash [Code] that will be loaded in RAM
/// forever. If you need to manage this storage yourself, see
/// [new_with_code].
pub fn new(syscon: pac::SYSCON, pmu: pac::PMU, flash: pac::FLASH_CTRL) -> Config<'static> {
    static FLASH_CODE: Code = Code::new();
    new_with_code(syscon, pmu, flash, &FLASH_CODE)
}

/// Create a configurator using a different flash code storage.
///
/// This allows you to put the flash [Code] that needs to be in RAM on
/// the heap or stack, so you can deallocate it later. If you don't
/// care, use [new] to use the built-in storage that remains in RAM
/// forever.
pub fn new_with_code(
    syscon: pac::SYSCON,
    pmu: pac::PMU,
    flash: pac::FLASH_CTRL,
    flash_code: &Code,
) -> Config {
    Config::new(syscon, pmu, flash, flash_code)
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
