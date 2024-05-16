use core::cell::Cell;

use super::{IntoMode, PinInfo, PinMode, PinState};

/// A pin that dynamically changes between input and output as needed.
pub struct InputOutputPin<Input, Output, const STATE: bool = false> {
    pin: core::cell::Cell<InOut<Input, Output>>,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum InOut<Input, Output> {
    Input(Input),
    Output(Output),
    None,
}

impl<Input, Output, const STATE: bool> core::fmt::Debug for InputOutputPin<Input, Output, STATE>
where
    Input: core::fmt::Debug,
    Output: core::fmt::Debug,
{
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let pin = self.pin.replace(InOut::None);
        let r = f
            .debug_tuple("InputOutputPin")
            .field(&pin)
            .field(&STATE)
            .finish();
        self.pin.set(pin);
        r
    }
}

#[cfg(feature = "defmt")]
impl<Input, Output, const STATE: bool> defmt::Format for InputOutputPin<Input, Output, STATE>
where
    Input: defmt::Format,
    Output: defmt::Format,
{
    #[allow(clippy::missing_inline_in_public_items)]
    fn format(&self, f: defmt::Formatter) {
        let pin = self.pin.replace(InOut::None);
        defmt::write!(f, "InputOutputPin({}, {})", pin, STATE);
        self.pin.set(pin);
    }
}

impl<Input, Output, I, O> InputOutputPin<Input, Output>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
{
    /// Create a new input/output pin from the given input and a
    /// function to convert into an output.
    ///
    /// This function is not used, except to infer the type of the
    /// output. [IntoMode] is used to change modes.
    #[inline(always)]
    pub fn new_from_input(pin: Input, _to_output: impl Fn(Input) -> Output) -> Self {
        Self {
            pin: Cell::new(InOut::Input(pin)),
        }
    }

    /// Create a new input/output pin from the given output and a
    /// function to convert into an input.
    ///
    /// This function is not used, except to infer the type of the
    /// output. [IntoMode] is used to change modes.
    #[inline(always)]
    pub fn new_from_output(pin: Output, _to_input: impl Fn(Output) -> Input) -> Self {
        Self {
            pin: Cell::new(InOut::Output(pin)),
        }
    }
}

impl<Input, Output, I, O, const STATE: bool> InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
{
    #[inline(always)]
    fn take_input(&self) -> Input {
        match self.pin.replace(InOut::None) {
            InOut::Input(i) => i,
            InOut::Output(o) => o.into_mode::<Input::Mode>(),
            InOut::None => panic!(),
        }
    }

    #[inline(always)]
    fn take_output(&self, state: Option<PinState>) -> Output {
        match self.pin.replace(InOut::None) {
            InOut::Input(i) => i.into_mode_in_state(state.unwrap_or(STATE.into())),
            InOut::Output(o) => match state {
                Some(state) => o.into_mode_in_state(state),
                None => o,
            },
            InOut::None => panic!(),
        }
    }

    /// Recover the input pin.
    #[inline(always)]
    pub fn into_input(self) -> Input {
        self.take_input()
    }

    /// Recover the output pin.
    #[inline(always)]
    pub fn into_output(self) -> Output {
        self.take_output(None)
    }

    /// Recover the output pin in the given state.
    #[inline(always)]
    pub fn into_output_in_state(self, state: PinState) -> Output {
        self.take_output(Some(state))
    }

    /// Change the default state of the output pin.
    #[inline(always)]
    pub fn default_state<const NEWSTATE: bool>(self) -> InputOutputPin<Input, Output, NEWSTATE> {
        InputOutputPin { pin: self.pin }
    }

    /// Change the default state of the output pin to low.
    #[inline(always)]
    pub fn default_low(self) -> InputOutputPin<Input, Output, false> {
        self.default_state()
    }

    /// Change the default state of the output pin to high.
    #[inline(always)]
    pub fn default_high(self) -> InputOutputPin<Input, Output, true> {
        self.default_state()
    }

    /// Get a reference to this pin as an input.
    ///
    /// Panics if any other method is called on [Self] inside `f`.
    #[inline(always)]
    pub fn with_input<R>(&self, f: impl FnOnce(&mut Input) -> R) -> R {
        let mut i = self.take_input();
        let r = f(&mut i);
        self.pin.set(InOut::Input(i));
        r
    }

    /// Get a reference to this pin as an output.
    ///
    /// Panics if any other method is called on [Self] inside `f`.
    #[inline(always)]
    pub fn with_output<R>(&self, f: impl FnOnce(&mut Output) -> R) -> R {
        let mut o = self.take_output(None);
        let r = f(&mut o);
        self.pin.set(InOut::Output(o));
        r
    }

    /// Get a reference to this pin as an output in the given state
    ///
    /// Panics if any other method is called on [Self] inside `f`.
    #[inline(always)]
    pub fn with_output_in_state<R>(&self, state: PinState, f: impl FnOnce(&mut Output) -> R) -> R {
        let mut o = self.take_output(Some(state));
        let r = f(&mut o);
        self.pin.set(InOut::Output(o));
        r
    }
}
