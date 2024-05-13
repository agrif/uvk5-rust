//! Types for using pins in alternate modes.

use super::*;

use crate::pac::portcon::porta_sel0::{PORTA1_A, PORTA2_A, PORTA3_A, PORTA4_A};

macro_rules! pin {
    ($name:literal, enum $pinname:ident {$($var:ident<$mode:ty>),*$(,)?}) => {
        #[derive(Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        #[doc = concat!("Choices for pin ", stringify!($pinname), " on ", $name, ".")]
        pub enum $pinname {
            $(
                $var($var<$mode>),
            )*
        }

        $(
            impl<Mode> From<$var<Mode>> for $pinname where Mode: PinMode {
                #[inline(always)]
                fn from(value: $var<Mode>) -> Self {
                    Self::$var(value.into_mode())
                }
            }
        )*
    };
}

macro_rules! pins {
    ($mod:ident, $name:literal, {$(enum $pinname:ident {$($var:ident<$mode:ty>),*$(,)?})*}) => {
        #[doc = concat!($name, ".")]
        pub mod $mod {
            use super::*;

            $(
                pin!($name, enum $pinname {
                    $(
                        $var<$mode>,
                    )*
                });
            )*
        }
    };
}

// total guesses on pin modes
pins!(xtah, "XTAH port", {
    enum Xi {
        PA3<Alternate<{PORTA3_A::XtahXi as u8}, Input<Floating>>>,
    }

    enum Xo {
        PA4<Alternate<{PORTA4_A::XtahXo as u8}, Output<PushPull>>>,
    }
});

// also total guesses on pin modes
pins!(xtal, "XTAL port", {
    enum Xi {
        PA1<Alternate<{PORTA1_A::XtalXi as u8}, Input<Floating>>>,
    }

    enum Xo {
        PA2<Alternate<{PORTA2_A::XtalXo as u8}, Output<PushPull>>>,
    }
});
