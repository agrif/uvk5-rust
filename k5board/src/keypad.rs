//! Read the keypad and other buttons.

use crate::bitflags;
use crate::hal::gpio::{
    Floating, Input, OpenDrain, Output, PullUp, PushPull, PA10, PA11, PA12, PA13, PA3, PA4, PA5,
    PA6, PC5,
};
use crate::hal::time::TimerDuration;

/// The pins required for the keypad.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Parts {
    /// The pin connected to the PTT button.
    pub ptt: PC5<Input<PullUp>>,

    /// The pins connected to the keypad rows.
    #[allow(clippy::type_complexity)]
    pub row: (
        PA3<Input<Floating>>,
        PA4<Input<Floating>>,
        PA5<Input<Floating>>,
        PA6<Input<Floating>>,
    ),

    /// The pins connected to the keypad columns.
    #[allow(clippy::type_complexity)]
    pub col: (
        // shared with i2c
        PA10<Output<OpenDrain>>,
        PA11<Output<OpenDrain>>,
        // shared with voice ic
        // FIXME we could detect more presses at once if this was open-drain
        PA12<Output<PushPull>>,
        PA13<Output<PushPull>>,
    ),
}

bitflags! {
    /// State of the keypad.
    #[repr(transparent)]
    #[derive(Default)]
    pub struct State: u32 {
        // in order, PA3 .. PA6

        // PA10 active
        const MENU = 1 << 0;
        const DTMF_A = Self::MENU.bits;
        const N1 = 1 << 1;
        const N4 = 1 << 2;
        const N7 = 1 << 3;

        // PA11 active
        const UP = 1 << 4;
        const DTMF_B = Self::UP.bits;
        const N2 = 1 << 5;
        const N5 = 1 << 6;
        const N8 = 1 << 7;

        // PA12 active
        const DOWN = 1 << 8;
        const DTMF_C = Self::DOWN.bits;
        const N3 = 1 << 9;
        const N6 = 1 << 10;
        const N9 = 1 << 11;

        // PA13 active
        const EXIT = 1 << 12;
        const DTMF_D = Self::EXIT.bits;
        const STAR = 1 << 13;
        const N0 = 1 << 14;
        const FUNCTION = 1 << 15;

        // sporadics, none active
        const SIDE1 = 1 << 16;
        const SIDE2 = 1 << 17;
        // nothing on PA5
        // nothing on PA6

        // PC5, Extremely Sporadic
        const PTT = 1 << 20;
    }
}

// useful methods, but way too much duplication, so use a macro
macro_rules! helper {
    ($doc:expr, $name:ident, $flag:ident) => {
        #[doc = concat!("Is the ", $doc, " button pressed?")]
        pub fn $name(&self) -> bool {
            self.contains(Self::$flag)
        }
    };
}

/// Helper methods for testing buttons.
impl State {
    helper!("push-to-talk", is_ptt, PTT);
    helper!("side button I", is_side_1, SIDE1);
    helper!("side button II", is_side_2, SIDE2);

    helper!("menu", is_menu, MENU);
    helper!("up", is_up, UP);
    helper!("down", is_down, DOWN);
    helper!("exit", is_exit, EXIT);

    helper!("DTMF A", is_dtmf_a, DTMF_A);
    helper!("DTMF B", is_dtmf_b, DTMF_B);
    helper!("DTMF C", is_dtmf_c, DTMF_C);
    helper!("DTMF D", is_dtmf_d, DTMF_D);

    helper!("*", is_star, STAR);
    helper!("function", is_function, FUNCTION);

    helper!("number 0", is_0, N0);
    helper!("number 1", is_1, N1);
    helper!("number 2", is_2, N2);
    helper!("number 3", is_3, N3);
    helper!("number 4", is_4, N4);
    helper!("number 5", is_5, N5);
    helper!("number 6", is_6, N6);
    helper!("number 7", is_7, N7);
    helper!("number 8", is_8, N8);
    helper!("number 9", is_9, N9);

    /// An array of all number flags.
    pub const fn all_numbers() -> &'static [Self; 10] {
        const ALL_NUMBERS: [State; 10] = [
            State::N0,
            State::N1,
            State::N2,
            State::N3,
            State::N4,
            State::N5,
            State::N6,
            State::N7,
            State::N8,
            State::N9,
        ];
        &ALL_NUMBERS
    }

    /// Get the value of the pressed number button, if one is pressed.
    ///
    /// This will arbitrarily choose the lowest number if multiple
    /// number buttons are pressed.
    pub fn number(&self) -> Option<u8> {
        let number_mask = Self::all_numbers().iter().copied().collect();
        let numbers = self.intersection(number_mask);
        if numbers.is_empty() {
            return None;
        }

        for (n, f) in Self::all_numbers().iter().enumerate() {
            if self.contains(*f) {
                return Some(n as u8);
            }
        }

        None
    }
}

/// The keypad interface.
///
/// Using powers of two for DEBOUNCE can avoid a modulus operation.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Keypad<const DEBOUNCE: usize = { 1 << 3 }> {
    pins: Parts,
    history: [State; DEBOUNCE],
    next: usize,
    state: State,
    up: State,
    down: State,
}

/// Create the keypad interface from the keypad pins.
pub fn new(parts: Parts) -> Keypad {
    Keypad::new(parts)
}

impl<const DEBOUNCE: usize> Keypad<DEBOUNCE> {
    /// Create the keypad interface from the keypad pins.
    pub fn new(parts: Parts) -> Self {
        Self {
            pins: parts,
            history: [State::empty(); DEBOUNCE],
            next: 0,
            state: State::empty(),
            up: State::empty(),
            down: State::empty(),
        }
    }

    /// Free the keypad interface and recover the pins.
    pub fn free(self) -> Parts {
        self.pins
    }

    // read a row, all at once, into bits
    fn read_row(&mut self, mask: u32) -> u32 {
        let bits = (self.pins.row.0.is_high() as u32)
            | ((self.pins.row.1.is_high() as u32) << 1)
            | ((self.pins.row.2.is_high() as u32) << 2)
            | ((self.pins.row.3.is_high() as u32) << 3);
        !bits & mask
    }

    /// Poll the keypad, returning any newly-pressed keys.
    pub fn poll(&mut self) -> State {
        let mut state = State::empty();

        // set all columns high
        self.pins.col.0.set_high();
        self.pins.col.1.set_high();
        self.pins.col.2.set_high();
        self.pins.col.3.set_high();

        // read sporadics
        self.pins.col.3.set_high();
        if self.pins.ptt.is_low() {
            state |= State::PTT;
        }

        // these buttons make entire rows always low, so, mask them out if set
        let sporadics = self.read_row(0b0011);
        state |= State::from_bits_truncate(sporadics << 16);
        let mask = 0b1111 & !sporadics;

        // column 0
        self.pins.col.0.set_low();
        state |= State::from_bits_truncate(self.read_row(mask));

        // column 1
        self.pins.col.0.set_high();
        self.pins.col.1.set_low();
        state |= State::from_bits_truncate(self.read_row(mask) << 4);

        // column 2
        self.pins.col.1.set_high();
        self.pins.col.2.set_low();
        state |= State::from_bits_truncate(self.read_row(mask) << 8);

        // column 3
        self.pins.col.2.set_high();
        self.pins.col.3.set_low();
        state |= State::from_bits_truncate(self.read_row(mask) << 12);

        // to prevent confusing our peripherals that share these pins

        // send i2c stop on i2c pins
        let scl = &mut self.pins.col.0;
        let sda = &mut self.pins.col.1;
        // conservative estimate of 1us in clock cycles
        let us = TimerDuration::<72_000_000>::micros(1).ticks();
        sda.set_low();
        cortex_m::asm::delay(us);
        scl.set_low();
        cortex_m::asm::delay(us);
        scl.set_high();
        cortex_m::asm::delay(us);
        sda.set_high();
        cortex_m::asm::delay(us);

        // reset voice ic pins
        let vclk = &mut self.pins.col.2;
        let vdata = &mut self.pins.col.3;
        vclk.set_low();
        vdata.set_high();

        // store our read state
        self.history[self.next] = state;
        self.next += 1;
        self.next %= DEBOUNCE;

        // calculate the debounced state
        let mut debounced_state = State::all();
        for (i, s) in self.history.iter().enumerate() {
            if i == self.next {
                // this is the oldest state. skip it -- we only use this
                // for detecting positive edges
                continue;
            }
            debounced_state &= *s;
        }

        self.down = !self.history[self.next] & debounced_state;
        self.up = self.state & !debounced_state;
        self.state = debounced_state;

        self.down
    }

    /// Get all keys currently pressed.
    pub fn pressed(&self) -> State {
        self.state
    }

    /// Get all keys newly pressed down on the last call to [Self::poll()].
    pub fn down(&self) -> State {
        self.down
    }

    /// Get all keys newly unpressed on the last call to [Self::poll()].
    pub fn up(&self) -> State {
        self.up
    }

    /// Get all keys which changed state in the last call to [Self::poll()].
    pub fn changed(&self) -> State {
        self.down | self.up
    }
}
