use core::convert::Infallible;

use embedded_hal_02::digital::v2 as hal02;

use super::{
    ErasedPin, Floating, Input, OpenDrain, Output, PartiallyErasedPin, Pin, PinInfo, PinMode,
    PinState, PullDown, PullUp, PushPull, WithMode,
};

/// A pin that can be shared between users.
///
/// Reads and writes to pins are non-reentrant, and thus can be
/// shared. However, using the same pin at the same time will likely
/// confuse whatever the pin is used for, and so you may wish to avoid
/// this by acquiring a
/// [CriticalSection][critical_section::CriticalSection] while the pin
/// is in use.
///
/// This pin can temporarily change modes. This acquires a
/// [CriticalSection][critical_section::CriticalSection] which ensures
/// no other code running at the same time sees this pin with the
/// wrong mode.
///
/// It is possible to use two copies of this pin to change its mode
/// *twice*. This will probably make the pin useless, or at least
/// misbehave, but the typestate will always *eventually* be correct
/// again. Nonetheless, try to avoid doing this. It's not unsafe, just
/// a bad idea.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SharedPin<P> {
    pin: P,
}

// a note on safety:
//
// reads from many places are already fine. writes from many places
// are guarded by a critical section, and so are fine (though they may
// behave confusingly). Changing modes is *not* fine, but holds a
// critical section for the duration, and so returns to normal before
// anything else can touch it.
//
// it is possible to grab multiple critical sections and then use two
// copies of a shared pin to change modes. This will make the pin
// malfunction, but due to the nested nature of the mode changes,
// it will, eventually, match the type state again.
//
// that is: don't do it, but it's fine

impl<P> SharedPin<P> {
    /// Create a shared pin.
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    /// Free the shared pin, recovering the base pin.
    ///
    /// # Safety
    /// You must ensure no other copy of this shared pin exists.
    pub unsafe fn free(self) -> P {
        self.pin
    }

    /// Do something with the pin inside a critical section.
    pub fn with<R>(&mut self, f: impl FnOnce(&mut P) -> R) -> R {
        critical_section::with(|_cs| f(&mut self.pin))
    }
}

impl<P> SharedPin<P>
where
    P: PinInfo,
{
    /// Get the pin number of this pin.
    pub fn pin(&self) -> u8 {
        self.pin.pin()
    }

    /// Get the port of this pin.
    pub fn port(&self) -> char {
        self.pin.port()
    }
}

impl<P> SharedPin<P>
where
    P: WithMode,
{
    /// Temporarily configure this pin in a new mode.
    ///
    /// If this is an output mode, the initial state is retained if
    /// the original mode was also an output mode. It is otherwise
    /// undefined.
    fn with_mode<Mode, R>(&mut self, f: impl FnOnce(&mut P::With<Mode>) -> R) -> R
    where
        Mode: PinMode,
    {
        self.with(|p| p.with_mode(f))
    }

    /// Temporarily configure this pin in a new mode, in the given
    /// initial state.
    fn with_mode_in_state<Mode, R>(
        &mut self,
        state: PinState,
        f: impl FnOnce(&mut P::With<Output<Mode>>) -> R,
    ) -> R
    where
        Output<Mode>: PinMode,
    {
        self.with(|p| p.with_mode_in_state(state, f))
    }

    super::mode::with_mode_aliases!(vis pub, (P::With), ());
}

impl<P> SharedPin<P>
where
    P: hal02::InputPin<Error = Infallible>,
{
    /// Read the input pin.
    pub fn read(&self) -> PinState {
        if self.pin.is_high().unwrap_or_else(|e| match e {}) {
            PinState::High
        } else {
            PinState::Low
        }
    }

    /// Is the input pin high?
    pub fn is_high(&self) -> bool {
        self.pin.is_high().unwrap_or_else(|e| match e {})
    }

    /// Is the input pin low?
    pub fn is_low(&self) -> bool {
        self.pin.is_low().unwrap_or_else(|e| match e {})
    }
}

impl<P> SharedPin<P>
where
    P: hal02::OutputPin<Error = Infallible>,
{
    /// Set the current output drive state.
    pub fn set_state(&mut self, state: PinState) {
        self.pin
            .set_state(state.into())
            .unwrap_or_else(|e| match e {})
    }

    /// Set the current output high.
    pub fn set_high(&mut self) {
        self.pin.set_high().unwrap_or_else(|e| match e {})
    }

    /// Set the current output low.
    pub fn set_low(&mut self) {
        self.pin.set_low().unwrap_or_else(|e| match e {})
    }
}

impl<P> SharedPin<P>
where
    P: hal02::StatefulOutputPin<Error = Infallible>,
{
    /// Get the current output drive state.
    pub fn get_state(&self) -> PinState {
        if self.pin.is_set_high().unwrap_or_else(|e| match e {}) {
            PinState::High
        } else {
            PinState::Low
        }
    }

    /// Is the output set high?
    pub fn is_set_high(&self) -> bool {
        self.pin.is_set_high().unwrap_or_else(|e| match e {})
    }

    /// Is the output set low?
    pub fn is_set_low(&self) -> bool {
        self.pin.is_set_low().unwrap_or_else(|e| match e {})
    }
}

impl<P> SharedPin<P>
where
    P: hal02::ToggleableOutputPin<Error = Infallible>,
{
    /// Toggle the output.
    pub fn toggle(&mut self) {
        self.pin.toggle().unwrap_or_else(|e| match e {})
    }
}

impl<const P: char, const N: u8, Mode> Clone for SharedPin<Pin<P, N, Mode>>
where
    Mode: PinMode,
{
    fn clone(&self) -> Self {
        unsafe {
            // safety: reads, writes, and mode changes are guarded
            // with a critical section
            Self { pin: Pin::steal() }
        }
    }
}

impl<const P: char, const N: u8, Mode> From<Pin<P, N, Mode>> for SharedPin<Pin<P, N, Mode>>
where
    Mode: PinMode,
{
    fn from(other: Pin<P, N, Mode>) -> Self {
        Self { pin: other }
    }
}

impl<const P: char, Mode> Clone for SharedPin<PartiallyErasedPin<P, Mode>>
where
    Mode: PinMode,
{
    fn clone(&self) -> Self {
        unsafe {
            // safety: reads, writes, and mode changes are guarded
            // with a critical section
            Self {
                pin: PartiallyErasedPin::steal(self.pin.pin()),
            }
        }
    }
}

impl<const P: char, Mode> From<PartiallyErasedPin<P, Mode>>
    for SharedPin<PartiallyErasedPin<P, Mode>>
where
    Mode: PinMode,
{
    fn from(other: PartiallyErasedPin<P, Mode>) -> Self {
        Self { pin: other }
    }
}

impl<Mode> Clone for SharedPin<ErasedPin<Mode>>
where
    Mode: PinMode,
{
    fn clone(&self) -> Self {
        unsafe {
            // safety: reads, writes, and mode changes are guarded
            // with a critical section
            Self {
                pin: ErasedPin::steal(self.pin.pin(), self.pin.port()),
            }
        }
    }
}

impl<Mode> From<ErasedPin<Mode>> for SharedPin<ErasedPin<Mode>>
where
    Mode: PinMode,
{
    fn from(other: ErasedPin<Mode>) -> Self {
        Self { pin: other }
    }
}

impl<P> PinInfo for SharedPin<P>
where
    P: PinInfo,
{
    type Mode = P::Mode;

    fn pin(&self) -> u8 {
        SharedPin::pin(self)
    }

    fn port(&self) -> char {
        SharedPin::port(self)
    }
}

impl<P> WithMode for SharedPin<P>
where
    P: WithMode,
{
    type With<Mode> = P::With<Mode>;

    fn with_mode<Mode, R>(&mut self, f: impl FnOnce(&mut Self::With<Mode>) -> R) -> R
    where
        Mode: PinMode,
    {
        SharedPin::with_mode(self, f)
    }

    fn with_mode_in_state<Mode, R>(
        &mut self,
        state: PinState,
        f: impl FnOnce(&mut Self::With<Output<Mode>>) -> R,
    ) -> R
    where
        Output<Mode>: PinMode,
    {
        SharedPin::with_mode_in_state(self, state, f)
    }
}
