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
    #[inline(always)]
    pub fn new(config: Config<Uart, Data>, rx: Uart::Rx, rts: Flow<Uart::Rts>) -> Self {
        // safety: we have configured the uart
        unsafe {
            config.uart.ctrl().set_bits(|w| w.uarten().enabled());
            Self::setup(config.uart, rx, rts)
        }
    }

    /// Recover a configurator from an [RxOnly].
    #[inline(always)]
    pub fn free(self) -> (Config<Uart, Data>, Uart::Rx, Flow<Uart::Rts>) {
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

impl<Uart, Data, Pair> Rx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    /// # Safety
    /// This half of the port must not be in use anywhere else.
    #[inline(always)]
    pub(super) unsafe fn setup(uart: Uart, rx: Uart::Rx, rts: Flow<Uart::Rts>) -> Self {
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
    pub(super) fn teardown(self) -> (Uart, Uart::Rx, Flow<Uart::Rts>) {
        // safety: we're consuming self, so turn this off
        unsafe {
            self.uart.ctrl().clear_bits(|w| w.rxen().disabled());
            self.uart.fc().clear_bits(|w| w.rtsen().disabled());
        }
        (self.uart, self.rx, self.rts)
    }
}
