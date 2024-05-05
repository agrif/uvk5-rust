#![no_std]

// sealed trait used all over
trait Sealed {}

/// Peripheral access crate, providing raw, unconstrained access to
/// peripherals.
pub use dp32g030 as pac;

pub mod gpio;
pub mod power;
