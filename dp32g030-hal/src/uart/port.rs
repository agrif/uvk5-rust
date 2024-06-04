use super::{Config, Flow, Instance, Rx, Tx, UartData};

/// A UART port, with both an [Rx] and a [Tx].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Port<Uart: Instance, Data = u8> {
    pub rx: Rx<Uart, Data>,
    pub tx: Tx<Uart, Data>,
}

/// An [Rx] or [Tx] with a matching pair from a [Port]. (typestate)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Paired;

/// An [Rx] or [Tx] without a matching pair. (typestate)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Lonely;

impl<Uart, Data> Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    /// Create a Port from the configured UART.
    pub fn new(
        config: Config<Uart, Data>,
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
    pub fn free(
        self,
    ) -> (
        Config<Uart, Data>,
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

impl<Uart, Data> core::fmt::Write for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.tx
            .write_all(s.as_bytes())
            .unwrap_or_else(|e| match e {});
        Ok(())
    }
}
