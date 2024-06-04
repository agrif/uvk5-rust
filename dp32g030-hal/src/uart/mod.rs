//! Interfaces for UART.

mod config;
pub use config::*;

mod data;
pub use data::*;

mod hal02;
mod hal1;

mod instance;
pub use instance::*;

mod port;
pub use port::*;

mod rx;
pub use rx::*;

mod tx;
pub use tx::*;
