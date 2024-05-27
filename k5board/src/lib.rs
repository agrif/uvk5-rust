#![no_std]

pub mod prelude;

/// HAL crate, providing structured access to peripherals.
pub use dp32g030_hal as hal;

/// Peripheral access crate, providing raw, unconstrained access to peripherals.
pub use hal::pac;
