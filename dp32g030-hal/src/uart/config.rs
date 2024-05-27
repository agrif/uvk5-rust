use crate::power::{Clocks, Gate};
use crate::time::Hertz;

use crate::pac;

use super::{Instance, Port, RxOnly, TxOnly, UartData};

/// Wrap a UART register into a configurator. Returns [Err] if baud
/// rate is not achievable.
pub fn new<Uart>(
    uart: Uart,
    gate: Gate<Uart>,
    clocks: &Clocks,
    baud: Hertz,
) -> Result<Config<Uart>, Error>
where
    Uart: Instance,
{
    Config::new(uart, gate, clocks, baud)
}

/// UART configuration error.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Requested baud rate is out of range.
    OutOfRange,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "UART Error {:?}", self)
    }
}

/// A UART configurator.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<Uart: Instance, Data = u8> {
    pub(super) uart: Uart,
    // this consumes and produces Data, so it goes on both sides
    pub(super) _marker: core::marker::PhantomData<fn(Data) -> Data>,
}

/// Choices for TX delay.
pub type TxDelay = pac::uart0::ctrl::TX_DLY_A;

/// Choices for automatic baud rate detection length.
pub type AutoBaudLen = pac::uart0::ctrl::ABRDBIT_A;

/// Choices for parity bit.
pub type Parity = pac::uart0::ctrl::PARMD_A;

/// Flow control presence and polarity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Flow<Pin> {
    /// This flow control pin is unused.
    None,
    /// This flow control pin is active low.
    ActiveLow(Pin),
    /// This flow control pin is active high.
    ActiveHigh(Pin),
}

impl<Uart> Config<Uart, u8>
where
    Uart: Instance,
{
    /// Wrap a UART register into a configurator. Returns [Err] if baud
    /// rate is not achievable.
    pub fn new(
        uart: Uart,
        mut gate: Gate<Uart>,
        clocks: &Clocks,
        baud: Hertz,
    ) -> Result<Self, Error> {
        gate.enable();

        // safety: we now own this uart, we can reset what we want
        uart.ctrl().reset();
        uart.baud().reset();
        uart.ie().reset();
        uart.if_().reset();
        uart.fifo().reset();
        uart.fc().reset();
        uart.rxto().reset();

        let config = Self {
            uart,
            _marker: Default::default(),
        };

        // must set baud here, otherwise it's 0 which is meaningless
        config.baud(clocks, baud)
    }
}

impl<Uart, Data> Config<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    /// Recover the UART register from a configurator.
    pub fn free(self) -> (Uart, Gate<Uart>) {
        // safety: we own this peripheral in self, and are dropping self
        unsafe {
            let mut gate = Gate::steal();
            gate.disable();
            (self.uart, gate)
        }
    }

    /// Set delay in bits between stop and start bits.
    pub fn tx_delay(self, delay: TxDelay) -> Self {
        // safety: we are sole owner of uart
        unsafe {
            self.uart.ctrl().clear_bits(|w| w.tx_dly().bits(0));
            self.uart.ctrl().set_bits(|w| w.tx_dly().variant(delay));
        }
        self
    }

    /// Get the delay in bits between stop and start bits.
    pub fn get_tx_delay(&self) -> TxDelay {
        self.uart.ctrl().read().tx_dly().variant()
    }

    /// Set the automatic baud rate detection length.
    pub fn auto_baud_len(self, len: AutoBaudLen) -> Self {
        // safety: we are sole owner of uart
        unsafe {
            self.uart.ctrl().clear_bits(|w| w.abrdbit().bits(0));
            self.uart.ctrl().set_bits(|w| w.abrdbit().variant(len))
        }
        self
    }

    /// Get the automatic baud rate detection length.
    pub fn get_auto_baud_len(&self) -> AutoBaudLen {
        self.uart.ctrl().read().abrdbit().variant()
    }

    /// Set parity mode. [None] means no parity bit.
    pub fn parity(self, parity: Option<Parity>) -> Self {
        // safety: we are sole owner of uart
        unsafe {
            match parity {
                Some(par) => {
                    self.uart.ctrl().clear_bits(|w| w.parmd().bits(0));
                    self.uart.ctrl().set_bits(|w| w.parmd().variant(par));
                    self.uart.ctrl().set_bits(|w| w.paren().enabled())
                }
                None => {
                    self.uart.ctrl().clear_bits(|w| w.paren().disabled());
                }
            }
        }
        self
    }

    /// Get parity mode. [None] means no parity bit.
    pub fn get_parity(&self) -> Option<Parity> {
        if self.uart.ctrl().read().paren().is_enabled() {
            Some(self.uart.ctrl().read().parmd().variant())
        } else {
            None
        }
    }

    /// Set nine-bit mode.
    pub fn ninebit(self) -> Config<Uart, u16> {
        // safety: we are sole owner of uart
        unsafe {
            self.uart.ctrl().set_bits(|w| w.ninebit().enabled());
        }
        Config {
            uart: self.uart,
            _marker: Default::default(),
        }
    }

    /// Set eight-bit mode.
    pub fn eightbit(self) -> Config<Uart, u8> {
        // safety: we are sole owner of uart
        unsafe {
            self.uart.ctrl().clear_bits(|w| w.ninebit().disabled());
        }
        Config {
            uart: self.uart,
            _marker: Default::default(),
        }
    }

    /// Is this a nine-bit UART?
    pub fn is_ninebit(&self) -> bool {
        Data::NINEBIT
    }

    /// Set the baud rate. Returns none if `baud` is not achievable.
    pub fn baud(self, clocks: &Clocks, baud: Hertz) -> Result<Self, Error> {
        let counter = clocks
            .sys_clk()
            .checked_add(baud / 2)
            .ok_or(Error::OutOfRange)?
            / baud;
        if counter > u16::MAX as u32 {
            return Err(Error::OutOfRange);
        }

        self.uart
            .baud()
            // safety: we are sole owner of uart
            .write(|w| unsafe { w.baud().bits(counter as u16) });

        Ok(Config {
            uart: self.uart,
            _marker: Default::default(),
        })
    }

    /// Get the baud rate.
    pub fn get_baud(&self, clocks: &Clocks) -> Hertz {
        clocks.sys_clk() / self.uart.baud().read().baud().bits() as u32
    }

    /// Get the configured [Port] using the provided pins.
    pub fn port(self, rx: Uart::Rx, tx: Uart::Tx) -> Port<Uart, Data> {
        self.port_flow(rx, tx, Flow::None, Flow::None)
    }

    /// Get the configured [Port] using the provided pins and flow control.
    pub fn port_flow(
        self,
        rx: Uart::Rx,
        tx: Uart::Tx,
        rts: Flow<Uart::Rts>,
        cts: Flow<Uart::Cts>,
    ) -> Port<Uart, Data> {
        Port::new(self, rx, tx, rts, cts)
    }

    /// Get the configured [RxOnly] using the provided pins.
    pub fn rx(self, rx: Uart::Rx) -> RxOnly<Uart, Data> {
        self.rx_flow(rx, Flow::None)
    }

    /// Get the configured [RxOnly] using the provided pins and flow control.
    pub fn rx_flow(self, rx: Uart::Rx, rts: Flow<Uart::Rts>) -> RxOnly<Uart, Data> {
        RxOnly::new(self, rx, rts)
    }

    /// Get the configured [TxOnly] using the provided pins.
    pub fn tx(self, tx: Uart::Tx) -> TxOnly<Uart, Data> {
        self.tx_flow(tx, Flow::None)
    }

    /// Get the configured [TxOnly] using the provided pins and flow control.
    pub fn tx_flow(self, tx: Uart::Tx, cts: Flow<Uart::Cts>) -> TxOnly<Uart, Data> {
        TxOnly::new(self, tx, cts)
    }
}
