#![no_std]

pub mod prelude;

/// HAL crate, providing structured access to peripherals.
pub use dp32g030_hal as hal;

/// Peripheral access crate, providing raw, unconstrained access to peripherals.
pub use hal::pac;

pub mod backlight;
pub mod flashlight;
pub mod lcd;
pub mod ptt;
pub mod uart;

/// A macro for producing a `VERSION` symbol containing the given string
/// literal, prefixed by a "*".
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
        static VERSION: &::core::ffi::CStr = unsafe {
            ::core::ffi::CStr::from_bytes_with_nul_unchecked(
                concat!("*", $version, "\0").as_bytes(),
            )
        };
    };
}
