//! Push-to-talk button.
// FIXME debounce?

use crate::hal::gpio::{Input, PinMode, PullUp, PC5};

pub type Pin<Mode> = PC5<Mode>;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ptt {
    pin: Pin<Input<PullUp>>,
}

/// Set up the PTT button for use.
pub fn new<Mode>(pin: Pin<Mode>) -> Ptt
where
    Mode: PinMode,
{
    Ptt::new(pin)
}

impl Ptt {
    /// Set up the PTT button for use.
    pub fn new<Mode>(pin: Pin<Mode>) -> Self
    where
        Mode: PinMode,
    {
        Self {
            pin: pin.into_mode(),
        }
    }

    /// Free the PTT pin for use elsewhere.
    pub fn free(self) -> Pin<Input<PullUp>> {
        self.pin
    }

    /// Is the PTT button pressed?
    pub fn is_pressed(&self) -> bool {
        self.pin.is_low()
    }
}
