//! The backlight, turn it on or off.
// FIXME pwm?

use crate::hal::gpio::{Output, PinMode, PushPull, PB6};

pub type Pin<Mode> = PB6<Mode>;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Backlight {
    pin: Pin<Output<PushPull>>,
}

/// Set up the backlight for use.
pub fn new<Mode>(pin: Pin<Mode>) -> Backlight
where
    Mode: PinMode,
{
    Backlight::new(pin)
}

impl Backlight {
    /// Set up the backlight for use.
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

    /// Turn the backlight on.
    pub fn on(&mut self) {
        self.pin.set_high();
    }

    /// Turn the backlight off.
    pub fn off(&mut self) {
        self.pin.set_low();
    }

    /// Turn the backlight state.
    pub fn set(&mut self, on: bool) {
        self.pin.set_state(on.into());
    }

    /// Toggle the backlight on or off.
    pub fn toggle(&mut self) {
        self.pin.toggle();
    }

    /// Is the backlight on?
    pub fn is_on(&self) -> bool {
        self.pin.is_set_high()
    }
}
