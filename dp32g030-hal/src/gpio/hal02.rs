use core::convert::Infallible;
use embedded_hal_02::digital::v2 as hal02;

use super::{
    ErasedPin, Input, InputOutputPin, IntoMode, Output, PartiallyErasedPin, Pin, PinInfo, PinMode,
    PinState,
};

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

impl<const P: char, Pull, OMode, Mode>
    hal02::IoPin<PartiallyErasedPin<P, Input<Pull>>, PartiallyErasedPin<P, Output<OMode>>>
    for PartiallyErasedPin<P, Mode>
where
    Input<Pull>: PinMode,
    Output<OMode>: PinMode,
    Mode: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn into_input_pin(self) -> Result<PartiallyErasedPin<P, Input<Pull>>, Self::Error> {
        Ok(self.into_mode())
    }

    #[inline(always)]
    fn into_output_pin(
        mut self,
        state: hal02::PinState,
    ) -> Result<PartiallyErasedPin<P, Output<OMode>>, Self::Error> {
        self.write_data(state.into());
        Ok(self.into_mode())
    }
}

impl<const P: char, Pull> hal02::InputPin for PartiallyErasedPin<P, Input<Pull>>
where
    Input<Pull>: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(PartiallyErasedPin::is_high(self))
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(PartiallyErasedPin::is_low(self))
    }
}

impl<const P: char, Mode> hal02::OutputPin for PartiallyErasedPin<P, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        PartiallyErasedPin::set_low(self);
        Ok(())
    }

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        PartiallyErasedPin::set_high(self);
        Ok(())
    }

    #[inline(always)]
    fn set_state(&mut self, state: hal02::PinState) -> Result<(), Self::Error> {
        PartiallyErasedPin::set_state(self, state.into());
        Ok(())
    }
}

impl<const P: char, Mode> hal02::StatefulOutputPin for PartiallyErasedPin<P, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    #[inline(always)]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(PartiallyErasedPin::is_set_high(self))
    }

    #[inline(always)]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(PartiallyErasedPin::is_set_low(self))
    }
}

impl<const P: char, Mode> hal02::ToggleableOutputPin for PartiallyErasedPin<P, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        PartiallyErasedPin::toggle(self);
        Ok(())
    }
}

impl<Pull, OMode, Mode> hal02::IoPin<ErasedPin<Input<Pull>>, ErasedPin<Output<OMode>>>
    for ErasedPin<Mode>
where
    Input<Pull>: PinMode,
    Output<OMode>: PinMode,
    Mode: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn into_input_pin(self) -> Result<ErasedPin<Input<Pull>>, Self::Error> {
        Ok(self.into_mode())
    }

    #[inline(always)]
    fn into_output_pin(
        mut self,
        state: hal02::PinState,
    ) -> Result<ErasedPin<Output<OMode>>, Self::Error> {
        self.write_data(state.into());
        Ok(self.into_mode())
    }
}

impl<Pull> hal02::InputPin for ErasedPin<Input<Pull>>
where
    Input<Pull>: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(ErasedPin::is_high(self))
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(ErasedPin::is_low(self))
    }
}

impl<Mode> hal02::OutputPin for ErasedPin<Output<Mode>>
where
    Output<Mode>: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        ErasedPin::set_low(self);
        Ok(())
    }

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        ErasedPin::set_high(self);
        Ok(())
    }

    #[inline(always)]
    fn set_state(&mut self, state: hal02::PinState) -> Result<(), Self::Error> {
        ErasedPin::set_state(self, state.into());
        Ok(())
    }
}

impl<Mode> hal02::StatefulOutputPin for ErasedPin<Output<Mode>>
where
    Output<Mode>: PinMode,
{
    #[inline(always)]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(ErasedPin::is_set_high(self))
    }

    #[inline(always)]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(ErasedPin::is_set_low(self))
    }
}

impl<Mode> hal02::ToggleableOutputPin for ErasedPin<Output<Mode>>
where
    Output<Mode>: PinMode,
{
    type Error = Infallible;

    #[inline(always)]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        ErasedPin::toggle(self);
        Ok(())
    }
}

impl<Input, Output, I, O, const STATE: bool>
    hal02::IoPin<InputOutputPin<Input, Output, STATE>, InputOutputPin<Input, Output, STATE>>
    for InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
    Input: hal02::InputPin<Error = Infallible>,
    Output: hal02::OutputPin<Error = Infallible>,
{
    type Error = Infallible;

    #[inline(always)]
    fn into_input_pin(self) -> Result<Self, Self::Error> {
        self.with_input(|_p| ());
        Ok(self)
    }

    #[inline(always)]
    fn into_output_pin(self, state: hal02::PinState) -> Result<Self, Self::Error> {
        self.with_output_in_state(state.into(), |_p| ());
        Ok(self)
    }
}

impl<Input, Output, I, O, const STATE: bool> hal02::InputPin
    for InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
    Input: hal02::InputPin<Error = Infallible>,
{
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.with_input(|p| p.is_high())
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        self.with_input(|p| p.is_low())
    }
}

impl<Input, Output, I, O, const STATE: bool> hal02::OutputPin
    for InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
    Output: hal02::OutputPin<Error = Infallible>,
{
    type Error = Infallible;

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.with_output(|p| p.set_low())
    }

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.with_output(|p| p.set_high())
    }

    #[inline(always)]
    fn set_state(&mut self, state: hal02::PinState) -> Result<(), Self::Error> {
        self.with_output(|p| p.set_state(state))
    }
}

impl<Input, Output, I, O, const STATE: bool> hal02::StatefulOutputPin
    for InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
    Output: hal02::StatefulOutputPin<Error = Infallible>,
{
    #[inline(always)]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        self.with_output(|p| p.is_set_high())
    }

    #[inline(always)]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        self.with_output(|p| p.is_set_low())
    }
}

impl<Input, Output, I, O, const STATE: bool> hal02::ToggleableOutputPin
    for InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
    Output: hal02::ToggleableOutputPin<Error = Infallible>,
{
    type Error = Infallible;

    #[inline(always)]
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.with_output(|p| p.toggle())
    }
}
