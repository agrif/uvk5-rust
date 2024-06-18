//! UART on the headset connector.

use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut};

use k5lib::{ArrayBuffer, ClientBuffer};

use crate::hal::block;
use crate::hal::gpio::{Alternate, Floating, Input, Output, PushPull, PA7, PA8};
use crate::hal::power::Gate;
use crate::hal::time::Hertz;
use crate::hal::uart;
use crate::pac::portcon::{porta_sel0, porta_sel1};
use crate::pac::UART1;

/// The pins and peripherals required for the UART.
#[derive(Debug)]
// defmt not implemented for UART1 (??)
pub struct Parts {
    /// The UART1 peripheral.
    pub uart: UART1,
    /// The gate controlling UART1 power.
    pub gate: Gate<UART1>,
    /// UART Tx pin.
    pub tx: PA7<Alternate<{ porta_sel0::PORTA7_A::Uart1Tx as u8 }, Output<PushPull>>>,
    /// UART Rx pin.
    pub rx: PA8<Alternate<{ porta_sel1::PORTA8_A::Uart1Rx as u8 }, Input<Floating>>>,
}

pub use uart::Error;

/// The UART interface.
pub type Uart = uart::Port<UART1>;

/// The Rx half of the UART.
pub type Rx = uart::Rx<UART1>;

/// The Tx half of the UART.
pub type Tx = uart::Tx<UART1>;

/// The global [k5lib::Client], created by [GlobalUart::client()].
pub type ClientRadio = k5lib::ClientRadio<GlobalUart, &'static mut ArrayBuffer>;

/// Create a new UART from parts.
pub fn new(baud: Hertz, parts: Parts) -> Result<Uart, Error> {
    Ok(uart::new(parts.uart, parts.gate, baud)?.port(parts.rx.into(), parts.tx.into()))
}

// the global UART
static RX: spin::Mutex<Option<Rx>> = spin::Mutex::new(None);
static TX: spin::Mutex<Option<Tx>> = spin::Mutex::new(None);

/// Print a line to the global UART.
///
/// See [install()] for how to install a global UART.
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Print to the global UART.
///
/// See [install()] for how to install a global UART.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
/// Internal print function, used by [print!()] macro.
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    if let Some(mut tx) = try_tx() {
        // intentionally ignore possible errors. This is best-effort,
        // it should not panic.
        let _ = write!(tx, "{}", args);
    }
}

/// Flush the global UART output.
///
/// See [install()] for how to install a global UART.
pub fn flush() {
    // best effort, ignore errors
    if let Some(mut tx) = try_tx() {
        let _ = block::block!(tx.flush());
    }
}

/// A token indicating the UART has been installed globally.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GlobalUart {
    _priv: (),
}

/// Install the UART as the global handler.
///
/// This allows the use of [println!()] and [print!()] anywhere.
pub fn install(uart: Uart) -> GlobalUart {
    GlobalUart::install(uart)
}

impl GlobalUart {
    /// Install the UART as the global handler.
    ///
    /// This allows the use of [println!()] and [print!()] anywhere.
    pub fn install(uart: Uart) -> Self {
        let mut rx = RX.lock();
        let mut tx = TX.lock();
        *rx = Some(uart.rx);
        *tx = Some(uart.tx);
        GlobalUart { _priv: () }
    }

    /// Try to install the UART as the global handler.
    ///
    /// This will fail, rather than lock up, if the global UART locks are held.
    pub fn try_install(uart: Uart) -> Result<Self, Uart> {
        if let Some(mut rx) = RX.try_lock() {
            if let Some(mut tx) = TX.try_lock() {
                *rx = Some(uart.rx);
                *tx = Some(uart.tx);
                return Ok(GlobalUart { _priv: () });
            }
        }

        Err(uart)
    }

    /// Uninstall the global handler, recovering the UART.
    pub fn uninstall(self) -> Uart {
        let mut rx = RX.lock();
        let mut tx = TX.lock();
        // unwrap is ok: owning this token means these were set
        Uart {
            rx: rx.take().unwrap(),
            tx: tx.take().unwrap(),
        }
    }

    /// Try to uninstall the global handler, recovering the UART.
    ///
    /// This will fail, rather than lock up, if the global UART locks are held.
    pub fn try_uninstall(self) -> Result<Uart, Self> {
        if let Some(mut rx) = RX.try_lock() {
            if let Some(mut tx) = TX.try_lock() {
                // unwrap is ok: owning this token means these were set
                return Ok(Uart {
                    rx: rx.take().unwrap(),
                    tx: tx.take().unwrap(),
                });
            }
        }

        Err(self)
    }

    /// Get the global [Rx] exclusively.
    pub fn lock_rx(&self) -> Proxy<Rx> {
        // unwrap is ok: we have reference to the token that sets it
        Proxy::new(RX.lock()).unwrap()
    }

    /// Get the global [Tx] exclusively.
    pub fn lock_tx(&self) -> Proxy<Tx> {
        // unwrap is ok: we have reference to the token that sets it
        Proxy::new(TX.lock()).unwrap()
    }

    /// Crate a [ClientRadio] from the global UART.
    pub fn client(self) -> ClientRadio {
        static mut BUFFER: UnsafeCell<ArrayBuffer> = UnsafeCell::new(ArrayBuffer::new());
        // safety: this is only ever accessed by a ClientRadio, which
        // owns a GlobalUart.  GlobalUart is a unique value, and we
        // own it now in self, so we're safe
        unsafe {
            let buf = BUFFER.get().as_mut().unwrap();
            buf.clear();
            ClientRadio::new_with(buf, self)
        }
    }
}

impl embedded_io::ErrorType for GlobalUart {
    type Error = core::convert::Infallible;
}

impl<'a> embedded_io::ErrorType for &'a GlobalUart {
    type Error = core::convert::Infallible;
}

impl embedded_io::Read for GlobalUart {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        embedded_io::Read::read(self.lock_rx().deref_mut(), buf)
    }

    fn read_exact(
        &mut self,
        buf: &mut [u8],
    ) -> Result<(), embedded_io::ReadExactError<Self::Error>> {
        embedded_io::Read::read_exact(self.lock_rx().deref_mut(), buf)
    }
}

impl<'a> embedded_io::Read for &'a GlobalUart {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        embedded_io::Read::read(self.lock_rx().deref_mut(), buf)
    }

    fn read_exact(
        &mut self,
        buf: &mut [u8],
    ) -> Result<(), embedded_io::ReadExactError<Self::Error>> {
        embedded_io::Read::read_exact(self.lock_rx().deref_mut(), buf)
    }
}

impl embedded_io::ReadReady for GlobalUart {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        embedded_io::ReadReady::read_ready(self.lock_rx().deref_mut())
    }
}

impl<'a> embedded_io::ReadReady for &'a GlobalUart {
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        embedded_io::ReadReady::read_ready(self.lock_rx().deref_mut())
    }
}

impl embedded_io::Write for GlobalUart {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        embedded_io::Write::write(self.lock_tx().deref_mut(), buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_io::Write::flush(self.lock_tx().deref_mut())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        embedded_io::Write::write_all(self.lock_tx().deref_mut(), buf)
    }

    fn write_fmt(
        &mut self,
        fmt: core::fmt::Arguments<'_>,
    ) -> Result<(), embedded_io::WriteFmtError<Self::Error>> {
        embedded_io::Write::write_fmt(self.lock_tx().deref_mut(), fmt)
    }
}

impl<'a> embedded_io::Write for &'a GlobalUart {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        embedded_io::Write::write(self.lock_tx().deref_mut(), buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_io::Write::flush(self.lock_tx().deref_mut())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        embedded_io::Write::write_all(self.lock_tx().deref_mut(), buf)
    }

    fn write_fmt(
        &mut self,
        fmt: core::fmt::Arguments<'_>,
    ) -> Result<(), embedded_io::WriteFmtError<Self::Error>> {
        embedded_io::Write::write_fmt(self.lock_tx().deref_mut(), fmt)
    }
}

impl embedded_io::WriteReady for GlobalUart {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        embedded_io::WriteReady::write_ready(self.lock_tx().deref_mut())
    }
}

impl<'a> embedded_io::WriteReady for &'a GlobalUart {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        embedded_io::WriteReady::write_ready(self.lock_tx().deref_mut())
    }
}

/// A proxy type for the global UART.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Proxy<T: 'static> {
    guard: spin::MutexGuard<'static, Option<T>>,
}

impl<T> Proxy<T> {
    fn new(guard: spin::MutexGuard<'static, Option<T>>) -> Option<Self> {
        if guard.is_none() {
            None
        } else {
            Some(Proxy { guard })
        }
    }
}

impl<T> Deref for Proxy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // unwrap is ok, new guarantees this is_some()
        self.guard.as_ref().unwrap()
    }
}

impl<T> DerefMut for Proxy<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // unwrap is ok, new guarantees this is_some()
        self.guard.as_mut().unwrap()
    }
}

/// Try to get the global [Rx].
///
/// This will fail if the global UART is not set, or if Rx is in use already.
pub fn try_rx() -> Option<Proxy<Rx>> {
    Proxy::new(RX.try_lock()?)
}

/// Try to get the global [Tx].
///
/// This will fail if the global UART is not set, or if Tx is in use already.
pub fn try_tx() -> Option<Proxy<Tx>> {
    Proxy::new(TX.try_lock()?)
}
