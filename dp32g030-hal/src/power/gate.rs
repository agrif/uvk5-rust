use crate::pac;

/// A control to power an individual device.
pub struct Gate<Dev> {
    _marker: core::marker::PhantomData<Dev>,
}

// seal for Device trait
trait Sealed {}

/// An unsafe trait for accessing device gates.
#[allow(private_bounds)]
pub trait Device: Sealed {
    /// The name of the device, used in Debug instances.
    const NAME: &'static str;

    /// Write to this device gate.
    ///
    /// # Safety
    /// Writing to this outside of using a Gate instance
    /// can cause the HAL to become out of sync with the device.
    /// In particular, writes to a disabled device will fail silently.
    unsafe fn set_enabled(enabled: bool);

    /// Read this device gate.
    fn is_enabled() -> bool;

    /// Write this device's name using defmt.
    #[cfg(feature = "defmt")]
    fn defmt(f: defmt::Formatter);
}

impl<Dev> core::fmt::Debug for Gate<Dev>
where
    Dev: Device,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("Gate")
            .field(&Dev::NAME)
            .field(&Dev::is_enabled())
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl<Dev> defmt::Format for Gate<Dev>
where
    Dev: Device,
{
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Gate(");
        Dev::defmt(f);
        defmt::write!(f, ", {})", Dev::is_enabled());
    }
}

impl<Dev> Gate<Dev>
where
    Dev: Device,
{
    /// safety: this reads and writes the bit for Dev in dev_clk_gate
    #[inline(always)]
    pub(crate) unsafe fn steal() -> Self {
        Self {
            _marker: Default::default(),
        }
    }

    /// Set this device to be on or off.
    #[inline(always)]
    pub fn set_enabled(&mut self, enabled: bool) {
        // safety: owning self is owning the right to modify this gate
        unsafe { Dev::set_enabled(enabled) }
    }

    /// Set this device to be on.
    #[inline(always)]
    pub fn enable(&mut self) {
        self.set_enabled(true);
    }

    /// Set this device to be off.
    #[inline(always)]
    pub fn disable(&mut self) {
        self.set_enabled(false);
    }

    /// Is this device enabled?
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        Dev::is_enabled()
    }
}

// way too much repitition to not use a macro
macro_rules! dev_gate_impl {
    {$(($dev:ident, $name:ident, $field:ident)),+,} => {
        /// A collection of controls for powering individual devices.
        #[derive(Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct Gates {
            $(pub $name: Gate<pac::$dev>),*
        }

        impl Gates {
            /// safety: this peripheral reads and writes SYSCON.dev_clk_gate()
            #[inline(always)]
            pub(crate) unsafe fn steal() -> Self {
                Self {
                    $($name: Gate::steal()),*
                }
            }
        }

        $(dev_gate_impl!(trait $dev, $field);)+
    };

    // helper to implement the Device trait
    (trait $dev:ident, $field: ident) => {
        impl Sealed for pac::$dev {}

        impl Device for pac::$dev {
            const NAME: &'static str = stringify!($dev);

            #[inline(always)]
            unsafe fn set_enabled(enabled: bool) {
                // safety: we only access our bit in dev_clk_gate, atomically
                let syscon = pac::SYSCON::steal();
                if enabled {
                    syscon.dev_clk_gate().set_bits(|w| w.$field().enabled())
                } else {
                    syscon.dev_clk_gate().clear_bits(|w| w.$field().disabled())
                }
            }

            #[inline(always)]
            fn is_enabled() -> bool {
                // safety: we only read our bit in dev_clk_gate
                unsafe {
                    pac::SYSCON::steal().dev_clk_gate().read().$field().is_enabled()
                }
            }

            #[cfg(feature = "defmt")]
            #[inline(always)]
            fn defmt(f: defmt::Formatter) {
                defmt::write!(f, "{}", stringify!($dev));
            }
        }
    };
}

dev_gate_impl! {
    (GPIOA, gpio_a, gpioa_clk_gate),
    (GPIOB, gpio_b, gpiob_clk_gate),
    (GPIOC, gpio_c, gpioc_clk_gate),
    (IIC0, iic0, iic0_clk_gate),
    (IIC1, iic1, iic1_clk_gate),
    (UART0, uart0, uart0_clk_gate),
    (UART1, uart1, uart1_clk_gate),
    (UART2, uart2, uart2_clk_gate),
    (SPI0, spi0, spi0_clk_gate),
    (SPI1, spi1, spi1_clk_gate),
    (TIMER_BASE0, timer_base0, timer_base0_clk_gate),
    (TIMER_BASE1, timer_base1, timer_base1_clk_gate),
    (TIMER_BASE2, timer_base2, timer_base2_clk_gate),
    (TIMER_PLUS0, timer_plus0, timer_plus0_clk_gate),
    (TIMER_PLUS1, timer_plus1, timer_plus1_clk_gate),
    (PWM_BASE0, pwm_base0, pwm_base0_clk_gate),
    (PWM_BASE1, pwm_base1, pwm_base1_clk_gate),
    (PWM_PLUS0, pwm_plus0, pwm_plus0_clk_gate),
    (PWM_PLUS1, pwm_plus1, pwm_plus1_clk_gate),
    (RTC, rtc, rtc_clk_gate),
    (IWDT, iwdt, iwdt_clk_gate),
    (WWDT, wwdt, wwdt_clk_gate),
    (SARADC, saradc, saradc_clk_gate),
    (CRC, crc, crc_clk_gate),
    (AES128, aes, aes_clk_gate),
}
