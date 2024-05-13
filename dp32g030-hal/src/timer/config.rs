use crate::power::{Clocks, Gate};
use crate::time::Hertz;

use super::{static_assert_timer_hz_not_zero, Base, Error, High, Low};

/// Wrap a timer register into a configurator.
#[inline(always)]
pub fn new<Timer>(timer: Timer, gate: Gate<Timer>) -> Config<Timer, 0>
where
    Timer: Base,
{
    Config::new(timer, gate)
}

/// Allows you to configure a timer peripheral.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<Timer, const HZ: u32> {
    timer: Timer,
}

/// The two timers in each peripheral, [Low] and [High].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LowHigh<Timer, const HZ: u32> {
    pub low: Low<Timer, HZ>,
    pub high: High<Timer, HZ>,
}

impl<Timer> Config<Timer, 0>
where
    Timer: Base,
{
    /// Wrap a timer register into a configurator.
    #[inline(always)]
    pub fn new(mut timer: Timer, mut gate: Gate<Timer>) -> Self {
        // safety: we own timer exclusively, which gives us control here
        unsafe {
            timer.reset();
        }
        gate.enable();
        Self { timer }
    }
}

impl<Timer, const HZ: u32> Config<Timer, HZ>
where
    Timer: Base,
{
    /// Recover the raw itmer register from this configurator.
    #[inline(always)]
    pub fn free(self) -> (Timer, Gate<Timer>) {
        // safety: we own self, which gives us control of this gate
        let mut gate = unsafe { Gate::steal() };
        gate.disable();
        (self.timer, gate)
    }

    /// Use `sys_clk` as input, and set the divider to 1 + `div`.
    ///
    /// # Safety
    ///
    /// This allows you to change the const frequency attached to this
    /// timer arbitrarily. You must use type annotations to ensure it
    /// reflects the real resulting frequency of this timer.
    ///
    /// Use [Self::frequency()] instead for a safer interface.
    #[inline(always)]
    pub unsafe fn divider<const C_HZ: u32>(mut self, div: u16) -> Config<Timer, C_HZ> {
        // safety: we own self, which gives us control here
        self.timer.set_div(div);
        Config { timer: self.timer }
    }

    /// Use `sys_clk` as input, and set the divider to most closely
    /// match the given frequency.
    ///
    /// If the frequency is too or high to be matched, returns [Err].
    #[inline(always)]
    pub fn frequency<const C_HZ: u32>(self, clocks: &Clocks) -> Result<Config<Timer, C_HZ>, Error> {
        let div = clocks
            .sys_clk()
            .to_Hz()
            .checked_add(C_HZ / 2)
            .ok_or(Error::OutOfRange)?
            .checked_div(C_HZ)
            .ok_or(Error::OutOfRange)?
            .checked_sub(1)
            .ok_or(Error::OutOfRange)?
            .try_into()
            .map_err(|_| Error::OutOfRange)?;

        // safety: we have calculated the divider correctly above
        unsafe { Ok(self.divider(div)) }
    }

    /// Get the configured timer input frequency.
    ///
    /// This may differ from the statically known frequency, as this
    /// uses run-time corrections to the system clock.
    #[inline(always)]
    pub fn input_clk(&self, clocks: &Clocks) -> Hertz {
        clocks.sys_clk() / (self.timer.get_div() as u32 + 1)
    }

    /// Split the configured timer into [Low] and [High] parts.
    ///
    /// Note: This will fail at compile-time if HZ is 0. Use
    /// [Self::frequency()] to configure HZ.
    #[inline(always)]
    pub fn split(self, clocks: &Clocks) -> LowHigh<Timer, HZ> {
        LowHigh::new(self, clocks)
    }
}

impl<Timer, const HZ: u32> LowHigh<Timer, HZ>
where
    Timer: Base,
{
    /// Split the configured timer into [Low] and [High] parts.
    ///
    /// Note: This will fail at compile-time if HZ is 0. Use
    /// [Config::frequency()] to configure HZ.
    #[inline(always)]
    pub fn new(config: Config<Timer, HZ>, clocks: &Clocks) -> Self {
        static_assert_timer_hz_not_zero::<HZ>();

        let input_clk = config.input_clk(clocks);
        // safety: we are splitting into exclusive high/low parts
        let timer = unsafe { config.timer.steal() };

        Self {
            low: Low {
                timer: config.timer,
                input_clk,
            },
            high: High { timer, input_clk },
        }
    }

    /// Recombine the two parts into a timer configurator.
    #[inline(always)]
    pub fn free(self) -> Config<Timer, HZ> {
        // arbitrarily return low timer and drop high
        Config {
            timer: self.low.timer,
        }
    }
}
