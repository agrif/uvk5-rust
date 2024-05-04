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
    pub fn iter() -> impl Iterator<Item = Self> {
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

    #[inline]
    /// Enable a device.
    pub fn enable(&mut self, dev: Dev) -> &mut Self {
        use Dev::*;

        // safety: setting bits in this register is ok
        unsafe {
            self.syscon.dev_clk_gate().set_bits(|w| match dev {
                GpioA => w.gpioa_clk_gate().enabled(),
                GpioB => w.gpiob_clk_gate().enabled(),
                GpioC => w.gpioc_clk_gate().enabled(),
                I2c0 => w.iic0_clk_gate().enabled(),
                I2c1 => w.iic1_clk_gate().enabled(),
                Uart0 => w.uart0_clk_gate().enabled(),
                Uart1 => w.uart1_clk_gate().enabled(),
                Uart2 => w.uart2_clk_gate().enabled(),
                Spi0 => w.spi0_clk_gate().enabled(),
                Spi1 => w.spi1_clk_gate().enabled(),
                TimerBase0 => w.timer_base0_clk_gate().enabled(),
                TimerBase1 => w.timer_base1_clk_gate().enabled(),
                TimerBase2 => w.timer_base2_clk_gate().enabled(),
                TimerPlus0 => w.timer_plus0_clk_gate().enabled(),
                TimerPlus1 => w.timer_plus1_clk_gate().enabled(),
                PwmBase0 => w.pwm_base0_clk_gate().enabled(),
                PwmBase1 => w.pwm_base1_clk_gate().enabled(),
                PwmPlus0 => w.pwm_plus0_clk_gate().enabled(),
                PwmPlus1 => w.pwm_plus1_clk_gate().enabled(),
                Rtc => w.rtc_clk_gate().enabled(),
                Iwdt => w.iwdt_clk_gate().enabled(),
                Wwdt => w.wwdt_clk_gate().enabled(),
                Saradc => w.saradc_clk_gate().enabled(),
                Crc => w.crc_clk_gate().enabled(),
                Aes => w.aes_clk_gate().enabled(),
            })
        }
        self
    }

    #[inline]
    /// Disable a device.
    pub fn disable(&mut self, dev: Dev) -> &mut Self {
        use Dev::*;

        // safety: clearing bits in this register is ok
        unsafe {
            self.syscon.dev_clk_gate().clear_bits(|w| match dev {
                GpioA => w.gpioa_clk_gate().disabled(),
                GpioB => w.gpiob_clk_gate().disabled(),
                GpioC => w.gpioc_clk_gate().disabled(),
                I2c0 => w.iic0_clk_gate().disabled(),
                I2c1 => w.iic1_clk_gate().disabled(),
                Uart0 => w.uart0_clk_gate().disabled(),
                Uart1 => w.uart1_clk_gate().disabled(),
                Uart2 => w.uart2_clk_gate().disabled(),
                Spi0 => w.spi0_clk_gate().disabled(),
                Spi1 => w.spi1_clk_gate().disabled(),
                TimerBase0 => w.timer_base0_clk_gate().disabled(),
                TimerBase1 => w.timer_base1_clk_gate().disabled(),
                TimerBase2 => w.timer_base2_clk_gate().disabled(),
                TimerPlus0 => w.timer_plus0_clk_gate().disabled(),
                TimerPlus1 => w.timer_plus1_clk_gate().disabled(),
                PwmBase0 => w.pwm_base0_clk_gate().disabled(),
                PwmBase1 => w.pwm_base1_clk_gate().disabled(),
                PwmPlus0 => w.pwm_plus0_clk_gate().disabled(),
                PwmPlus1 => w.pwm_plus1_clk_gate().disabled(),
                Rtc => w.rtc_clk_gate().disabled(),
                Iwdt => w.iwdt_clk_gate().disabled(),
                Wwdt => w.wwdt_clk_gate().disabled(),
                Saradc => w.saradc_clk_gate().disabled(),
                Crc => w.crc_clk_gate().disabled(),
                Aes => w.aes_clk_gate().disabled(),
            })
        }
        self
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

    #[inline]
    /// Get whether a device is on or off.
    pub fn is_enabled(&self, dev: Dev) -> bool {
        use Dev::*;

        let r = self.syscon.dev_clk_gate().read();
        match dev {
            GpioA => r.gpioa_clk_gate().is_enabled(),
            GpioB => r.gpiob_clk_gate().is_enabled(),
            GpioC => r.gpioc_clk_gate().is_enabled(),
            I2c0 => r.iic0_clk_gate().is_enabled(),
            I2c1 => r.iic1_clk_gate().is_enabled(),
            Uart0 => r.uart0_clk_gate().is_enabled(),
            Uart1 => r.uart1_clk_gate().is_enabled(),
            Uart2 => r.uart2_clk_gate().is_enabled(),
            Spi0 => r.spi0_clk_gate().is_enabled(),
            Spi1 => r.spi1_clk_gate().is_enabled(),
            TimerBase0 => r.timer_base0_clk_gate().is_enabled(),
            TimerBase1 => r.timer_base1_clk_gate().is_enabled(),
            TimerBase2 => r.timer_base2_clk_gate().is_enabled(),
            TimerPlus0 => r.timer_plus0_clk_gate().is_enabled(),
            TimerPlus1 => r.timer_plus1_clk_gate().is_enabled(),
            PwmBase0 => r.pwm_base0_clk_gate().is_enabled(),
            PwmBase1 => r.pwm_base1_clk_gate().is_enabled(),
            PwmPlus0 => r.pwm_plus0_clk_gate().is_enabled(),
            PwmPlus1 => r.pwm_plus1_clk_gate().is_enabled(),
            Rtc => r.rtc_clk_gate().is_enabled(),
            Iwdt => r.iwdt_clk_gate().is_enabled(),
            Wwdt => r.wwdt_clk_gate().is_enabled(),
            Saradc => r.saradc_clk_gate().is_enabled(),
            Crc => r.crc_clk_gate().is_enabled(),
            Aes => r.aes_clk_gate().is_enabled(),
        }
    }
}
