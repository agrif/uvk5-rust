use crate::power::{Clocks, Gate};
use crate::time::Hertz;

use super::{Base, High, Low};

/// Wrap the timer register into a configurator.
#[inline(always)]
pub fn new<Timer>(timer: Timer, gate: Gate<Timer>) -> Config<Timer>
where
    Timer: Base,
{
    Config::new(timer, gate)
}

/// Allows you to configure a timer peripheral.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<Timer> {
    timer: Timer,
}

/// The two timers in each peripheral, Low and High.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LowHigh<Timer> {
    pub low: Low<Timer>,
    pub high: High<Timer>,
}

impl<Timer> Config<Timer>
where
    Timer: Base,
{
    /// Wrap the timer register into a configurator.
    #[inline(always)]
    pub fn new(mut timer: Timer, mut gate: Gate<Timer>) -> Config<Timer> {
        // safety: we own timer exclusively, which gives us control here
        unsafe {
            timer.reset();
        }
        gate.enable();
        Self { timer }
    }

    /// Recover the raw itmer register from this configurator.
    #[inline(always)]
    pub fn free(self) -> (Timer, Gate<Timer>) {
        // safety: we own self, which gives us control of this gate
        let mut gate = unsafe { Gate::steal() };
        gate.disable();
        (self.timer, gate)
    }

    /// Use sys_clk as input, and set the divider to 1 + div.
    #[inline(always)]
    pub fn divider(mut self, div: u16) -> Self {
        // safety: we own self, which gives us control here
        unsafe {
            self.timer.set_div(div);
        }
        self
    }

    /// Use sys_clk as input, and set the divider to most closely
    /// match the given frequency.
    ///
    /// If the frequency is too low to be matched, the highest
    /// possible divider value will be used.
    #[inline(always)]
    pub fn frequency(self, clocks: &Clocks, freq: Hertz) -> Self {
        let div = (clocks.sys_clk() / freq).min(u16::MAX as u32);
        self.divider((div.max(1) - 1) as u16)
    }

    /// Get the configured timer input frequency.
    #[inline(always)]
    pub fn input_clk(&self, clocks: &Clocks) -> Hertz {
        clocks.sys_clk() / (self.timer.get_div() as u32 + 1)
    }

    /// Split the configured timer into Low and High parts.
    #[inline(always)]
    pub fn split(self, clocks: &Clocks) -> LowHigh<Timer> {
        LowHigh::new(self, clocks)
    }
}

impl<Timer> LowHigh<Timer>
where
    Timer: Base,
{
    /// Split the configured timer into Low and High parts.
    #[inline(always)]
    pub fn new(config: Config<Timer>, clocks: &Clocks) -> Self {
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
    pub fn free(self) -> Config<Timer> {
        // arbitrarily return low timer and drop high
        Config {
            timer: self.low.timer,
        }
    }
}
