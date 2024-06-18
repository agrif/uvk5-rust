use crate::power::Gate;
use crate::time::Hertz;

use super::{static_assert_timer_hz_not_zero, BaseInstance, Error, High, Low, Timer};

/// Wrap a timer register into a configurator.
pub fn new<T>(timer: T, gate: Gate<T>) -> Config<T, 0>
where
    T: BaseInstance,
{
    Config::new(timer, gate)
}

/// Allows you to configure a timer peripheral.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<T, const HZ: u32> {
    timer: T,
}

/// The two timers in each peripheral, [Low] and [High].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Pair<T, const HZ: u32> {
    pub low: Timer<T, Low, HZ>,
    pub high: Timer<T, High, HZ>,
}

impl<T> Config<T, 0>
where
    T: BaseInstance,
{
    /// Wrap a timer register into a configurator.
    pub fn new(mut timer: T, mut gate: Gate<T>) -> Self {
        gate.enable();

        // safety: we own timer exclusively, which gives us control here
        unsafe {
            timer.reset();
        }

        Self { timer }
    }
}

impl<T, const HZ: u32> Config<T, HZ>
where
    T: BaseInstance,
{
    fn sys_clk(&self) -> Hertz {
        // safety: we own self, which gives us control of this gate
        let gate = unsafe { Gate::<T>::steal() };
        gate.clocks().sys_clk()
    }

    /// Recover the raw timer register from this configurator.
    pub fn free(self) -> (T, Gate<T>) {
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
    pub unsafe fn divider<const C_HZ: u32>(mut self, div: u16) -> Config<T, C_HZ> {
        // safety: we own self, which gives us control here
        self.timer.set_div(div);
        Config { timer: self.timer }
    }

    /// Use `sys_clk` as input, and set the divider to most closely
    /// match the given frequency.
    ///
    /// If the frequency is too or high to be matched, returns [Err].
    pub fn frequency<const C_HZ: u32>(self) -> Result<Config<T, C_HZ>, Error> {
        let div = self
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
    pub fn input_clk(&self) -> Hertz {
        self.sys_clk() / (self.timer.get_div() as u32 + 1)
    }

    /// Split the configured timer into [Low] and [High] parts.
    ///
    /// Note: This will fail at compile-time if HZ is 0. Use
    /// [Self::frequency()] to configure HZ.
    pub fn split(self) -> Pair<T, HZ> {
        Pair::new(self)
    }
}

impl<T, const HZ: u32> Pair<T, HZ>
where
    T: BaseInstance,
{
    /// Split the configured timer into [Low] and [High] parts.
    ///
    /// Note: This will fail at compile-time if HZ is 0. Use
    /// [Config::frequency()] to configure HZ.
    pub fn new(config: Config<T, HZ>) -> Self {
        static_assert_timer_hz_not_zero::<HZ>();

        let input_clk = config.input_clk();
        // safety: we are splitting into exclusive high/low parts
        let timer = unsafe { config.timer.steal() };

        Self {
            low: Timer {
                timer: config.timer,
                _half: Default::default(),
                input_clk,
            },
            high: Timer {
                timer,
                _half: Default::default(),
                input_clk,
            },
        }
    }

    /// Recombine the two parts into a timer configurator.
    pub fn free(self) -> Config<T, HZ> {
        // arbitrarily return low timer and drop high
        Config {
            timer: self.low.timer,
        }
    }
}
