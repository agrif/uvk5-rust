//! Interfaces for UART.

mod config;
pub use config::*;
mod data;
pub use data::*;
mod instance;
pub use instance::*;
mod port;
pub use port::*;
mod rx;
pub use rx::*;
mod tx;
pub use tx::*;