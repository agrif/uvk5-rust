use crate::pac;

use super::{
    Alternate, Floating, Input, IntoMode, OpenDrain, Output, PartiallyErasedPin, Pin, PinInfo,
    PinMode, PinState, PullDown, PullUp, PushPull, WithMode,
};

/// An erased pin with dynamic port and pin number.
pub struct ErasedPin<Mode = Input> {
    // bits 0-3 are pin, 4-7 are port, starting at A
    pin_port: u8,
    _marker: core::marker::PhantomData<Mode>,
}

impl<Mode> core::fmt::Debug for ErasedPin<Mode>
where
    Mode: PinMode,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (pin, port) = self.pin_port();
        f.debug_tuple("ErasedPin")
            .field(&pin)
            .field(&port)
            .field(&Mode::default())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl<Mode> defmt::Format for ErasedPin<Mode>
where
    Mode: PinMode + defmt::Format,
{
    fn format(&self, f: defmt::Formatter) {
        let (pin, port) = self.pin_port();
        defmt::write!(f, "ErasedPin({}, {}, {})", pin, port, Mode::default())
    }
}

impl<Mode> ErasedPin<Mode>
where
    Mode: PinMode,
{
    /// # Safety
    /// This must be the only place this pin is accessed in both
    /// PORTCON and GPIO, and the mode must match the pin's mode.
    pub(super) unsafe fn steal(n: u8, p: char) -> Self {
        assert!(n < 16);
        let port = match p {
            'A' => 0,
            'B' => 1,
            'C' => 2,
            _ => {
                // we never make these ourselves
                panic!()
            }
        };
        Self {
            pin_port: n | (port << 4),
            _marker: Default::default(),
        }
    }

    /// Get the port and pin number of this pin.
    fn pin_port(&self) -> (u8, char) {
        let pin = self.pin_port & 0x0f;
        let port = match (self.pin_port & 0xf0) >> 4 {
            0 => 'A',
            1 => 'B',
            2 => 'C',
            _ => {
                // we never make these
                panic!()
            }
        };

        (pin, port)
    }

    /// Get the pin number of this pin.
    pub fn pin(&self) -> u8 {
        self.pin_port().0
    }

    /// Get the port of this pin.
    pub fn port(&self) -> char {
        self.pin_port().1
    }

    /// Erase the pin number and port of a pin.
    pub fn erase<const P: char, const N: u8>(_pin: Pin<P, N, Mode>) -> Self {
        // safety: we have ownership of this pin
        unsafe { Self::steal(N, P) }
    }

    /// Erase the port of a partially-erased pin.
    pub fn erase_partial<const P: char>(pin: PartiallyErasedPin<P, Mode>) -> Self {
        // safety: we have ownership of this pin
        unsafe { Self::steal(pin.pin(), P) }
    }

    /// Restore the erased pin.
    pub fn restore<const P: char, const N: u8>(self) -> Result<Pin<P, N, Mode>, Self> {
        let (pin, port) = self.pin_port();
        if N == pin && P == port {
            // safety: we own this pin via self, and drop self here.
            Ok(unsafe { Pin::steal() })
        } else {
            Err(self)
        }
    }

    /// Restore the erased pin into a partially-erased pin.
    pub fn restore_partial<const P: char>(self) -> Result<PartiallyErasedPin<P, Mode>, Self> {
        let (pin, port) = self.pin_port();
        if P == port {
            // safety: we own this pin via self, and drop self here
            Ok(unsafe { PartiallyErasedPin::steal(pin) })
        } else {
            Err(self)
        }
    }

    /// Convert pin into a new mode.
    pub fn into_mode<M>(self) -> ErasedPin<M>
    where
        M: PinMode,
    {
        let (pin, port) = self.pin_port();

        // safety: we will consume this pin and return a new one
        // with valid type state, so we can access these register
        unsafe {
            use super::pin::change_mode;
            if port == 'A' {
                change_mode!(pac::GPIOA, porta, pin, Mode, M);
            } else if port == 'B' {
                change_mode!(pac::GPIOB, portb, pin, Mode, M);
            } else if port == 'C' {
                change_mode!(pac::GPIOC, portc, pin, Mode, M);
            } else {
                // we never build these, someone did a naughty transmute
                panic!();
            }
        }

        // safety: we changed the mode above, and are consuming self
        unsafe { ErasedPin::steal(pin, port) }
    }

    /// Convert pin into a new mode, in the given initial state.
    pub fn into_mode_in_state<M>(mut self, state: PinState) -> ErasedPin<Output<M>>
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
    pub fn with_mode<M, R>(&mut self, f: impl FnOnce(&mut ErasedPin<M>) -> R) -> R
    where
        M: PinMode,
    {
        let (pin, port) = self.pin_port();

        // safety: we have exclusive access to self, so we can create a copy
        // and then only use the copy until we discard it in the same mode
        let subpin = unsafe { Self::steal(pin, port) };

        // we must change mode back before returning
        let mut subpin = subpin.into_mode();
        let r = f(&mut subpin);
        // change mode back and drop it
        let _: Self = subpin.into_mode();

        r
    }

    /// Temporarily configure this pin in a new mode, in the given
    /// initial state.
    pub fn with_mode_in_state<M, R>(
        &mut self,
        state: PinState,
        f: impl FnOnce(&mut ErasedPin<Output<M>>) -> R,
    ) -> R
    where
        Output<M>: PinMode,
    {
        self.write_data(state);
        self.with_mode(f)
    }

    // internal helper to read data register
    fn read_data(&self) -> PinState {
        let (pin, port) = self.pin_port();

        // safety: we control these registers, and can read them
        unsafe {
            if port == 'A' {
                pac::GPIOA::steal().data().read().data(pin).is_high().into()
            } else if port == 'B' {
                pac::GPIOB::steal().data().read().data(pin).is_high().into()
            } else if port == 'C' {
                pac::GPIOC::steal().data().read().data(pin).is_high().into()
            } else {
                // we never build these, someone did a naughty transmute
                panic!();
            }
        }
    }

    // internal helper to write data register
    pub(super) fn write_data(&mut self, state: PinState) {
        let (pin, port) = self.pin_port();

        // safety: we control these registers and can write them
        unsafe {
            if port == 'A' {
                let gpio = pac::GPIOA::steal();
                if state.is_high() {
                    gpio.data().set_bits(|w| w.data(pin).high());
                } else {
                    gpio.data().clear_bits(|w| w.data(pin).low());
                }
            } else if port == 'B' {
                let gpio = pac::GPIOB::steal();
                if state.is_high() {
                    gpio.data().set_bits(|w| w.data(pin).high());
                } else {
                    gpio.data().clear_bits(|w| w.data(pin).low());
                }
            } else if port == 'C' {
                let gpio = pac::GPIOC::steal();
                if state.is_high() {
                    gpio.data().set_bits(|w| w.data(pin).high());
                } else {
                    gpio.data().clear_bits(|w| w.data(pin).low());
                }
            } else {
                // we never build these, someone did a naughty transmute
                panic!();
            }
        }
    }

    super::mode::into_mode_aliases!(vis pub, (ErasedPin), ());
    super::mode::with_mode_aliases!(vis pub, (ErasedPin), ());

    /// Convert pin into an alternate mode but otherwise preserve state.
    pub fn into_alternate<const A: u8>(self) -> ErasedPin<Alternate<A, Mode::Inner>>
    where
        Alternate<A, Mode::Inner>: PinMode,
    {
        self.into_mode()
    }

    /// Convert pin in alternate mode into a regular GPIO pin, but
    /// otherwise preserve state.
    pub fn into_gpio(self) -> ErasedPin<Mode::Inner> {
        self.into_mode()
    }
}

impl<Pull> ErasedPin<Input<Pull>>
where
    Input<Pull>: PinMode,
{
    /// Read the input pin.
    pub fn read(&self) -> PinState {
        self.read_data()
    }

    /// Is the input pin high?
    pub fn is_high(&self) -> bool {
        self.read().is_high()
    }

    /// Is the input pin low?
    pub fn is_low(&self) -> bool {
        self.read().is_low()
    }
}

impl<Mode> ErasedPin<Output<Mode>>
where
    Output<Mode>: PinMode,
{
    /// Get the current output drive state.
    pub fn get_state(&self) -> PinState {
        self.read_data()
    }

    /// Is the output set high?
    pub fn is_set_high(&self) -> bool {
        self.get_state().is_high()
    }

    /// Is the output set low?
    pub fn is_set_low(&self) -> bool {
        self.get_state().is_low()
    }

    /// Set the current output drive state.
    pub fn set_state(&mut self, state: PinState) {
        self.write_data(state);
    }

    /// Set the current output high.
    pub fn set_high(&mut self) {
        self.set_state(PinState::High);
    }

    /// Set the current output low.
    pub fn set_low(&mut self) {
        self.set_state(PinState::Low);
    }

    /// Toggle the output.
    pub fn toggle(&mut self) {
        // FIXME this could be done with atomic xor
        self.set_state(!self.get_state());
    }
}

impl<Mode> PinInfo for ErasedPin<Mode>
where
    Mode: PinMode,
{
    type Mode = Mode;

    fn pin(&self) -> u8 {
        ErasedPin::pin(self)
    }

    fn port(&self) -> char {
        ErasedPin::port(self)
    }
}

impl<Mode> IntoMode for ErasedPin<Mode>
where
    Mode: PinMode,
{
    type As<M> = ErasedPin<M>;

    fn into_mode<M>(self) -> Self::As<M>
    where
        M: PinMode,
    {
        ErasedPin::into_mode(self)
    }

    fn into_mode_in_state<M>(self, state: PinState) -> Self::As<Output<M>>
    where
        Output<M>: PinMode,
    {
        ErasedPin::into_mode_in_state(self, state)
    }
}

impl<Mode> WithMode for ErasedPin<Mode>
where
    Mode: PinMode,
{
    type With<M> = ErasedPin<M>;

    fn with_mode<M, R>(&mut self, f: impl FnOnce(&mut Self::With<M>) -> R) -> R
    where
        M: PinMode,
    {
        ErasedPin::with_mode(self, f)
    }

    fn with_mode_in_state<M, R>(
        &mut self,
        state: PinState,
        f: impl FnOnce(&mut Self::With<Output<M>>) -> R,
    ) -> R
    where
        Output<M>: PinMode,
    {
        ErasedPin::with_mode_in_state(self, state, f)
    }
}

impl<const P: char, const N: u8, Mode> From<Pin<P, N, Mode>> for ErasedPin<Mode>
where
    Mode: PinMode,
{
    fn from(value: Pin<P, N, Mode>) -> Self {
        Self::erase(value)
    }
}

impl<const P: char, Mode> From<PartiallyErasedPin<P, Mode>> for ErasedPin<Mode>
where
    Mode: PinMode,
{
    fn from(value: PartiallyErasedPin<P, Mode>) -> Self {
        Self::erase_partial(value)
    }
}
