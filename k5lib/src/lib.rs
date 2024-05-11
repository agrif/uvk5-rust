#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
mod client;
#[cfg(feature = "std")]
pub use client::*;

pub mod protocol;

mod version;
pub use version::*;
