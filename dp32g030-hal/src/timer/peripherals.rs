use crate::pac;

use crate::power::Device;

/// A trait for base timers.
#[allow(private_bounds)]
pub trait BaseInstance: BaseInstanceSealed + Device {}

/// A trait for base timers.
pub(super) trait BaseInstanceSealed {
    /// Reset the timer peripheral.
    ///
    /// # Safety
    /// You must be the sole owner of this entire timer.
    unsafe fn reset(&mut self);

    /// Set the timer's input divider.
    ///
    /// # Safety
    /// You must be the sole owner of this entire timer.
    unsafe fn set_div(&mut self, div: u16);

    /// Get the timer's input divider.
    fn get_div(&self) -> u16;

    /// Steal the timer peripheral.
    ///
    /// # Safety
    /// Every existing clone of this timer peripheral must be
    /// used for exclusive purposes, such as Low and High sides.
    unsafe fn steal(&self) -> Self;

    /// Set the Low/High enable value.
    ///
    /// # Safety
    /// This must only be accessed from at most one owner.
    unsafe fn set_enabled(&mut self, high: bool, enable: bool);

    /// Get the Low/High enable value.
    fn get_enabled(&self, high: bool) -> bool;

    /// Get the Low/High flag value.
    fn get_flag(&self, high: bool) -> bool;

    /// Clear the Low/High flag value.
    ///
    /// # Safety
    /// This must only be accessed from at most one owner.
    unsafe fn clear_flag(&mut self, high: bool);

    /// Get the Low/High load value.
    fn get_load(&self, high: bool) -> u16;

    /// Set the Low/High load value.
    ///
    /// # Safety
    /// This must only be accessed from at most one owner.
    unsafe fn set_load(&mut self, high: bool, load: u16);

    /// Get the Low/High count value.
    fn get_count(&self, high: bool) -> u16;
}

macro_rules! impl_base {
    ($timer:path) => {
        impl BaseInstance for $timer {}

        impl BaseInstanceSealed for $timer {
            unsafe fn reset(&mut self) {
                // only called when both halves are owned
                self.en().reset();
                self.div().reset();
                self.ie().reset();
                self.if_().reset();
                self.high_load().reset();
                self.low_load().reset();
            }

            unsafe fn set_div(&mut self, div: u16) {
                // only called when both halves are owned
                self.div().modify(|_r, w| w.div().bits(div));
            }

            fn get_div(&self) -> u16 {
                self.div().read().div().bits()
            }

            unsafe fn steal(&self) -> Self {
                Self::steal()
            }

            unsafe fn set_enabled(&mut self, high: bool, enable: bool) {
                // use a critical section, as this register is shared
                critical_section::with(|_cs| {
                    if high {
                        self.en().modify(|_r, w| w.high_en().bit(enable));
                    } else {
                        self.en().modify(|_r, w| w.low_en().bit(enable));
                    }
                });
            }

            fn get_enabled(&self, high: bool) -> bool {
                if high {
                    self.en().read().high_en().is_enabled()
                } else {
                    self.en().read().low_en().is_enabled()
                }
            }

            fn get_flag(&self, high: bool) -> bool {
                if high {
                    self.if_().read().high_if().is_set()
                } else {
                    self.if_().read().low_if().is_set()
                }
            }

            unsafe fn clear_flag(&mut self, high: bool) {
                // use a critical section, as this register is shared
                critical_section::with(|_cs| {
                    // write 1 to clear
                    if high {
                        self.if_().modify(|_r, w| w.high_if().set_())
                    } else {
                        self.if_().modify(|_r, w| w.low_if().set_())
                    }
                });
            }

            fn get_load(&self, high: bool) -> u16 {
                if high {
                    self.high_load().read().high_load().bits()
                } else {
                    self.low_load().read().low_load().bits()
                }
            }

            unsafe fn set_load(&mut self, high: bool, load: u16) {
                // registers are not shared between high/low, so now
                // critical section
                if high {
                    self.high_load().write(|w| w.high_load().bits(load))
                } else {
                    self.low_load().write(|w| w.low_load().bits(load))
                }
            }

            fn get_count(&self, high: bool) -> u16 {
                if high {
                    self.high_cnt().read().high_cnt().bits()
                } else {
                    self.low_cnt().read().low_cnt().bits()
                }
            }
        }
    };
}

impl_base!(pac::TIMER_BASE0);
impl_base!(pac::TIMER_BASE1);
impl_base!(pac::TIMER_BASE2);
