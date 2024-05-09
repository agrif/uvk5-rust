use crate::pac;

use super::{
    Alternate, ErasedPin, Floating, Input, IntoMode, OpenDrain, Output, PartiallyErasedPin,
    PinMode, PullDown, PullUp, PushPull,
};

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

// allow this to be used elsewhere in gpio
pub(super) use change_mode;

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

    /// Get the pin number of this pin.
    #[inline(always)]
    pub fn pin(&self) -> u8 {
        N
    }

    /// Get the port of this pin.
    #[inline(always)]
    pub fn port(&self) -> char {
        P
    }

    /// Erase the pin number and port from the type.
    #[inline(always)]
    pub fn erase(self) -> ErasedPin<Mode> {
        ErasedPin::erase(self)
    }

    /// Erase the pin number from the type.
    #[inline(always)]
    pub fn erase_number(self) -> PartiallyErasedPin<P, Mode> {
        PartiallyErasedPin::erase(self)
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

    /// Convert pin into a new mode, in the given initial state.
    #[inline(always)]
    fn into_mode_in_state<M>(mut self, state: PinState) -> Pin<P, N, Output<M>>
    where
        Output<M>: PinMode,
    {
        self.write_data(state);
        self.into_mode()
    }

    /// Temporarily configure this pin in a new mode.
    ///
    /// If this is an output mode, the initial state is retained if
    /// the original mode was also an output mode. It is otherwise
    /// undefined.
    #[inline(always)]
    fn with_mode<M, R>(&mut self, f: impl FnOnce(&mut Pin<P, N, M>) -> R) -> R
    where
        M: PinMode,
    {
        // safety: we have exclusive access to self, so we can create a copy
        // and then only use the copy until we discard it in the same mode
        let subpin = unsafe { Self::steal() };

        // we must change mode back before returning
        let mut subpin = subpin.into_mode();
        let r = f(&mut subpin);
        // change mode back and drop it
        let _: Self = subpin.into_mode();

        r
    }

    /// Temporarily configure this pin in a new mode, in the given
    /// initial state.
    #[inline(always)]
    fn with_mode_in_state<M, R>(
        &mut self,
        state: PinState,
        f: impl FnOnce(&mut Pin<P, N, Output<M>>) -> R,
    ) -> R
    where
        Output<M>: PinMode,
    {
        self.write_data(state);
        self.with_mode(f)
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
    pub(super) fn write_data(&mut self, state: PinState) {
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

    super::mode::into_mode_aliases!(vis pub, (Pin), (P, N,));

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

impl<const P: char, const N: u8, Mode> IntoMode for Pin<P, N, Mode>
where
    Mode: PinMode,
{
    type As<M> = Pin<P, N, M>;

    #[inline(always)]
    fn into_mode<M>(self) -> Self::As<M>
    where
        M: PinMode,
    {
        Pin::into_mode(self)
    }

    #[inline(always)]
    fn into_mode_in_state<M>(self, state: PinState) -> Self::As<Output<M>>
    where
        Output<M>: PinMode,
    {
        Pin::into_mode_in_state(self, state)
    }

    #[inline(always)]
    fn with_mode<M, R>(&mut self, f: impl FnOnce(&mut Self::As<M>) -> R) -> R
    where
        M: PinMode,
    {
        Pin::with_mode(self, f)
    }

    #[inline(always)]
    fn with_mode_in_state<M, R>(
        &mut self,
        state: PinState,
        f: impl FnOnce(&mut Self::As<Output<M>>) -> R,
    ) -> R
    where
        Output<M>: PinMode,
    {
        Pin::with_mode_in_state(self, state, f)
    }
}

impl<const P: char, const N: u8, Mode> TryFrom<PartiallyErasedPin<P, Mode>> for Pin<P, N, Mode>
where
    Mode: PinMode,
{
    // FIXME actual pin erasure error?
    type Error = ();

    #[inline(always)]
    fn try_from(value: PartiallyErasedPin<P, Mode>) -> Result<Self, Self::Error> {
        value.restore().ok_or(())
    }
}

impl<const P: char, const N: u8, Mode> TryFrom<ErasedPin<Mode>> for Pin<P, N, Mode>
where
    Mode: PinMode,
{
    // FIXME actual pin erasure error?
    type Error = ();

    #[inline(always)]
    fn try_from(value: ErasedPin<Mode>) -> Result<Self, Self::Error> {
        value.restore().ok_or(())
    }
}
