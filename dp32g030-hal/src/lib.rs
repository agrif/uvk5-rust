#![no_std]

/// Peripheral access crate, providing raw, unconstrained access to
/// peripherals.
pub use dp32g030 as pac;

pub mod power;
