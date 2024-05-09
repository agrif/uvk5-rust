use core::convert::Infallible;
use embedded_hal_02::digital::v2 as hal02;

use super::{Input, Output, Pin, PinMode, PinState};

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