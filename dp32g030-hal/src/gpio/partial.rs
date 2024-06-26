use crate::pac;

use super::{
    Alternate, ErasedPin, Floating, Input, IntoMode, OpenDrain, Output, Pin, PinInfo, PinMode,
    PinState, PullDown, PullUp, PushPull, WithMode,
};

/// A partially-erased pin with static port and dynamic number.
pub struct PartiallyErasedPin<const P: char, Mode = Input> {
    n: u8,
    _marker: core::marker::PhantomData<Mode>,
}

impl<const P: char, Mode> core::fmt::Debug for PartiallyErasedPin<P, Mode>
where
    Mode: PinMode,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("PartiallyErasedPin")
            .field(&P)
            .field(&self.n)
            .field(&Mode::default())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl<const P: char, Mode> defmt::Format for PartiallyErasedPin<P, Mode>
where
    Mode: PinMode + defmt::Format,
{
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "PartiallyErasedPin({}, {}, {})",
            P,
            self.n,
            Mode::default()
        )
    }
}

impl<const P: char, Mode> PartiallyErasedPin<P, Mode>
where
    Mode: PinMode,
{
    /// # Safety
    /// This must be the only place this pin is accessed in both
    /// PORTCON and GPIO, and the mode must match the pin's mode.
    pub(super) unsafe fn steal(n: u8) -> Self {
        Self {
            n,
            _marker: Default::default(),
        }
    }

    /// Get the pin number of this pin.
    pub fn pin(&self) -> u8 {
        self.n
    }

    /// Get the port of this pin.
    pub fn port(&self) -> char {
        P
    }

    /// Erase the pin number of a pin.
    pub fn erase<const N: u8>(_pin: Pin<P, N, Mode>) -> Self {
        // safety: we have ownership of this pin
        unsafe { Self::steal(N) }
    }

    /// Erase the port of this pin.
    pub fn erase_port(self) -> ErasedPin<Mode> {
        ErasedPin::erase_partial(self)
    }

    /// Restore the erased pin.
    pub fn restore<const N: u8>(self) -> Result<Pin<P, N, Mode>, Self> {
        if N == self.n {
            // safety: we own this pin via self, and drop self here.
            Ok(unsafe { Pin::steal() })
        } else {
            Err(self)
        }
    }

    /// Convert pin into a new mode.
    pub fn into_mode<M>(self) -> PartiallyErasedPin<P, M>
    where
        M: PinMode,
    {
        critical_section::with(|_cs| {
            // safety: we will consume this pin and return a new one
            // with valid type state, so we can access these register
            // Modifies occur inside a critical section.
            unsafe {
                use super::pin::change_mode;
                if P == 'A' {
                    change_mode!(pac::GPIOA, porta, self.n, Mode, M);
                } else if P == 'B' {
                    change_mode!(pac::GPIOB, portb, self.n, Mode, M);
                } else if P == 'C' {
                    change_mode!(pac::GPIOC, portc, self.n, Mode, M);
                } else {
                    // we never build these, someone did a naughty transmute
                    panic!();
                }
            }
        });

        // safety: we changed the mode above, and are consuming self
        unsafe { PartiallyErasedPin::steal(self.n) }
    }

    /// Convert pin into a new mode, in the given initial state.
    pub fn into_mode_in_state<M>(mut self, state: PinState) -> PartiallyErasedPin<P, Output<M>>
    where
        Output<M>: PinMode,
    {
        self.write_data(state);
        let mut pin = self.into_mode();
        pin.write_data(state);
        pin
    }

    /// Temporarily configure this pin in a new mode.
    ///
    /// If this is an output mode, the initial state is retained if
    /// the original mode was also an output mode. It is otherwise
    /// undefined.
    pub fn with_mode<M, R>(&mut self, f: impl FnOnce(&mut PartiallyErasedPin<P, M>) -> R) -> R
    where
        M: PinMode,
    {
        // safety: we have exclusive access to self, so we can create a copy
        // and then only use the copy until we discard it in the same mode
        let subpin = unsafe { Self::steal(self.n) };

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
        f: impl FnOnce(&mut PartiallyErasedPin<P, Output<M>>) -> R,
    ) -> R
    where
        Output<M>: PinMode,
    {
        self.write_data(state);
        self.with_mode(|p| {
            p.write_data(state);
            f(p)
        })
    }

    // internal helper to read data register
    fn read_data(&self) -> PinState {
        // safety: we control these registers, and can read them
        unsafe {
            if P == 'A' {
                pac::GPIOA::steal()
                    .data()
                    .read()
                    .data(self.n)
                    .is_high()
                    .into()
            } else if P == 'B' {
                pac::GPIOB::steal()
                    .data()
                    .read()
                    .data(self.n)
                    .is_high()
                    .into()
            } else if P == 'C' {
                pac::GPIOC::steal()
                    .data()
                    .read()
                    .data(self.n)
                    .is_high()
                    .into()
            } else {
                // we never build these, someone did a naughty transmute
                panic!();
            }
        }
    }

    // internal helper to write data register
    pub(super) fn write_data(&mut self, state: PinState) {
        critical_section::with(|_cs| {
            // safety: we control these registers and can write them
            // and we are inside a critical section so we can modify
            unsafe {
                if P == 'A' {
                    let gpio = pac::GPIOA::steal();
                    gpio.data()
                        .modify(|_r, w| w.data(self.n).bit(state.is_high()));
                } else if P == 'B' {
                    let gpio = pac::GPIOB::steal();
                    gpio.data()
                        .modify(|_r, w| w.data(self.n).bit(state.is_high()));
                } else if P == 'C' {
                    let gpio = pac::GPIOC::steal();
                    gpio.data()
                        .modify(|_r, w| w.data(self.n).bit(state.is_high()));
                } else {
                    // we never build these, someone did a naughty transmute
                    panic!();
                }
            }
        });
    }

    super::mode::into_mode_aliases!(vis pub, (PartiallyErasedPin), (P,));
    super::mode::with_mode_aliases!(vis pub, (PartiallyErasedPin), (P,));

    /// Convert pin into an alternate mode but otherwise preserve state.
    pub fn into_alternate<const A: u8>(self) -> PartiallyErasedPin<P, Alternate<A, Mode::Inner>>
    where
        Alternate<A, Mode::Inner>: PinMode,
    {
        self.into_mode()
    }

    /// Convert pin in alternate mode into a regular GPIO pin, but
    /// otherwise preserve state.
    pub fn into_gpio(self) -> PartiallyErasedPin<P, Mode::Inner> {
        self.into_mode()
    }
}

impl<const P: char, Pull> PartiallyErasedPin<P, Input<Pull>>
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

impl<const P: char> PartiallyErasedPin<P, Output<OpenDrain>> {
    /// Read the input pin.
    pub fn read(&self) -> PinState {
        if self.read_data().is_high() {
            // high means high-Z, turn into an input briefly to see
            // if something else is pulling us low
            // safety: we own this pin, and it's undone at the end
            unsafe {
                let mut pin = Self::steal(self.n);
                let r = pin.with_floating_input(|p| p.read());
                // cursed: pin output defaults to last input
                pin.write_data(PinState::High);
                r
            }
        } else {
            // we're pulling it low
            PinState::Low
        }
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

impl<const P: char, Mode> PartiallyErasedPin<P, Output<Mode>>
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
        // FIXME this could be done with xor
        self.set_state(!self.get_state());
    }
}

impl<const P: char, Mode> PinInfo for PartiallyErasedPin<P, Mode>
where
    Mode: PinMode,
{
    type Mode = Mode;

    fn pin(&self) -> u8 {
        PartiallyErasedPin::pin(self)
    }

    fn port(&self) -> char {
        PartiallyErasedPin::port(self)
    }
}

impl<const P: char, Mode> IntoMode for PartiallyErasedPin<P, Mode>
where
    Mode: PinMode,
{
    type As<M> = PartiallyErasedPin<P, M>;

    fn into_mode<M>(self) -> Self::As<M>
    where
        M: PinMode,
    {
        PartiallyErasedPin::into_mode(self)
    }

    fn into_mode_in_state<M>(self, state: PinState) -> Self::As<Output<M>>
    where
        Output<M>: PinMode,
    {
        PartiallyErasedPin::into_mode_in_state(self, state)
    }
}

impl<const P: char, Mode> WithMode for PartiallyErasedPin<P, Mode>
where
    Mode: PinMode,
{
    type With<M> = PartiallyErasedPin<P, M>;

    fn with_mode<M, R>(&mut self, f: impl FnOnce(&mut Self::With<M>) -> R) -> R
    where
        M: PinMode,
    {
        PartiallyErasedPin::with_mode(self, f)
    }

    fn with_mode_in_state<M, R>(
        &mut self,
        state: PinState,
        f: impl FnOnce(&mut Self::With<Output<M>>) -> R,
    ) -> R
    where
        Output<M>: PinMode,
    {
        PartiallyErasedPin::with_mode_in_state(self, state, f)
    }
}

impl<const P: char, const N: u8, Mode> From<Pin<P, N, Mode>> for PartiallyErasedPin<P, Mode>
where
    Mode: PinMode,
{
    fn from(value: Pin<P, N, Mode>) -> Self {
        Self::erase(value)
    }
}

impl<const P: char, Mode> TryFrom<ErasedPin<Mode>> for PartiallyErasedPin<P, Mode>
where
    Mode: PinMode,
{
    type Error = ErasedPin<Mode>;

    fn try_from(value: ErasedPin<Mode>) -> Result<Self, Self::Error> {
        value.restore_partial()
    }
}
