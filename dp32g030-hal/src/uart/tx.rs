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
    #[inline(always)]
    pub fn new(config: Config<Uart, Data>, tx: Uart::Tx, cts: Flow<Uart::Cts>) -> Self {
        // safety: we have configured the uart
        unsafe {
            config.uart.ctrl().set_bits(|w| w.uarten().enabled());
            Self::setup(config.uart, tx, cts)
        }
    }

    /// Recover a configurator from a [TxOnly].
    #[inline(always)]
    pub fn free(self) -> (Config<Uart, Data>, Uart::Tx, Flow<Uart::Cts>) {
        let (uart, tx, cts) = self.teardown();

        // safety: we have closed this lonely half
        unsafe {
            uart.ctrl().clear_bits(|w| w.uarten().disabled());
        }

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
    /// This half of the port must not be in use anywhere else.
    #[inline(always)]
    pub(super) unsafe fn setup(uart: Uart, tx: Uart::Tx, cts: Flow<Uart::Cts>) -> Self {
        match cts {
            Flow::None => uart.fc().clear_bits(|w| w.ctsen().disabled()),
            Flow::ActiveHigh(_) => {
                uart.fc().set_bits(|w| w.ctspol().active_high());
                uart.fc().set_bits(|w| w.ctsen().enabled());
            }
            Flow::ActiveLow(_) => {
                uart.fc().clear_bits(|w| w.ctspol().active_low());
                uart.fc().set_bits(|w| w.ctsen().enabled());
            }
        }

        uart.fifo().set_bits(|w| w.tf_clr().clear());
        uart.ctrl().set_bits(|w| w.txen().enabled());

        Self {
            uart,
            tx,
            cts,
            _marker: Default::default(),
        }
    }

    #[inline(always)]
    pub(super) fn teardown(self) -> (Uart, Uart::Tx, Flow<Uart::Cts>) {
        // safety: we're consuming self, so turn this off
        unsafe {
            self.uart.ctrl().clear_bits(|w| w.txen().disabled());
            self.uart.fc().clear_bits(|w| w.ctsen().disabled());
        }
        (self.uart, self.tx, self.cts)
    }

    /// Is the transmitter busy?
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        self.uart.if_().read().txbusy().is_busy()
    }

    /// Is the FIFO full?
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.uart.if_().read().txfifo_full().is_full()
    }

    /// Is the FIFO half full?
    #[inline(always)]
    pub fn is_half_full(&self) -> bool {
        self.uart.if_().read().txfifo_hfull().is_half_full()
    }

    /// Is the FIFO empty?
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.uart.if_().read().txfifo_empty().is_empty()
    }

    /// Get the FIFO level, 0 is empty and 8 is full.
    #[inline(always)]
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
    #[inline(always)]
    pub fn write_one(&mut self, data: u8) -> block::Result<(), Infallible> {
        if self.is_full() {
            Err(block::Error::WouldBlock)
        } else {
            self.uart.tdr().write(|w| w.data().set(data));
            Ok(())
        }
    }

    /// Write at least one byte to the UART.
    #[inline(always)]
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
    #[inline(always)]
    pub fn write_all(&mut self, data: &[u8]) -> Result<(), Infallible> {
        for b in data {
            block::block!(self.write_one(*b))?;
        }

        Ok(())
    }

    /// Flush the Tx FIFO.
    #[inline(always)]
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
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_all(s.as_bytes()).unwrap_or_else(|e| match e {});
        Ok(())
    }
}
