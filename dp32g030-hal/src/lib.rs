#![no_std]

// we really, *really* want to be carefuly about marking functions here
// as inline. A ton of them are *tiny* and should be inline(always).
// If a function shouldn't be inlined, explicitly mark it with:
// #[allow(clippy::missing_inline_in_public_items)]
#![deny(clippy::missing_inline_in_public_items)]

/// Peripheral access crate, providing raw, unconstrained access to
/// peripherals.
pub use dp32g030 as pac;

pub mod gpio;
pub mod power;

// FIXME this should probably be in a board support crate, not here
/// A macro for producing a VERSION symbol containing the given string
/// literal, prefixed by a *.
///
/// Note, you must either use this in your program or otherwise tell
/// the linker to keep it, or it will be pruned during compilation.
///
/// One method is to add "EXTERN(VERSION);" to memory.x.
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
