use core::convert::Infallible;

use crate::block;

use super::{Config, Flow, Instance, Lonely, Paired, UartData};

/// The Rx half of a UART.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Rx<Uart: Instance, Data = u8, Pair = Paired> {
    uart: Uart,
    rx: Uart::Rx,
    rts: Flow<Uart::Rts>,
    // this produces data
    _marker: core::marker::PhantomData<(Data, Pair)>,
}

/// A UART configured for only [Rx].
pub type RxOnly<Uart, Data = u8> = Rx<Uart, Data, Lonely>;

impl<Uart, Data> RxOnly<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    /// Create an [RxOnly] from a configurator.
    pub fn new(config: Config<Uart, Data>, rx: Uart::Rx, rts: Flow<Uart::Rts>) -> Self {
        // safety: we have configured the uart
        unsafe {
            config.uart.ctrl().modify(|_r, w| w.uarten().enabled());
            Self::setup(config.uart, rx, rts)
        }
    }

    /// Recover a configurator from an [RxOnly].
    pub fn free(self) -> (Config<Uart, Data>, Uart::Rx, Flow<Uart::Rts>) {
        // safety: we are the only user of this uart
        let (uart, rx, rts) = unsafe { self.teardown() };

        uart.ctrl().modify(|_r, w| w.uarten().disabled());

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

impl<Uart, Data, Pair> Rx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    /// # Safety
    /// This entire port must not be in use anywhere else, and this half
    /// must not be owned anywhere else.
    pub(super) unsafe fn setup(uart: Uart, rx: Uart::Rx, rts: Flow<Uart::Rts>) -> Self {
        // we know the port is not in use anywhere else, so no critical section
        // even though these registers are shared between halves
        match rts {
            Flow::None => uart.fc().modify(|_r, w| w.rtsen().disabled()),
            Flow::ActiveHigh(_) => {
                uart.fc()
                    .modify(|_r, w| w.rtspol().active_high().rtsen().enabled());
            }
            Flow::ActiveLow(_) => {
                uart.fc()
                    .modify(|_r, w| w.rtspol().active_low().rtsen().enabled());
            }
        }

        let mut rx = Self {
            uart,
            rx,
            rts,
            _marker: Default::default(),
        };

        rx.clear();
        rx.uart.ctrl().modify(|_r, w| w.rxen().enabled());

        rx
    }

    /// # Safety
    /// This entire port must not be in use anywhere else, and this half
    /// must not be owned anywhere else.
    pub(super) unsafe fn teardown(self) -> (Uart, Uart::Rx, Flow<Uart::Rts>) {
        self.uart.ctrl().modify(|_r, w| w.rxen().disabled());
        self.uart.fc().modify(|_r, w| w.rtsen().disabled());
        (self.uart, self.rx, self.rts)
    }

    /// Clear the FIFO.
    pub fn clear(&mut self) {
        critical_section::with(|_cs| {
            // this register is shared but we're in a critical section
            self.uart.fifo().modify(|_r, w| w.rf_clr().clear());
        });
    }

    /// Is the FIFO full?
    pub fn is_full(&self) -> bool {
        self.uart.if_().read().rxfifo_full().is_full()
    }

    /// Is the FIFO half full?
    pub fn is_half_full(&self) -> bool {
        self.uart.if_().read().rxfifo_hfull().is_half_full()
    }

    /// Is the FIFO empty?
    pub fn is_empty(&self) -> bool {
        self.uart.if_().read().rxfifo_empty().is_empty()
    }

    /// Get the FIFO level, 0 is empty and 8 is full.
    pub fn level(&self) -> u8 {
        match self.uart.if_().read().rf_level().bits() {
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

    /// Read a single byte from the UART.
    pub fn read_one(&mut self) -> block::Result<u8, Infallible> {
        if self.is_empty() {
            Err(block::Error::WouldBlock)
        } else {
            Ok(self.uart.rdr().read().data().bits())
        }
    }

    /// Read at least one byte from the UART.
    pub fn read(&mut self, buf: &mut [u8]) -> block::Result<usize, Infallible> {
        let mut amt = 0;
        while amt < buf.len() {
            match self.read_one() {
                Ok(b) => {
                    buf[amt] = b;
                    amt += 1;
                    continue;
                }
                Err(block::Error::WouldBlock) => {
                    if amt == 0 {
                        return Err(block::Error::WouldBlock);
                    } else {
                        return Ok(amt);
                    }
                }
                Err(block::Error::Other(e)) => match e {},
            }
        }
        Ok(amt)
    }

    /// Read bytes from the UART, filling the buffer and blocking if needed.
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Infallible> {
        let mut start = 0;
        while start < buf.len() {
            start += block::block!(self.read(&mut buf[start..]))?;
        }

        Ok(())
    }
}
