//! Interfaces for SPI.

mod config;
pub use config::*;

mod hal02;
mod hal1;

mod instance;
pub use instance::*;

mod port;
pub use port::*;
