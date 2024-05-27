//! The flashlight, turn it on or off.

use crate::hal::gpio::{Output, PinMode, PushPull, PC3};

pub type Pin<Mode> = PC3<Mode>;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Flashlight {
    pin: Pin<Output<PushPull>>,
}

/// Set up the flashlight for use.
pub fn new<Mode>(pin: Pin<Mode>) -> Flashlight
where
    Mode: PinMode,
{
    Flashlight::new(pin)
}

impl Flashlight {
    /// Set up the flashlight for use.
    pub fn new<Mode>(pin: Pin<Mode>) -> Self
    where
        Mode: PinMode,
    {
        Self {
            pin: pin.into_mode(),
        }
    }

    /// Free the flaslight pin for use elsewhere.
    pub fn free(self) -> Pin<Output<PushPull>> {
        self.pin
    }

    /// Turn the flashlight on.
    pub fn on(&mut self) {
        self.pin.set_high();
    }

    /// Turn the flashlight off.
    pub fn off(&mut self) {
        self.pin.set_low();
    }

    /// Turn the flashlight state.
    pub fn set(&mut self, on: bool) {
        self.pin.set_state(on.into());
    }

    /// Toggle the flashlight on or off.
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Is the flashlight on?
    pub fn is_on(&self) -> bool {
        self.pin.is_set_high()
    }
}
