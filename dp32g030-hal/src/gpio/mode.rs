// seal for PinMode trait
trait Sealed {}

/// A trait for pin mode type states.
#[allow(private_bounds)]
pub trait PinMode: Sealed + core::fmt::Debug + Default {
    /// For Alternate modes, this is the inner mode. For all others, this
    /// is Self.
    type Inner: PinMode;

    /// Whether to force full reconfiguration if this is the current pin mode.
    const UNSPECIFIED: bool;

    /// Input enable.
    const IE: bool;
    /// Pull-down.
    const PD: bool;
    /// Pull-up;
    const PU: bool;

    /// Open-drain.
    const OD: bool;

    /// Function selection, 4 bits at most.
    const SEL: u8;
    /// GPIO direction, 0 is input, 1 is output. Probably the same as !IE.
    const DIR: bool;
}

/// Unspecified pin state, unusable until changed. (type state)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Unspecified;

impl Sealed for Unspecified {}
impl PinMode for Unspecified {
    type Inner = Self;

    const UNSPECIFIED: bool = true;

    const IE: bool = false;
    const PD: bool = false;
    const PU: bool = false;

    const OD: bool = false;

    const SEL: u8 = 0;
    const DIR: bool = false;
}

/// Floating input. (type state)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Floating;

/// Pull-up on input. (type state)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PullUp;

/// Pull-down on input. (type state)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PullDown;

/// Input mode, with optional pull up or down. (type state)
pub struct Input<Pull = Floating> {
    _marker: core::marker::PhantomData<Pull>,
}

impl<Pull> Default for Input<Pull> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<Pull> core::fmt::Debug for Input<Pull>
where
    Pull: Default + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("Input").field(&Pull::default()).finish()
    }
}

#[cfg(feature = "defmt")]
impl<Pull> defmt::Format for Input<Pull>
where
    Pull: Default + defmt::Format,
{
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Input({})", Pull::default())
    }
}

impl Sealed for Input<Floating> {}
impl PinMode for Input<Floating> {
    type Inner = Self;

    const UNSPECIFIED: bool = false;

    const IE: bool = true;
    const PD: bool = false;
    const PU: bool = false;

    const OD: bool = false;

    const SEL: u8 = 0;
    const DIR: bool = false;
}

impl Sealed for Input<PullUp> {}
impl PinMode for Input<PullUp> {
    type Inner = Self;

    const UNSPECIFIED: bool = false;

    const IE: bool = true;
    const PD: bool = false;
    const PU: bool = true;

    const OD: bool = false;

    const SEL: u8 = 0;
    const DIR: bool = false;
}

impl Sealed for Input<PullDown> {}
impl PinMode for Input<PullDown> {
    type Inner = Self;

    const UNSPECIFIED: bool = false;

    const IE: bool = true;
    const PD: bool = true;
    const PU: bool = false;

    const OD: bool = false;

    const SEL: u8 = 0;
    const DIR: bool = false;
}

/// Push-pull output. (type state)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PushPull;

/// Open-drain output. (type state)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OpenDrain;

/// Output mode, either push-pull or open-drain. (type state)
pub struct Output<Mode = PushPull> {
    _marker: core::marker::PhantomData<Mode>,
}

impl<Mode> Default for Output<Mode> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<Mode> core::fmt::Debug for Output<Mode>
where
    Mode: Default + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("Output").field(&Mode::default()).finish()
    }
}

#[cfg(feature = "defmt")]
impl<Mode> defmt::Format for Output<Mode>
where
    Mode: Default + defmt::Format,
{
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Output({})", Mode::default())
    }
}

impl Sealed for Output<PushPull> {}
impl PinMode for Output<PushPull> {
    type Inner = Self;

    const UNSPECIFIED: bool = false;

    const IE: bool = false;
    const PD: bool = false;
    const PU: bool = false;

    const OD: bool = false;

    const SEL: u8 = 0;
    const DIR: bool = true;
}

impl Sealed for Output<OpenDrain> {}
impl PinMode for Output<OpenDrain> {
    type Inner = Self;

    const UNSPECIFIED: bool = false;

    const IE: bool = false;
    const PD: bool = false;
    const PU: bool = false;

    const OD: bool = true;

    const SEL: u8 = 0;
    const DIR: bool = true;
}

/// Alternate pin mode, 1 <= A < 16. (type state)
pub struct Alternate<const A: u8, Mode> {
    _marker: core::marker::PhantomData<Mode>,
}

impl<const A: u8, Mode> Default for Alternate<A, Mode> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<const A: u8, Mode> core::fmt::Debug for Alternate<A, Mode>
where
    Mode: Default + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("Alternate")
            .field(&A)
            .field(&Mode::default())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl<const A: u8, Mode> defmt::Format for Alternate<A, Mode>
where
    Mode: Default + defmt::Format,
{
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Alternate({}, {})", A, Mode::default())
    }
}

// avoid repitition for alternate modes
macro_rules! impl_alternate {
    ($Mode:ty) => {
        impl<const A: u8> Sealed for Alternate<A, $Mode> {}

        impl<const A: u8> PinMode for Alternate<A, $Mode> {
            type Inner = $Mode;

            const UNSPECIFIED: bool = <$Mode as PinMode>::UNSPECIFIED;

            const IE: bool = <$Mode as PinMode>::IE;
            const PD: bool = <$Mode as PinMode>::PD;
            const PU: bool = <$Mode as PinMode>::PU;

            const OD: bool = <$Mode as PinMode>::OD;

            const SEL: u8 = A;
            const DIR: bool = false;
        }
    };
}

impl_alternate!(Input<Floating>);
impl_alternate!(Input<PullUp>);
impl_alternate!(Input<PullDown>);
impl_alternate!(Output<PushPull>);
impl_alternate!(Output<OpenDrain>);
