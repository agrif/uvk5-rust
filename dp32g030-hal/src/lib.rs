#![no_std]

pub mod prelude;

/// Peripheral access crate, providing raw, unconstrained access to
/// peripherals.
pub use dp32g030 as pac;

pub mod block;
pub mod gpio;
pub mod power;
pub mod spi;
pub mod time;
pub mod timer;
pub mod uart;
