//! Interfaces for UART.

mod config;
pub use config::*;
mod peripherals;
pub use peripherals::*;
mod port;
pub use port::*;
