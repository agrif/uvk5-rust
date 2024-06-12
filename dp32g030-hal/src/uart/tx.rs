use core::convert::Infallible;

use crate::block;

use super::{Config, Flow, Instance, Lonely, Paired, UartData};

/// The Tx half of a UART.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Tx<Uart: Instance, Data = u8, Pair = Paired> {
    uart: Uart,
    tx: Uart::Tx,
    cts: Flow<Uart::Cts>,
    // this consumes data
    _marker: core::marker::PhantomData<(fn(Data) -> (), Pair)>,
}

/// A UART configured for only [Tx].
pub type TxOnly<Uart, Data = u8> = Tx<Uart, Data, Lonely>;

impl<Uart, Data> TxOnly<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    /// Create a [TxOnly] from a configurator.
    pub fn new(config: Config<Uart, Data>, tx: Uart::Tx, cts: Flow<Uart::Cts>) -> Self {
        // safety: we have configured the uart
        unsafe {
            config.uart.ctrl().modify(|_r, w| w.uarten().enabled());
            Self::setup(config.uart, tx, cts)
        }
    }

    /// Recover a configurator from a [TxOnly].
    pub fn free(self) -> (Config<Uart, Data>, Uart::Tx, Flow<Uart::Cts>) {
        // safety: we are the only user of this uart
        let (uart, tx, cts) = unsafe { self.teardown() };

        uart.ctrl().modify(|_r, w| w.uarten().disabled());

        (
            Config {
                uart,
                _marker: Default::default(),
            },
            tx,
            cts,
        )
    }
}

impl<Uart, Data, Pair> Tx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    /// # Safety
    /// This entire of the port must not be in use anywhere else, and this half
    /// must not be owned anywhere else.
    pub(super) unsafe fn setup(uart: Uart, tx: Uart::Tx, cts: Flow<Uart::Cts>) -> Self {
        // we know the port is not in use anywhere else, so no critical section
        // even though these registers are shared between halves
        match cts {
            Flow::None => uart.fc().modify(|_r, w| w.ctsen().disabled()),
            Flow::ActiveHigh(_) => {
                uart.fc()
                    .modify(|_r, w| w.ctspol().active_high().ctsen().enabled());
            }
            Flow::ActiveLow(_) => {
                uart.fc()
                    .modify(|_r, w| w.ctspol().active_low().ctsen().enabled());
            }
        }

        let mut tx = Self {
            uart,
            tx,
            cts,
            _marker: Default::default(),
        };

        tx.clear();
        tx.uart.ctrl().modify(|_r, w| w.txen().enabled());

        tx
    }

    /// # Safety
    /// This entire port must not be in use anywhere else, and this half
    /// must not be owned anywhere else.
    pub(super) unsafe fn teardown(self) -> (Uart, Uart::Tx, Flow<Uart::Cts>) {
        self.uart.ctrl().modify(|_r, w| w.txen().disabled());
        self.uart.fc().modify(|_r, w| w.ctsen().disabled());
        (self.uart, self.tx, self.cts)
    }

    /// Clear the FIFO.
    pub fn clear(&mut self) {
        critical_section::with(|_cs| {
            // this register is shared but we're in a critical section
            self.uart.fifo().modify(|_r, w| w.tf_clr().clear());
        });
    }

    /// Is the transmitter busy?
    pub fn is_busy(&self) -> bool {
        self.uart.if_().read().txbusy().is_busy()
    }

    /// Is the FIFO full?
    pub fn is_full(&self) -> bool {
        self.uart.if_().read().txfifo_full().is_full()
    }

    /// Is the FIFO half full?
    pub fn is_half_full(&self) -> bool {
        self.uart.if_().read().txfifo_hfull().is_half_full()
    }

    /// Is the FIFO empty?
    pub fn is_empty(&self) -> bool {
        self.uart.if_().read().txfifo_empty().is_empty()
    }

    /// Get the FIFO level, 0 is empty and 8 is full.
    pub fn level(&self) -> u8 {
        match self.uart.if_().read().tf_level().bits() {
            0 => {
                if self.is_full() {
                    8
                } else {
                    0
                }
            }
            l => l,
        }
    }

    /// Write a single byte to the UART.
    pub fn write_one(&mut self, data: u8) -> block::Result<(), Infallible> {
        if self.is_full() {
            Err(block::Error::WouldBlock)
        } else {
            self.uart.tdr().write(|w| w.data().set(data));
            Ok(())
        }
    }

    /// Write at least one byte to the UART.
    pub fn write(&mut self, data: &[u8]) -> block::Result<usize, Infallible> {
        for (i, b) in data.iter().enumerate() {
            match self.write_one(*b) {
                Ok(()) => continue,
                Err(block::Error::WouldBlock) => {
                    if i == 0 {
                        return Err(block::Error::WouldBlock);
                    } else {
                        return Ok(i);
                    }
                }
                Err(block::Error::Other(e)) => match e {},
            }
        }

        Ok(data.len())
    }

    /// Write all bytes to the UART, blocking as needed.
    pub fn write_all(&mut self, data: &[u8]) -> Result<(), Infallible> {
        for b in data {
            block::block!(self.write_one(*b))?;
        }

        Ok(())
    }

    /// Flush the Tx FIFO.
    pub fn flush(&mut self) -> block::Result<(), Infallible> {
        if self.is_busy() {
            Err(block::Error::WouldBlock)
        } else {
            Ok(())
        }
    }
}

impl<Uart, Data, Pair> core::fmt::Write for Tx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_all(s.as_bytes()).unwrap_or_else(|e| match e {});
        Ok(())
    }
}
