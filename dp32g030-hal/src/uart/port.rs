use core::convert::Infallible;

use crate::block;

use super::{Config, Flow, Instance, UartData};

/// A UART port, with both an [Rx] and a [Tx].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Port<Uart: Instance, Data> {
    pub rx: Rx<Uart, Data>,
    pub tx: Tx<Uart, Data>,
}

/// The Rx half of a UART.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Rx<Uart: Instance, Data, const PAIR: bool = true> {
    uart: Uart,
    rx: Uart::Rx,
    rts: Flow<Uart::Rts>,
    // this produces data
    _marker: core::marker::PhantomData<Data>,
}

/// The Tx half of a UART.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Tx<Uart: Instance, Data, const PAIR: bool = true> {
    uart: Uart,
    tx: Uart::Tx,
    cts: Flow<Uart::Cts>,
    // this consumes data
    _marker: core::marker::PhantomData<fn(Data) -> ()>,
}

impl<Uart, Data> Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    /// Create a Port from the configured UART.
    #[inline(always)]
    pub fn new(
        config: Config<Uart, Data, true>,
        rx: Uart::Rx,
        tx: Uart::Tx,
        rts: Flow<Uart::Rts>,
        cts: Flow<Uart::Cts>,
    ) -> Self {
        // safety: we have configured the uart
        unsafe {
            config.uart.ctrl().set_bits(|w| w.uarten().enabled());
        }
        Self {
            // safety: we are only using this on exclusive rx/tx sides
            rx: unsafe { Rx::setup(config.uart.steal(), rx, rts) },
            tx: unsafe { Tx::setup(config.uart, tx, cts) },
        }
    }

    /// Recover the Port into a configurator.
    #[allow(clippy::type_complexity)]
    #[inline(always)]
    pub fn free(
        self,
    ) -> (
        Config<Uart, Data, true>,
        Uart::Rx,
        Uart::Tx,
        Flow<Uart::Rts>,
        Flow<Uart::Cts>,
    ) {
        let (uart, rx, rts) = self.rx.teardown();
        let (_, tx, cts) = self.tx.teardown();

        // safety: we have closed both halves of the uart
        unsafe {
            uart.ctrl().clear_bits(|w| w.uarten().disabled());
        }

        (
            Config {
                uart,
                _marker: Default::default(),
            },
            rx,
            tx,
            rts,
            cts,
        )
    }
}

impl<Uart, Data> Rx<Uart, Data, false>
where
    Uart: Instance,
    Data: UartData,
{
    /// Create a lonely Rx from a configurator.
    #[inline(always)]
    pub fn new(config: Config<Uart, Data, true>, rx: Uart::Rx, rts: Flow<Uart::Rts>) -> Self {
        // safety: we have configured the uart
        unsafe {
            config.uart.ctrl().set_bits(|w| w.uarten().enabled());
            Self::setup(config.uart, rx, rts)
        }
    }

    /// Recover a configurator from a lonely Rx.
    #[inline(always)]
    pub fn free(self) -> (Config<Uart, Data, true>, Uart::Rx, Flow<Uart::Rts>) {
        let (uart, rx, rts) = self.teardown();

        // safety: we have closed this lonely half
        unsafe {
            uart.ctrl().clear_bits(|w| w.uarten().disabled());
        }

        (
            Config {
                uart,
                _marker: Default::default(),
            },
            rx,
            rts,
        )
    }
}

impl<Uart, Data, const PAIR: bool> Rx<Uart, Data, PAIR>
where
    Uart: Instance,
    Data: UartData,
{
    /// # Safety
    /// This half of the port must not be in use anywhere else.
    #[inline(always)]
    unsafe fn setup(uart: Uart, rx: Uart::Rx, rts: Flow<Uart::Rts>) -> Self {
        match rts {
            Flow::None => uart.fc().clear_bits(|w| w.rtsen().disabled()),
            Flow::ActiveHigh(_) => {
                uart.fc().set_bits(|w| w.rtspol().active_high());
                uart.fc().set_bits(|w| w.rtsen().enabled());
            }
            Flow::ActiveLow(_) => {
                uart.fc().clear_bits(|w| w.rtspol().active_low());
                uart.fc().set_bits(|w| w.rtsen().enabled());
            }
        }

        uart.fifo().set_bits(|w| w.rf_clr().clear());
        uart.ctrl().set_bits(|w| w.rxen().enabled());

        Self {
            uart,
            rx,
            rts,
            _marker: Default::default(),
        }
    }

    #[inline(always)]
    fn teardown(self) -> (Uart, Uart::Rx, Flow<Uart::Rts>) {
        // safety: we're consuming self, so turn this off
        unsafe {
            self.uart.ctrl().clear_bits(|w| w.rxen().disabled());
            self.uart.fc().clear_bits(|w| w.rtsen().disabled());
        }
        (self.uart, self.rx, self.rts)
    }
}

impl<Uart, Data> Tx<Uart, Data, false>
where
    Uart: Instance,
    Data: UartData,
{
    /// Create a lonely Rx from a configurator.
    #[inline(always)]
    pub fn new(config: Config<Uart, Data, true>, tx: Uart::Tx, cts: Flow<Uart::Cts>) -> Self {
        // safety: we have configured the uart
        unsafe {
            config.uart.ctrl().set_bits(|w| w.uarten().enabled());
            Self::setup(config.uart, tx, cts)
        }
    }

    /// Recover a configurator from a lonely Rx.
    #[inline(always)]
    pub fn free(self) -> (Config<Uart, Data, true>, Uart::Tx, Flow<Uart::Cts>) {
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

impl<Uart, Data, const PAIR: bool> Tx<Uart, Data, PAIR>
where
    Uart: Instance,
    Data: UartData,
{
    /// # Safety
    /// This half of the port must not be in use anywhere else.
    #[inline(always)]
    unsafe fn setup(uart: Uart, tx: Uart::Tx, cts: Flow<Uart::Cts>) -> Self {
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
    fn teardown(self) -> (Uart, Uart::Tx, Flow<Uart::Cts>) {
        // safety: we're consuming self, so turn this off
        unsafe {
            self.uart.ctrl().clear_bits(|w| w.txen().disabled());
            self.uart.fc().clear_bits(|w| w.ctsen().disabled());
        }
        (self.uart, self.tx, self.cts)
    }

    /// Is the FIFO full?
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.uart.if_().read().txfifo_full().is_full()
    }

    /// Is the FIFO empty?
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.uart.if_().read().txfifo_empty().is_empty()
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
        if !self.is_empty() {
            Err(block::Error::WouldBlock)
        } else {
            Ok(())
        }
    }
}

impl<Uart, Data, const PAIR: bool> core::fmt::Write for Tx<Uart, Data, PAIR>
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
