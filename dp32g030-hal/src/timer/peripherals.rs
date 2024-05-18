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
            #[inline(always)]
            unsafe fn reset(&mut self) {
                self.en().reset();
                self.div().reset();
                self.ie().reset();
                self.if_().reset();
                self.high_load().reset();
                self.low_load().reset();
            }

            #[inline(always)]
            unsafe fn set_div(&mut self, div: u16) {
                self.div().clear_bits(|w| w.div().bits(0));
                self.div().set_bits(|w| w.div().bits(div));
            }

            #[inline(always)]
            fn get_div(&self) -> u16 {
                self.div().read().div().bits()
            }

            #[inline(always)]
            unsafe fn steal(&self) -> Self {
                Self::steal()
            }

            #[inline(always)]
            unsafe fn set_enabled(&mut self, high: bool, enable: bool) {
                if high {
                    if enable {
                        self.en().set_bits(|w| w.high_en().enabled())
                    } else {
                        self.en().clear_bits(|w| w.high_en().disabled())
                    }
                } else {
                    if enable {
                        self.en().set_bits(|w| w.low_en().enabled())
                    } else {
                        self.en().clear_bits(|w| w.low_en().disabled())
                    }
                }
            }

            #[inline(always)]
            fn get_enabled(&self, high: bool) -> bool {
                if high {
                    self.en().read().high_en().is_enabled()
                } else {
                    self.en().read().low_en().is_enabled()
                }
            }

            #[inline(always)]
            fn get_flag(&self, high: bool) -> bool {
                if high {
                    self.if_().read().high_if().is_set()
                } else {
                    self.if_().read().low_if().is_set()
                }
            }

            #[inline(always)]
            unsafe fn clear_flag(&mut self, high: bool) {
                // write 1 to clear
                if high {
                    self.if_().set_bits(|w| w.high_if().set_())
                } else {
                    self.if_().set_bits(|w| w.low_if().set_())
                }
            }

            #[inline(always)]
            fn get_load(&self, high: bool) -> u16 {
                if high {
                    self.high_load().read().high_load().bits()
                } else {
                    self.low_load().read().low_load().bits()
                }
            }

            #[inline(always)]
            unsafe fn set_load(&mut self, high: bool, load: u16) {
                if high {
                    self.high_load().write(|w| w.high_load().bits(load))
                } else {
                    self.low_load().write(|w| w.low_load().bits(load))
                }
            }

            #[inline(always)]
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
