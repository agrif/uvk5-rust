//! Interfaces for interacting with GPIO pins.

use crate::pac;

mod hal02;
mod hal1;

mod mode;
pub use mode::*;

mod pin;
pub use pin::*;

/// Wrap the GPIO registers into ports.
#[inline(always)]
pub fn new(portcon: pac::PORTCON, a: pac::GPIOA, b: pac::GPIOB, c: pac::GPIOC) -> Ports {
    Ports::new(portcon, a, b, c)
}

/// Contains the GPIO peripherals as wrapped ports.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ports {
    pub port_a: port_a::Port,
    pub port_b: port_b::Port,
    pub port_c: port_c::Port,
}

impl Ports {
    /// Safety: This accesses PORTCON and all GPIO registers.
    #[inline(always)]
    unsafe fn steal() -> Self {
        Self {
            port_a: port_a::Port::steal(),
            port_b: port_b::Port::steal(),
            port_c: port_c::Port::steal(),
        }
    }

    /// Wrap the GPIO registers into ports.
    #[inline(always)]
    pub fn new(_portcon: pac::PORTCON, _a: pac::GPIOA, _b: pac::GPIOB, _c: pac::GPIOC) -> Self {
        // safety: we own the unique tokens that let us control these registers
        unsafe { Self::steal() }
    }

    /// Recover the raw GPIO registers from the wrapped ports.
    #[inline(always)]
    pub fn free(self) -> (pac::PORTCON, pac::GPIOA, pac::GPIOB, pac::GPIOC) {
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
}

// macro for each port module
macro_rules! port_mod {
    ($reg:ident, $name:literal, $P:literal, $p:ident, $bigp:ident, {$($N:literal),+}) => {
        paste::paste! {
            #[doc = concat!("Helper types for ", $name, ".")]
            pub mod [<port_ $p>] {
                use super::{Pin, Unspecified};
                use crate::power::Gate;
                use crate::pac::$reg;

                #[doc = concat!("Pins for ", $name, ".")]
                #[derive(Debug)]
                #[cfg_attr(feature = "defmt", derive(defmt::Format))]
                pub struct Pins {
                    $(pub [<$p $N>]: Pin<$P, $N, Unspecified>),+
                }

                impl Pins {
                    // safety: must be the only thing to write to this
                    // port in SYSCON and GPIO
                    #[inline(always)]
                    unsafe fn steal() -> Self {
                        Self {
                            $([<$p $N>]: Pin::steal()),+
                        }
                    }

                    /// Disable this port and regain its original components.
                    #[inline(always)]
                    pub fn disable(self) -> (Port, Gate<$reg>) {
                        // safety: we have all the pins here together,
                        // we join them back up and turn off the gate
                        unsafe {
                            let mut gate = Gate::steal();
                            gate.disable();
                            (Port::steal(), gate)
                        }
                    }
                }

                #[doc = concat!("Port for ", $name, ".")]
                pub struct Port {
                    _private: (),
                }

                impl core::fmt::Debug for Port {
                    #[allow(clippy::missing_inline_in_public_items)]
                    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        f.debug_tuple("Port").field(&stringify!($reg)).finish()
                    }
                }

                #[cfg(feature = "defmt")]
                impl defmt::Format for Port {
                    #[allow(clippy::missing_inline_in_public_items)]
                    fn format(&self, f: defmt::Formatter) {
                        defmt::write!(f, "Port({})", stringify!($reg));
                    }
                }

                impl Port {
                    // safety: this provides access to the pins for this
                    // GPIO in PORTCON and its own register
                    #[inline(always)]
                    pub(super) unsafe fn steal() -> Self {
                        Self { _private: () }
                    }

                    /// Enable this port and get access to its pins.
                    #[inline(always)]
                    pub fn enable(self, mut gate: Gate<$reg>) -> Pins {
                        gate.enable();

                        // safety: we've enabled this port and control the gate
                        unsafe {
                            Pins::steal()
                        }
                    }
                }

                $(
                    // PA0 etc. aliases
                    #[doc = concat!($name, " pin ", stringify!($N), ".")]
                    pub type [<P $bigp $N>]<Mode> = Pin<$P, $N, Mode>;
                )*
            }

            $(
                // re-export the PA0 etc. aliases
                #[doc(inline)]
                pub use [<port_ $p>]::[<P $bigp $N>];
            )*
        }
    };
}

port_mod!(GPIOA, "GPIO port A", 'A', a, A, {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15});
port_mod!(GPIOB, "GPIO port B", 'B', b, B, {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15});
port_mod!(GPIOC, "GPIO port C", 'C', c, C, {0, 1, 2, 3, 4, 5, 6, 7});

// alternate function aliases
macro_rules! alternate_aliases {
    {$($n:literal),*} => {
        paste::paste! {
            $(
                #[doc = concat!("Alternate function ", stringify!($n), ". (type state)")]
                pub type [<AF $n>]<Mode> = Alternate<$n, Mode>;
            )*
        }
    }
}

alternate_aliases! {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15}
