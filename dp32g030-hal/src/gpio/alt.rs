//! Types for using pins in alternate modes.

use super::*;

use crate::pac::portcon::porta_sel0::*;
use crate::pac::portcon::porta_sel1::*;
use crate::pac::portcon::portb_sel0::*;
use crate::pac::portcon::portb_sel1::*;
use crate::pac::portcon::portc_sel0::*;

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
    ($mod:ident, $name:literal, {$(enum $pinname:ident : $varname:ident {$($var:ident<$mode:ty>),*$(,)?})*}) => {
        #[doc = concat!($name, ".")]
        pub mod $mod {
            paste::paste! {
                use super::*;

                $(
                    pin!($name, enum $pinname {
                        $(
                            [<P $var>]<Alternate<{[<PORT $var _A>]::$varname as u8}, $mode>>,
                        )*
                    });
                )*
            }
        }
    };
}

// total guesses on pin modes
pins!(xtah, "XTAH port", {
    enum Xi: XtahXi {
        A3<Input<Floating>>,
    }

    enum Xo: XtahXo {
        A4<Output<PushPull>>,
    }
});

// also total guesses on pin modes
pins!(xtal, "XTAL port", {
    enum Xi: XtalXi {
        A1<Input<Floating>>,
    }

    enum Xo: XtalXo {
        A2<Output<PushPull>>,
    }
});

pins!(uart0, "UART0", {
    enum Rx: Uart0Rx {
        B8<Input<Floating>>,
        C4<Input<Floating>>,
    }

    enum Tx: Uart0Tx {
        B7<Output<PushPull>>,
        C3<Output<PushPull>>,
    }

    enum Rts: Uart0Rts {
        B10<Output<PushPull>>,
    }

    enum Cts: Uart0Cts {
        B9<Input<Floating>>,
    }
});

pins!(uart1, "UART1", {
    enum Rx: Uart1Rx {
        A8<Input<Floating>>,
        B13<Input<Floating>>,
    }

    enum Tx: Uart1Tx {
        A7<Output<PushPull>>,
        B12<Output<PushPull>>,
    }

    enum Rts: Uart1Rts {
        A6<Output<PushPull>>,
    }

    enum Cts: Uart1Cts {
        A5<Input<Floating>>,
    }
});

pins!(uart2, "UART2", {
    enum Rx: Uart2Rx {
        B1<Input<Floating>>,
        B15<Input<Floating>>,
    }

    enum Tx: Uart2Tx {
        B0<Output<PushPull>>,
        B14<Output<PushPull>>,
    }

    enum Rts: Uart2Rts {
        C1<Output<PushPull>>,
    }

    enum Cts: Uart2Cts {
        C0<Input<Floating>>,
    }
});
