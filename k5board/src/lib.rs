#![no_std]

pub mod prelude;

/// HAL crate, providing structured access to peripherals.
pub use dp32g030_hal as hal;

/// Peripheral access crate, providing raw, unconstrained access to peripherals.
pub use hal::pac;

pub mod backlight;
pub mod eeprom;
pub mod flashlight;
pub mod keypad;
pub mod lcd;
pub mod shared_i2c;
pub mod uart;

#[cfg(not(feature = "defmt"))]
use bitflags::bitflags;

#[cfg(feature = "defmt")]
use defmt::bitflags;

/// A k5lib Version, re-exported here so [version!()] can use it.
pub use k5lib::Version;

/// A macro for producing a `VERSION` symbol containing the given string
/// literal, prefixed by a "*". This will be a &[k5lib::Version].
///
/// Sufficiently smart programming tools can extract this value from
/// the compiled ELF file while flashing the radio.
///
/// Note, you must either use this in your program or otherwise tell
/// the linker to keep it, or it will be pruned during compilation.
///
/// One method is to add `EXTERN(VERSION);` to memory.x. See the crate
/// sources for an example.
#[macro_export]
macro_rules! version {
    // use expr not literal, so we can accept things like env!(..)
    ($version:expr) => {
        #[no_mangle]
        static VERSION: &$crate::Version =
            &match $crate::Version::new_from_str(concat!("*", $version)) {
                ::core::result::Result::Ok(v) => v,
                ::core::result::Result::Err(e) => panic!("could not build version"),
            };
    };
}
