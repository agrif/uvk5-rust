use core::convert::Infallible;
use embedded_hal_02::digital::v2 as hal02;
use embedded_hal_1::digital as hal1;

use crate::pac;

use super::{Alternate, Floating, Input, OpenDrain, Output, PinMode, PullDown, PullUp, PushPull};

/// Digital pin state.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PinState {
    Low = 0,
    High = 1,
}

impl From<bool> for PinState {
    #[inline(always)]
    fn from(value: bool) -> Self {
        if value {
            Self::High
        } else {
            Self::Low
        }
    }
}

impl From<hal02::PinState> for PinState {
    #[inline(always)]
    fn from(value: hal02::PinState) -> Self {
        match value {
            hal02::PinState::Low => Self::Low,
            hal02::PinState::High => Self::High,
        }
    }
}

impl From<PinState> for hal02::PinState {
    #[inline(always)]
    fn from(value: PinState) -> Self {
        match value {
            PinState::Low => Self::Low,
            PinState::High => Self::High,
        }
    }
}

impl From<hal1::PinState> for PinState {
    #[inline(always)]
    fn from(value: hal1::PinState) -> Self {
        match value {
            hal1::PinState::Low => Self::Low,
            hal1::PinState::High => Self::High,
        }
    }
}

impl From<PinState> for hal1::PinState {
    #[inline(always)]
    fn from(value: PinState) -> Self {
        match value {
            PinState::Low => Self::Low,
            PinState::High => Self::High,
        }
    }
}

impl core::ops::Not for PinState {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        match self {
            Self::High => Self::Low,
            Self::Low => Self::High,
        }
    }
}

impl PinState {
    /// Is the pin high?
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == Self::High
    }

    /// Is the pin low?
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == Self::Low
    }
}

/// A generic pin type, with type state indicating mode.
pub struct Pin<const P: char, const N: u8, Mode = Input> {
    _marker: core::marker::PhantomData<Mode>,
}

impl<const P: char, const N: u8, Mode> core::fmt::Debug for Pin<P, N, Mode>
where
    Mode: PinMode,
{
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("Pin")
            .field(&P)
            .field(&N)
            .field(&Mode::default())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl<const P: char, const N: u8, Mode> defmt::Format for Pin<P, N, Mode>
where
    Mode: PinMode + defmt::Format,
{
    #[allow(clippy::missing_inline_in_public_items)]
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Pin({}, {}, {})", P, N, Mode::default())
    }
}

// avoid repetitive code, unfortunately the Gpio registers have no
// generic interface.
macro_rules! change_mode {
    ($Gpio:ty, $port:ident, $N:expr, $From:ty, $To:ty) => {
        let force = <$From>::UNSPECIFIED;
        let portcon = pac::PORTCON::steal();
        let port = <$Gpio>::steal();

        if force || <$From>::IE != <$To>::IE {
            change_mode!(change portcon, $port, ie, $N, <$To>::IE);
        }
        if force || <$From>::PD != <$To>::PD {
            change_mode!(change portcon, $port, pd, $N, <$To>::PD);
        }
        if force || <$From>::PU != <$To>::PU {
            change_mode!(change portcon, $port, pu, $N, <$To>::PU);
        }

        if force || <$From>::OD != <$To>::OD {
            change_mode!(change portcon, $port, od, $N, <$To>::OD);
        }

        if force || <$From>::SEL != <$To>::SEL {
            change_mode!(function portcon, $port, $N, <$To>::SEL);
        }
        if force || <$From>::DIR != <$To>::DIR {
            if <$To>::DIR {
                port.dir().set_bits(|w| w.dir($N).output());
            } else {
                port.dir().clear_bits(|w| w.dir($N).input());
            }
        }
    };

    // helper to set a field on portcon
    // accesses port_name, index N, and then enables/disables based on val
    (change $portcon:expr, $port:ident, $name:ident, $N:expr, $val:expr) => {
        paste::paste! {
            if $val {
                $portcon.[<$port _ $name>]().set_bits(|w| w.[<$port _ $name>]($N).enabled());
            } else {
                $portcon.[<$port _ $name>]().set_bits(|w| w.[<$port _ $name>]($N).disabled());
            }
        }
    };

    // helper to set a function on portcon, for portc only
    // sets port_sel0/port_sel1, index N, based on val
    (function $portcon:expr, portc, $N:expr, $val:expr) => {
        match $N {
            0 => change_mode!(func-one $portcon, portc, sel0, 0, $val),
            1 => change_mode!(func-one $portcon, portc, sel0, 1, $val),
            2 => change_mode!(func-one $portcon, portc, sel0, 2, $val),
            3 => change_mode!(func-one $portcon, portc, sel0, 3, $val),

            4 => change_mode!(func-one $portcon, portc, sel0, 4, $val),
            5 => change_mode!(func-one $portcon, portc, sel0, 5, $val),
            6 => change_mode!(func-one $portcon, portc, sel0, 6, $val),
            7 => change_mode!(func-one $portcon, portc, sel0, 7, $val),

            _ => panic!(), // we never construct these
        }
    };

    // helper to set a function on portcon
    // sets port_sel0/port_sel1, index N, based on val
    (function $portcon:expr, $port:ident, $N:expr, $val:expr) => {
        match $N {
            0 => change_mode!(func-one $portcon, $port, sel0, 0, $val),
            1 => change_mode!(func-one $portcon, $port, sel0, 1, $val),
            2 => change_mode!(func-one $portcon, $port, sel0, 2, $val),
            3 => change_mode!(func-one $portcon, $port, sel0, 3, $val),

            4 => change_mode!(func-one $portcon, $port, sel0, 4, $val),
            5 => change_mode!(func-one $portcon, $port, sel0, 5, $val),
            6 => change_mode!(func-one $portcon, $port, sel0, 6, $val),
            7 => change_mode!(func-one $portcon, $port, sel0, 7, $val),

            8 => change_mode!(func-one $portcon, $port, sel1, 8, $val),
            9 => change_mode!(func-one $portcon, $port, sel1, 9, $val),
            10 => change_mode!(func-one $portcon, $port, sel1, 10, $val),
            11 => change_mode!(func-one $portcon, $port, sel1, 11, $val),

            12 => change_mode!(func-one $portcon, $port, sel1, 12, $val),
            13 => change_mode!(func-one $portcon, $port, sel1, 13, $val),
            14 => change_mode!(func-one $portcon, $port, sel1, 14, $val),
            15 => change_mode!(func-one $portcon, $port, sel1, 15, $val),

            _ => panic!(), // we never construct these
        }
    };

    // helper to set a single port_sel/port to a value, used above
    (func-one $portcon:expr, $port:ident, $sel:ident, $pin:tt, $val:expr) => {
        paste::paste! {
            {
                $portcon.[<$port _ $sel>]().clear_bits(|w| w.[<$port $pin>]().bits(0));
                $portcon.[<$port _ $sel>]().set_bits(|w| w.[<$port $pin>]().bits($val));
            }
        }
    };
}

impl<const P: char, const N: u8, Mode> Pin<P, N, Mode>
where
    Mode: PinMode,
{
    /// Safety: this must be the only place this pin is accessed in both
    /// PORTCON and GPIO. You should also be sure Mode matches the pin's mode.
    #[inline(always)]
    pub(crate) unsafe fn steal() -> Self {
        Pin {
            _marker: Default::default(),
        }
    }

    /// Convert pin into a new mode.
    #[inline(always)]
    pub fn into_mode<M>(self) -> Pin<P, N, M>
    where
        M: PinMode,
    {
        // safety: we will be immediately returning a pin with valid
        // type state, and consuming this pin.
        unsafe {
            if P == 'A' {
                change_mode!(pac::GPIOA, porta, N, Mode, M);
            } else if P == 'B' {
                change_mode!(pac::GPIOB, portb, N, Mode, M);
            } else if P == 'C' {
                change_mode!(pac::GPIOC, portc, N, Mode, M);
            } else {
                // we never build these, someone did a naughty transmute
                panic!();
            }
        }
        // safety: we have changed the mode above, and we are consuming
        // the existing token owning this pin (self)
        unsafe { Pin::steal() }
    }

    // internal helper to read data register
    #[inline(always)]
    fn read_data(&self) -> PinState {
        // safety: we control these registers, and can read them
        unsafe {
            if P == 'A' {
                pac::GPIOA::steal().data().read().data(N).is_high().into()
            } else if P == 'B' {
                pac::GPIOB::steal().data().read().data(N).is_high().into()
            } else if P == 'C' {
                pac::GPIOC::steal().data().read().data(N).is_high().into()
            } else {
                // we never build these, someone did a naughty transmute
                panic!();
            }
        }
    }

    // internal helper to write data register
    #[inline(always)]
    fn write_data(&mut self, state: PinState) {
        // safety: we control these registers and can write them
        unsafe {
            if P == 'A' {
                let gpio = pac::GPIOA::steal();
                if state.is_high() {
                    gpio.data().set_bits(|w| w.data(N).high());
                } else {
                    gpio.data().clear_bits(|w| w.data(N).low());
                }
            } else if P == 'B' {
                let gpio = pac::GPIOB::steal();
                if state.is_high() {
                    gpio.data().set_bits(|w| w.data(N).high());
                } else {
                    gpio.data().clear_bits(|w| w.data(N).low());
                }
            } else if P == 'C' {
                let gpio = pac::GPIOC::steal();
                if state.is_high() {
                    gpio.data().set_bits(|w| w.data(N).high());
                } else {
                    gpio.data().clear_bits(|w| w.data(N).low());
                }
            } else {
                // we never build these, someone did a naughty transmute
                panic!();
            }
        }
    }

    /// Convert pin into a floating input.
    #[inline(always)]
    pub fn into_floating_input(self) -> Pin<P, N, Input<Floating>> {
        self.into_mode()
    }

    /// Convert pin into an input with a pull-up resistor.
    #[inline(always)]
    pub fn into_pull_up_input(self) -> Pin<P, N, Input<PullUp>> {
        self.into_mode()
    }

    /// Convert pin into an input with a pull-down resistor.
    #[inline(always)]
    pub fn into_pull_down_input(self) -> Pin<P, N, Input<PullDown>> {
        self.into_mode()
    }

    /// Convert pin into a push-pull output, initially low.
    #[inline(always)]
    pub fn into_push_pull_output(self) -> Pin<P, N, Output<PushPull>> {
        self.into_push_pull_output_in_state(PinState::Low)
    }

    /// Convert a pin into a push-pull output in the given state.
    #[inline(always)]
    pub fn into_push_pull_output_in_state(
        mut self,
        state: PinState,
    ) -> Pin<P, N, Output<PushPull>> {
        self.write_data(state);
        self.into_mode()
    }

    /// Convert pin into an open-drain output, initially low.
    #[inline(always)]
    pub fn into_open_drain_output(self) -> Pin<P, N, Output<OpenDrain>> {
        self.into_open_drain_output_in_state(PinState::Low)
    }

    /// Convert pin into an open-drain output, initially low.
    #[inline(always)]
    pub fn into_open_drain_output_in_state(
        mut self,
        state: PinState,
    ) -> Pin<P, N, Output<OpenDrain>> {
        self.write_data(state);
        self.into_mode()
    }

    /// Convert pin into an alternate mode but otherwise preserve state.
    #[inline(always)]
    pub fn into_alternate<const A: u8>(self) -> Pin<P, N, Alternate<A, Mode::Inner>>
    where
        Alternate<A, Mode::Inner>: PinMode,
    {
        self.into_mode()
    }

    /// Convert pin in alternate mode into a regular GPIO pin, but
    /// otherwise preserve state.
    #[inline(always)]
    pub fn into_gpio(self) -> Pin<P, N, Mode::Inner> {
        self.into_mode()
    }
}

impl<const P: char, const N: u8, Pull> Pin<P, N, Input<Pull>>
where
    Input<Pull>: PinMode,
{
    /// Read the input pin.
    #[inline(always)]
    pub fn read(&self) -> PinState {
        self.read_data()
    }

    /// Is the input pin high?
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        self.read().is_high()
    }

    /// Is the input pin low?
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        self.read().is_low()
    }
}

impl<const P: char, const N: u8, Mode> Pin<P, N, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    /// Get the current output drive state.
    #[inline(always)]
    pub fn get_state(&self) -> PinState {
        self.read_data()
    }

    /// Is the output set high?
    #[inline(always)]
    pub fn is_set_high(&self) -> bool {
        self.get_state().is_high()
    }

    /// Is the output set low?
    #[inline(always)]
    pub fn is_set_low(&self) -> bool {
        self.get_state().is_low()
    }

    /// Set the current output drive state.
    #[inline(always)]
    pub fn set_state(&mut self, state: PinState) {
        self.write_data(state);
    }

    /// Set the current output high.
    #[inline(always)]
    pub fn set_high(&mut self) {
        self.set_state(PinState::High);
    }

    /// Set the current output low.
    #[inline(always)]
    pub fn set_low(&mut self) {
        self.set_state(PinState::Low);
    }

    /// Toggle the output.
    #[inline(always)]
    pub fn toggle(&mut self) {
        // FIXME this could be done with atomic xor
        self.set_state(!self.get_state());
    }
}

impl<const P: char, const N: u8, Pull, OMode, Mode>
    hal02::IoPin<Pin<P, N, Input<Pull>>, Pin<P, N, Output<OMode>>> for Pin<P, N, Mode>
where
    Input<Pull>: PinMode,
    Output<OMode>: PinMode,
    Mode: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn into_input_pin(self) -> Result<Pin<P, N, Input<Pull>>, Self::Error> {
        Ok(self.into_mode())
    }

    #[inline(always)]
    fn into_output_pin(
        mut self,
        state: hal02::PinState,
    ) -> Result<Pin<P, N, Output<OMode>>, Self::Error> {
        self.write_data(state.into());
        Ok(self.into_mode())
    }
}

impl<const P: char, const N: u8, Pull> hal02::InputPin for Pin<P, N, Input<Pull>>
where
    Input<Pull>: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(Pin::is_high(self))
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(Pin::is_low(self))
    }
}

impl<const P: char, const N: u8, Mode> hal02::OutputPin for Pin<P, N, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Pin::set_low(self);
        Ok(())
    }

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Pin::set_high(self);
        Ok(())
    }

    #[inline(always)]
    fn set_state(&mut self, state: hal02::PinState) -> Result<(), Self::Error> {
        Pin::set_state(self, state.into());
        Ok(())
    }
}

impl<const P: char, const N: u8, Mode> hal02::StatefulOutputPin for Pin<P, N, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    #[inline(always)]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(Pin::is_set_high(self))
    }

    #[inline(always)]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(Pin::is_set_low(self))
    }
}

impl<const P: char, const N: u8, Mode> hal02::ToggleableOutputPin for Pin<P, N, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        Pin::toggle(self);
        Ok(())
    }
}

impl<const P: char, const N: u8, Mode> hal1::ErrorType for Pin<P, N, Mode>
where
    Mode: PinMode,
{
    type Error = Infallible;
}

impl<const P: char, const N: u8, Pull> hal1::InputPin for Pin<P, N, Input<Pull>>
where
    Input<Pull>: PinMode,
{
    #[inline(always)]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::is_high(self))
    }

    #[inline(always)]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::is_low(self))
    }
}

impl<const P: char, const N: u8, Mode> hal1::OutputPin for Pin<P, N, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Pin::set_low(self);
        Ok(())
    }

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Pin::set_high(self);
        Ok(())
    }

    #[inline(always)]
    fn set_state(&mut self, state: hal1::PinState) -> Result<(), Self::Error> {
        Pin::set_state(self, state.into());
        Ok(())
    }
}

impl<const P: char, const N: u8, Mode> hal1::StatefulOutputPin for Pin<P, N, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    #[inline(always)]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::is_set_high(self))
    }

    #[inline(always)]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::is_set_low(self))
    }

    #[inline(always)]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        Pin::toggle(self);
        Ok(())
    }
}
