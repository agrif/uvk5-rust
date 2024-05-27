//! Push-to-talk button.
// FIXME debounce?

use crate::hal::gpio::{Input, PullUp, PC5};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// The PTT button.
pub struct Ptt {
    pin: PC5<Input<PullUp>>,
}

/// Set up the PTT button for use.
pub fn new(pin: PC5<Input<PullUp>>) -> Ptt {
    Ptt::new(pin)
}

impl Ptt {
    /// Set up the PTT button for use.
    pub fn new(pin: PC5<Input<PullUp>>) -> Self {
        Self { pin }
    }

    /// Free the PTT pin for use elsewhere.
    pub fn free(self) -> PC5<Input<PullUp>> {
        self.pin
    }

    /// Is the PTT button pressed?
    pub fn is_pressed(&self) -> bool {
        self.pin.is_low()
    }
}
