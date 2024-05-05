//! Interfaces for interacting with GPIO pins.

use crate::pac;

mod mode;
pub use mode::*;

mod pin;
pub use pin::*;

/// Contains the GPIO peripherals split into pins.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Pins {
    pub port_a: gpioa::Pins,
    pub port_b: gpiob::Pins,
    pub port_c: gpioc::Pins,
}

/// Split the GPIO peripherals into pins.
#[inline(always)]
pub fn split(_portcon: pac::PORTCON, _a: pac::GPIOA, _b: pac::GPIOB, _c: pac::GPIOC) -> Pins {
    // safety: we own the unique tokens that let us control these registers
    unsafe {
        Pins {
            port_a: gpioa::Pins::steal(),
            port_b: gpiob::Pins::steal(),
            port_c: gpioc::Pins::steal(),
        }
    }
}

/// Recover the raw GPIO registers from the split pins.
#[inline(always)]
pub fn recover(_pins: Pins) -> (pac::PORTCON, pac::GPIOA, pac::GPIOB, pac::GPIOC) {
    // safety: we have all of the pins, and destroy them here, so these
    // registers are safe to use again
    unsafe {
        (
            pac::PORTCON::steal(),
            pac::GPIOA::steal(),
            pac::GPIOB::steal(),
            pac::GPIOC::steal(),
        )
    }
}

/// Helper types for GPIO A.
pub mod gpioa {
    use super::{Pin, Unspecified};

    /// Pins for the GPIO A port.
    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Pins {
        pub a0: Pin<'A', 0, Unspecified>,
        pub a1: Pin<'A', 1, Unspecified>,
        pub a2: Pin<'A', 2, Unspecified>,
        pub a3: Pin<'A', 3, Unspecified>,

        pub a4: Pin<'A', 4, Unspecified>,
        pub a5: Pin<'A', 5, Unspecified>,
        pub a6: Pin<'A', 6, Unspecified>,
        pub a7: Pin<'A', 7, Unspecified>,

        pub a8: Pin<'A', 8, Unspecified>,
        pub a9: Pin<'A', 9, Unspecified>,
        pub a10: Pin<'A', 10, Unspecified>,
        pub a11: Pin<'A', 11, Unspecified>,

        pub a12: Pin<'A', 12, Unspecified>,
        pub a13: Pin<'A', 13, Unspecified>,
        pub a14: Pin<'A', 14, Unspecified>,
        pub a15: Pin<'A', 15, Unspecified>,
    }

    impl Pins {
        // safety: must be the only thing to write to this port in SYSCON and GPIO
        #[inline(always)]
        pub(crate) unsafe fn steal() -> Self {
            Self {
                a0: Pin::steal(),
                a1: Pin::steal(),
                a2: Pin::steal(),
                a3: Pin::steal(),

                a4: Pin::steal(),
                a5: Pin::steal(),
                a6: Pin::steal(),
                a7: Pin::steal(),

                a8: Pin::steal(),
                a9: Pin::steal(),
                a10: Pin::steal(),
                a11: Pin::steal(),

                a12: Pin::steal(),
                a13: Pin::steal(),
                a14: Pin::steal(),
                a15: Pin::steal(),
            }
        }
    }
}

/// Helper types for GPIO B.
pub mod gpiob {
    use super::{Pin, Unspecified};

    /// Pins for the GPIO B port.
    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Pins {
        pub b0: Pin<'B', 0, Unspecified>,
        pub b1: Pin<'B', 1, Unspecified>,
        pub b2: Pin<'B', 2, Unspecified>,
        pub b3: Pin<'B', 3, Unspecified>,

        pub b4: Pin<'B', 4, Unspecified>,
        pub b5: Pin<'B', 5, Unspecified>,
        pub b6: Pin<'B', 6, Unspecified>,
        pub b7: Pin<'B', 7, Unspecified>,

        pub b8: Pin<'B', 8, Unspecified>,
        pub b9: Pin<'B', 9, Unspecified>,
        pub b10: Pin<'B', 10, Unspecified>,
        pub b11: Pin<'B', 11, Unspecified>,

        pub b12: Pin<'B', 12, Unspecified>,
        pub b13: Pin<'B', 13, Unspecified>,
        pub b14: Pin<'B', 14, Unspecified>,
        pub b15: Pin<'B', 15, Unspecified>,
    }

    impl Pins {
        // safety: must be the only thing to write to this port in SYSCON and GPIO
        #[inline(always)]
        pub(crate) unsafe fn steal() -> Self {
            Self {
                b0: Pin::steal(),
                b1: Pin::steal(),
                b2: Pin::steal(),
                b3: Pin::steal(),

                b4: Pin::steal(),
                b5: Pin::steal(),
                b6: Pin::steal(),
                b7: Pin::steal(),

                b8: Pin::steal(),
                b9: Pin::steal(),
                b10: Pin::steal(),
                b11: Pin::steal(),

                b12: Pin::steal(),
                b13: Pin::steal(),
                b14: Pin::steal(),
                b15: Pin::steal(),
            }
        }
    }
}

/// Helper types for GPIO C.
pub mod gpioc {
    use super::{Pin, Unspecified};

    /// Pins for the GPIO C port.
    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct Pins {
        pub c0: Pin<'C', 0, Unspecified>,
        pub c1: Pin<'C', 1, Unspecified>,
        pub c2: Pin<'C', 2, Unspecified>,
        pub c3: Pin<'C', 3, Unspecified>,

        pub c4: Pin<'C', 4, Unspecified>,
        pub c5: Pin<'C', 5, Unspecified>,
        pub c6: Pin<'C', 6, Unspecified>,
        pub c7: Pin<'C', 7, Unspecified>,
    }

    impl Pins {
        // safety: must be the only thing to write to this port in SYSCON and GPIO
        #[inline(always)]
        pub(crate) unsafe fn steal() -> Self {
            Self {
                c0: Pin::steal(),
                c1: Pin::steal(),
                c2: Pin::steal(),
                c3: Pin::steal(),

                c4: Pin::steal(),
                c5: Pin::steal(),
                c6: Pin::steal(),
                c7: Pin::steal(),
            }
        }
    }
}
