use super::PinState;

// seal for PinMode trait
trait Sealed {}

/// A trait for pin mode type states.
pub(super) trait PinModeSealed {
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

/// A trait for pin mode type states.
#[allow(private_bounds)]
#[cfg(not(feature = "defmt"))]
pub trait PinMode: PinModeSealed + core::fmt::Debug + Default {
    /// For Alternate modes, this is the inner mode. For all others, this
    /// is Self.
    type Inner: PinMode;
}

/// A trait for pin mode type states.
#[allow(private_bounds)]
#[cfg(feature = "defmt")]
pub trait PinMode: PinModeSealed + core::fmt::Debug + defmt::Format + Default {
    /// For Alternate modes, this is the inner mode. For all others, this
    /// is Self.
    type Inner: PinMode;
}

/// Unspecified pin state, unusable until changed. (type state)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Unspecified;

impl PinMode for Unspecified {
    type Inner = Self;
}

impl PinModeSealed for Unspecified {
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
    #[inline(always)]
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
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("Input").field(&Pull::default()).finish()
    }
}

#[cfg(feature = "defmt")]
impl<Pull> defmt::Format for Input<Pull>
where
    Pull: Default + defmt::Format,
{
    #[allow(clippy::missing_inline_in_public_items)]
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Input({})", Pull::default())
    }
}

impl PinMode for Input<Floating> {
    type Inner = Self;
}

impl PinModeSealed for Input<Floating> {
    const UNSPECIFIED: bool = false;

    const IE: bool = true;
    const PD: bool = false;
    const PU: bool = false;

    const OD: bool = false;

    const SEL: u8 = 0;
    const DIR: bool = false;
}

impl PinMode for Input<PullUp> {
    type Inner = Self;
}

impl PinModeSealed for Input<PullUp> {
    const UNSPECIFIED: bool = false;

    const IE: bool = true;
    const PD: bool = false;
    const PU: bool = true;

    const OD: bool = false;

    const SEL: u8 = 0;
    const DIR: bool = false;
}

impl PinMode for Input<PullDown> {
    type Inner = Self;
}

impl PinModeSealed for Input<PullDown> {
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
    #[inline(always)]
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
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("Output").field(&Mode::default()).finish()
    }
}

#[cfg(feature = "defmt")]
impl<Mode> defmt::Format for Output<Mode>
where
    Mode: Default + defmt::Format,
{
    #[allow(clippy::missing_inline_in_public_items)]
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Output({})", Mode::default())
    }
}

impl PinMode for Output<PushPull> {
    type Inner = Self;
}

impl PinModeSealed for Output<PushPull> {
    const UNSPECIFIED: bool = false;

    const IE: bool = false;
    const PD: bool = false;
    const PU: bool = false;

    const OD: bool = false;

    const SEL: u8 = 0;
    const DIR: bool = true;
}

impl PinMode for Output<OpenDrain> {
    type Inner = Self;
}

impl PinModeSealed for Output<OpenDrain> {
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
    #[inline(always)]
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
    #[allow(clippy::missing_inline_in_public_items)]
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
    #[allow(clippy::missing_inline_in_public_items)]
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Alternate({}, {})", A, Mode::default())
    }
}

// avoid repitition for alternate modes
macro_rules! impl_alternate {
    ($Mode:ty) => {
        impl<const A: u8> PinMode for Alternate<A, $Mode> {
            type Inner = $Mode;
        }

        impl<const A: u8> PinModeSealed for Alternate<A, $Mode> {
            const UNSPECIFIED: bool = <$Mode as PinModeSealed>::UNSPECIFIED;

            const IE: bool = <$Mode as PinModeSealed>::IE;
            const PD: bool = <$Mode as PinModeSealed>::PD;
            const PU: bool = <$Mode as PinModeSealed>::PU;

            const OD: bool = <$Mode as PinModeSealed>::OD;

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

// A macro to implement aliases on top of into_mode and into_mode_in_state.
macro_rules! into_mode_aliases {
    ($(vis $vis:tt,)? ($($as:tt)*), ($($args:tt)*)) => {
        /// Convert pin into a floating input.
        #[inline(always)]
        $($vis)? fn into_floating_input(self) -> $($as)*<$($args)* Input<Floating>> {
            self.into_mode()
        }

        /// Temporarily configure pin as a floating input.
        #[inline(always)]
        $($vis)? fn with_floating_input<R>(&mut self, f: impl FnOnce(&mut $($as)*<$($args)* Input<Floating>>) -> R) -> R {
            self.with_mode(f)
        }

        /// Convert pin into an input with a pull-up resistor.
        #[inline(always)]
        $($vis)? fn into_pull_up_input(self) -> $($as)*<$($args)* Input<PullUp>> {
            self.into_mode()
        }

        /// Temporarily configure pin as an input with a pull-up resistor.
        #[inline(always)]
        $($vis)? fn with_pull_up_input<R>(&mut self, f: impl FnOnce(&mut $($as)*<$($args)* Input<PullUp>>) -> R) -> R {
            self.with_mode(f)
        }

        /// Convert pin into an input with a pull-down resistor.
        #[inline(always)]
        $($vis)? fn into_pull_down_input(self) -> $($as)*<$($args)* Input<PullDown>> {
            self.into_mode()
        }

        /// Temporarily configure pin as an input with a pull-down resistor.
        #[inline(always)]
        $($vis)? fn with_pull_down_input<R>(&mut self, f: impl FnOnce(&mut $($as)*<$($args)* Input<PullDown>>) -> R) -> R {
            self.with_mode(f)
        }

        /// Convert pin into a push-pull output, initially low.
        #[inline(always)]
        $($vis)? fn into_push_pull_output(self) -> $($as)*<$($args)* Output<PushPull>> {
            self.into_mode_in_state(PinState::Low)
        }

        /// Temporarily configure pin as a push-pull output.
        ///
        /// The initial state is retained if the original mode was
        /// also an output mode. It is otherwise undefined.
        #[inline(always)]
        $($vis)? fn with_push_pull_output<R>(&mut self, f: impl FnOnce(&mut $($as)*<$($args)* Output<PushPull>>) -> R) -> R {
            self.with_mode(f)
        }

        /// Convert a pin into a push-pull output in the given state.
        #[inline(always)]
        $($vis)? fn into_push_pull_output_in_state(
            self,
            state: PinState,
        ) -> $($as)*<$($args)* Output<PushPull>> {
            self.into_mode_in_state(state)
        }

        /// Temporarily configure pin as a push-pull output in the given state.
        #[inline(always)]
        $($vis)? fn with_push_pull_output_in_state<R>(
            &mut self,
            state: PinState,
            f: impl FnOnce(&mut $($as)*<$($args)* Output<PushPull>>) -> R,
        ) -> R {
            self.with_mode_in_state(state, f)
        }

        /// Convert pin into an open-drain output, initially low.
        #[inline(always)]
        $($vis)? fn into_open_drain_output(self) -> $($as)*<$($args)* Output<OpenDrain>> {
            self.into_mode_in_state(PinState::Low)
        }

        /// Temporarily configure pin as an open-drain output.
        ///
        /// The initial state is retained if the original mode was
        /// also an output mode. It is otherwise undefined.
        #[inline(always)]
        $($vis)? fn with_open_drain_output<R>(&mut self, f: impl FnOnce(&mut $($as)*<$($args)* Output<OpenDrain>>) -> R) -> R {
            self.with_mode(f)
        }

        /// Convert pin into an open-drain output, initially low.
        #[inline(always)]
        $($vis)? fn into_open_drain_output_in_state(
            self,
            state: PinState,
        ) -> $($as)*<$($args)* Output<OpenDrain>> {
            self.into_mode_in_state(state)
        }

        /// Temporarily configure pin as an open-drain output in the given state.
        #[inline(always)]
        $($vis)? fn with_open_drain_output_in_state<R>(
            &mut self,
            state: PinState,
            f: impl FnOnce(&mut $($as)*<$($args)* Output<OpenDrain>>) -> R,
        ) -> R {
            self.with_mode_in_state(state, f)
        }
    };
}

// allow this to be used elsewhere in gpio
pub(super) use into_mode_aliases;

/// A pin that can change mode.
pub trait IntoMode: Sized {
    /// The current pin type, with the mode changed to Mode.
    type As<Mode>;

    /// Get the pin number of this pin.
    fn pin(&self) -> u8;

    /// Get the port of this pin.
    fn port(&self) -> char;

    /// Convert pin into a new mode.
    fn into_mode<Mode>(self) -> Self::As<Mode>
    where
        Mode: PinMode;

    /// Convert pin into a new mode, in the given initial state.
    fn into_mode_in_state<Mode>(self, state: PinState) -> Self::As<Output<Mode>>
    where
        Output<Mode>: PinMode;

    /// Temporarily configure this pin in a new mode.
    ///
    /// If this is an output mode, the initial state is retained if
    /// the original mode was also an output mode. It is otherwise
    /// undefined.
    fn with_mode<Mode, R>(&mut self, f: impl FnOnce(&mut Self::As<Mode>) -> R) -> R
    where
        Mode: PinMode;

    /// Temporarily configure this pin in a new mode, in the given
    /// initial state.
    fn with_mode_in_state<Mode, R>(
        &mut self,
        state: PinState,
        f: impl FnOnce(&mut Self::As<Output<Mode>>) -> R,
    ) -> R
    where
        Output<Mode>: PinMode;

    into_mode_aliases!((Self::As), ());
}
