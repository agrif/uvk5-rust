use core::convert::Infallible;
use embedded_hal_1::digital as hal1;

use super::{
    ErasedPin, Input, InputOutputPin, IntoMode, Output, PartiallyErasedPin, Pin, PinInfo, PinMode,
    PinState,
};

impl From<hal1::PinState> for PinState {
    fn from(value: hal1::PinState) -> Self {
        match value {
            hal1::PinState::Low => Self::Low,
            hal1::PinState::High => Self::High,
        }
    }
}

impl From<PinState> for hal1::PinState {
    fn from(value: PinState) -> Self {
        match value {
            PinState::Low => Self::Low,
            PinState::High => Self::High,
        }
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
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::is_high(self))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::is_low(self))
    }
}

impl<const P: char, const N: u8, Mode> hal1::OutputPin for Pin<P, N, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Pin::set_low(self);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Pin::set_high(self);
        Ok(())
    }

    fn set_state(&mut self, state: hal1::PinState) -> Result<(), Self::Error> {
        Pin::set_state(self, state.into());
        Ok(())
    }
}

impl<const P: char, const N: u8, Mode> hal1::StatefulOutputPin for Pin<P, N, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::is_set_high(self))
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::is_set_low(self))
    }

    fn toggle(&mut self) -> Result<(), Self::Error> {
        Pin::toggle(self);
        Ok(())
    }
}

impl<const P: char, Mode> hal1::ErrorType for PartiallyErasedPin<P, Mode>
where
    Mode: PinMode,
{
    type Error = Infallible;
}

impl<const P: char, Pull> hal1::InputPin for PartiallyErasedPin<P, Input<Pull>>
where
    Input<Pull>: PinMode,
{
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(PartiallyErasedPin::is_high(self))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(PartiallyErasedPin::is_low(self))
    }
}

impl<const P: char, Mode> hal1::OutputPin for PartiallyErasedPin<P, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        PartiallyErasedPin::set_low(self);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        PartiallyErasedPin::set_high(self);
        Ok(())
    }

    fn set_state(&mut self, state: hal1::PinState) -> Result<(), Self::Error> {
        PartiallyErasedPin::set_state(self, state.into());
        Ok(())
    }
}

impl<const P: char, Mode> hal1::StatefulOutputPin for PartiallyErasedPin<P, Output<Mode>>
where
    Output<Mode>: PinMode,
{
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(PartiallyErasedPin::is_set_high(self))
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(PartiallyErasedPin::is_set_low(self))
    }

    fn toggle(&mut self) -> Result<(), Self::Error> {
        PartiallyErasedPin::toggle(self);
        Ok(())
    }
}

impl<Mode> hal1::ErrorType for ErasedPin<Mode>
where
    Mode: PinMode,
{
    type Error = Infallible;
}

impl<Pull> hal1::InputPin for ErasedPin<Input<Pull>>
where
    Input<Pull>: PinMode,
{
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(ErasedPin::is_high(self))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(ErasedPin::is_low(self))
    }
}

impl<Mode> hal1::OutputPin for ErasedPin<Output<Mode>>
where
    Output<Mode>: PinMode,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        ErasedPin::set_low(self);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        ErasedPin::set_high(self);
        Ok(())
    }

    fn set_state(&mut self, state: hal1::PinState) -> Result<(), Self::Error> {
        ErasedPin::set_state(self, state.into());
        Ok(())
    }
}

impl<Mode> hal1::StatefulOutputPin for ErasedPin<Output<Mode>>
where
    Output<Mode>: PinMode,
{
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(ErasedPin::is_set_high(self))
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(ErasedPin::is_set_low(self))
    }

    fn toggle(&mut self) -> Result<(), Self::Error> {
        ErasedPin::toggle(self);
        Ok(())
    }
}

impl<Input, Output, I, O, const STATE: bool> hal1::ErrorType
    for InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
{
    type Error = Infallible;
}

impl<Input, Output, I, O, const STATE: bool> hal1::InputPin for InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
    Input: hal1::InputPin<Error = Infallible>,
{
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.with_input(|p| p.is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.with_input(|p| p.is_low())
    }
}

impl<Input, Output, I, O, const STATE: bool> hal1::OutputPin
    for InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
    Output: hal1::OutputPin<Error = Infallible>,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.with_output(|p| p.set_low())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.with_output(|p| p.set_high())
    }

    fn set_state(&mut self, state: hal1::PinState) -> Result<(), Self::Error> {
        self.with_output(|p| p.set_state(state))
    }
}

impl<Input, Output, I, O, const STATE: bool> hal1::StatefulOutputPin
    for InputOutputPin<Input, Output, STATE>
where
    Input: PinInfo<Mode = super::Input<I>>,
    Output: PinInfo<Mode = super::Output<O>>,
    Input: IntoMode<As<Output::Mode> = Output>,
    Output: IntoMode<As<Input::Mode> = Input>,
    Output: IntoMode<As<Output::Mode> = Output>,
    super::Input<I>: PinMode,
    super::Output<O>: PinMode,
    Output: hal1::StatefulOutputPin<Error = Infallible>,
{
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        self.with_output(|p| p.is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        self.with_output(|p| p.is_set_low())
    }

    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.with_output(|p| p.toggle())
    }
}
