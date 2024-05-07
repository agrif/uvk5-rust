use core::convert::Infallible;
use embedded_hal_1::digital as hal1;

use super::{Input, Output, Pin, PinMode, PinState};

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
