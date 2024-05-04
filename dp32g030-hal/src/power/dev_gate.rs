use crate::pac;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumIter)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// All possible devices controlled by DevGate.
pub enum Dev {
    GpioA,
    GpioB,
    GpioC,
    I2c0,
    I2c1,
    Uart0,
    Uart1,
    Uart2,
    Spi0,
    Spi1,
    TimerBase0,
    TimerBase1,
    TimerBase2,
    TimerPlus0,
    TimerPlus1,
    PwmBase0,
    PwmBase1,
    PwmPlus0,
    PwmPlus1,
    Rtc,
    Iwdt,
    Wwdt,
    Saradc,
    Crc,
    Aes,
}

impl Dev {
    #[inline(always)]
    /// Iterate over all possible devices.
    pub fn iter() -> DevIter {
        <Self as strum::IntoEnumIterator>::iter()
    }
}

/// Control power to individual devices.
pub struct DevGate {
    syscon: pac::SYSCON,
}

impl core::fmt::Debug for DevGate {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut tup = f.debug_tuple("DevGate");
        for dev in Dev::iter() {
            if self.is_enabled(dev) {
                tup.field(&dev);
            }
        }
        tup.finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for DevGate {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "DevGate(");
        let mut first = true;
        for dev in Dev::iter() {
            if self.is_enabled(dev) {
                if first {
                    defmt::write!(f, "{}", dev);
                    first = false;
                } else {
                    defmt::write!(f, ", {}", dev);
                }
            }
        }
        defmt::write!(f, ")");
    }
}

// way too much repitition to not use a macro
macro_rules! dev_gate_impl {
    {$(($var:tt, $name:tt, $field:tt)),+,} => {
        paste::paste!{
            /// Enable a device.
            #[inline]
            pub fn enable(&mut self, dev: Dev) -> &mut Self {
                // safety: setting bits in this register is ok
                unsafe {
                    self.syscon.dev_clk_gate().set_bits(|w| match dev {
                        $(Dev::$var => w.$field().enabled(),)+
                    })
                }
                self
            }
            /// Disable a device.
            #[inline]
            pub fn disable(&mut self, dev: Dev) -> &mut Self {
                // safety: setting bits in this register is ok
                unsafe {
                    self.syscon.dev_clk_gate().set_bits(|w| match dev {
                        $(Dev::$var => w.$field().disabled(),)+
                    })
                }
                self
            }
            /// Get whether a device is enabled.
            #[inline]
            pub fn is_enabled(&self, dev: Dev) -> bool {
                let r = self.syscon.dev_clk_gate().read();
                match dev {
                    $(Dev::$var => r.$field().is_enabled(),)+
                }
            }
            $(
                #[inline(always)]
                #[doc = concat!("Enable ", stringify!($var), ".")]
                pub fn [<enable_ $name>](&mut self) -> &mut Self {
                    // safety: setting bits in dev_clk_gate is ok
                    unsafe {
                        self.syscon.dev_clk_gate().set_bits(|w| {
                            w.$field().enabled()
                        })
                    }
                    self
                }
            )+
            $(
                #[inline(always)]
                #[doc = concat!("Disable ", stringify!($var), ".")]
                pub fn [<disable_ $name>](&mut self) -> &mut Self {
                    // safety: clearing bits in dev_clk_gate is ok
                    unsafe {
                        self.syscon.dev_clk_gate().clear_bits(|w| {
                            w.$field().disabled()
                        })
                    }
                    self
                }
            )+
            $(
                #[inline(always)]
                #[doc = concat!("Set whether ", stringify!($var), " is enabled.")]
                pub fn [<set_ $name _enabled>](&mut self, enabled: bool) -> &mut Self {
                    if enabled {
                        self.[<enable_ $name>]()
                    } else {
                        self.[<disable_ $name>]()
                    }
                }
            )+
            $(
                #[inline(always)]
                #[doc = concat!("Get whether ", stringify!($var), " is enabled.")]
                pub fn [<is_ $name _enabled>](&self) -> bool {
                    self.syscon.dev_clk_gate().read().$field().is_enabled()
                }
            )+
        }
    }
}

impl DevGate {
    /// safety: this peripheral reads and writes SYSCON.dev_clk_gate()
    #[inline(always)]
    pub(crate) unsafe fn steal() -> Self {
        Self {
            syscon: pac::SYSCON::steal(),
        }
    }

    #[inline(always)]
    /// Reset all devices to disabled.
    pub fn reset(&mut self) -> &mut Self {
        self.syscon.dev_clk_gate().reset();
        self
    }

    dev_gate_impl! {
        (GpioA, gpioa, gpioa_clk_gate),
        (GpioB, gpiob, gpiob_clk_gate),
        (GpioC, gpioc, gpioc_clk_gate),
        (I2c0, iic0, iic0_clk_gate),
        (I2c1, iic1, iic1_clk_gate),
        (Uart0, uart0, uart0_clk_gate),
        (Uart1, uart1, uart1_clk_gate),
        (Uart2, uart2, uart2_clk_gate),
        (Spi0, spi0, spi0_clk_gate),
        (Spi1, spi1, spi1_clk_gate),
        (TimerBase0, timer_base0, timer_base0_clk_gate),
        (TimerBase1, timer_base1, timer_base1_clk_gate),
        (TimerBase2, timer_base2, timer_base2_clk_gate),
        (TimerPlus0, timer_plus0, timer_plus0_clk_gate),
        (TimerPlus1, timer_plus1, timer_plus1_clk_gate),
        (PwmBase0, pwm_base0, pwm_base0_clk_gate),
        (PwmBase1, pwm_base1, pwm_base1_clk_gate),
        (PwmPlus0, pwm_plus0, pwm_plus0_clk_gate),
        (PwmPlus1, pwm_plus1, pwm_plus1_clk_gate),
        (Rtc, rtc, rtc_clk_gate),
        (Iwdt, iwdt, iwdt_clk_gate),
        (Wwdt, wwdt, wwdt_clk_gate),
        (Saradc, saradc, saradc_clk_gate),
        (Crc, crc, crc_clk_gate),
        (Aes, aes, aes_clk_gate),
    }

    #[inline]
    /// Set a device to be on or off.
    pub fn set_enabled(&mut self, dev: Dev, enabled: bool) -> &mut Self {
        if enabled {
            self.enable(dev)
        } else {
            self.disable(dev)
        }
    }
}
