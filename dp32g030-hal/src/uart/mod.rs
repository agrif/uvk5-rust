//! Interfaces for UART.

mod config;
pub use config::*;
mod data;
pub use data::*;
mod peripherals;
pub use peripherals::*;
mod port;
pub use port::*;
