//! Interfaces for the internal registers of the BK4819.
//!
#![doc = crate::doc_table::doc_table! {
    "0x00" => {
        /* 0x00 */ Reset, /* 0x01 */, /* 0x02 */ Interrupts, /* 0x03 */,
        /* 0x04 */, /* 0x05 */, /* 0x06 */, /* 0x07 */ CtcControl,
        /* 0x08 */, /* 0x09 */, /* 0x0a */, /* 0x0b */,
        /* 0x0c */, /* 0x0d */, /* 0x0e */, /* 0x0f */,
    },
    "0x10" => {
        /* 0x10 */ AgcGainTable0, /* 0x11 */ AgcGainTable1, /* 0x12 */ AgcGainTable2, /* 0x13 */ AgcGainTable3,
        /* 0x14 */ AgcGainTable4, /* 0x15 */, /* 0x16 */, /* 0x17 */,
        /* 0x18 */, /* 0x19 */, /* 0x1a */, /* 0x1b */,
        /* 0x1c */, /* 0x1d */, /* 0x1e */, /* 0x1f */,
    },
    "0x20" => {
        /* 0x20 */, /* 0x21 */, /* 0x22 */, /* 0x23 */,
        /* 0x24 */, /* 0x25 */, /* 0x26 */, /* 0x27 */,
        /* 0x28 */, /* 0x29 */, /* 0x2a */, /* 0x2b */,
        /* 0x2c */, /* 0x2d */, /* 0x2e */, /* 0x2f */,
    },
    "0x30" => {
        /* 0x30 */, /* 0x31 */, /* 0x32 */, /* 0x33 */ GpioOutput,
        /* 0x34 */, /* 0x35 */, /* 0x36 */ PaControl, /* 0x37 */ PowerControl,
        /* 0x38 */, /* 0x39 */, /* 0x3a */, /* 0x3b */,
        /* 0x3c */, /* 0x3d */, /* 0x3e */, /* 0x3f */,
    },
    "0x40" => {
        /* 0x40 */, /* 0x41 */, /* 0x42 */, /* 0x43 */,
        /* 0x44 */, /* 0x45 */, /* 0x46 */, /* 0x47 */,
        /* 0x48 */, /* 0x49 */, /* 0x4a */, /* 0x4b */,
        /* 0x4c */, /* 0x4d */, /* 0x4e */, /* 0x4f */,
    },
    "0x50" => {
        /* 0x50 */, /* 0x51 */, /* 0x52 */, /* 0x53 */,
        /* 0x54 */, /* 0x55 */, /* 0x56 */, /* 0x57 */,
        /* 0x58 */, /* 0x59 */, /* 0x5a */, /* 0x5b */,
        /* 0x5c */, /* 0x5d */, /* 0x5e */, /* 0x5f */,
    },
    "0x60" => {
        /* 0x60 */, /* 0x61 */, /* 0x62 */, /* 0x63 */,
        /* 0x64 */, /* 0x65 */, /* 0x66 */, /* 0x67 */,
        /* 0x68 */, /* 0x69 */, /* 0x6a */, /* 0x6b */,
        /* 0x6c */, /* 0x6d */, /* 0x6e */, /* 0x6f */,
    },
    "0x70" => {
        /* 0x70 */, /* 0x71 */, /* 0x72 */, /* 0x73 */,
        /* 0x74 */, /* 0x75 */, /* 0x76 */, /* 0x77 */,
        /* 0x78 */, /* 0x79 */, /* 0x7a */, /* 0x7b */,
        /* 0x7c */, /* 0x7d */, /* 0x7e */, /* 0x7f */,
    },
}]

use bitfield_struct::bitfield;

/// A trait describing a register generically.
pub trait Register: Clone + From<u16> + Into<u16> {
    /// The address of this register, 7 bits.
    const ADDRESS: u8;
}

// a helper macro to define a register that's an instance of some common data
macro_rules! instance {
    (addr = $addr:expr, name = $name:ident, inner = $inner:ty, default = $default:expr, doc = $doc:literal, $($with:ident : $withty:ty),* $(,)?) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = concat!("See [", stringify!($inner), "] for more details.")]
        #[doc = concat!("This struct will dereference to [", stringify!($inner), "]")]
        #[doc = "and re-implements all of its `with_*` methods."]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        #[repr(transparent)]
        pub struct $name(pub $inner);

        impl From<u16> for $name {
            fn from(other: u16) -> Self {
                Self(other.into())
            }
        }

        impl From<$name> for u16 {
            fn from(other: $name) -> Self {
                other.0.into()
            }
        }

        impl From<$inner> for $name {
            fn from(other: $inner) -> Self {
                Self(other)
            }
        }

        impl From<$name> for $inner {
            fn from(other: $name) -> Self {
                other.0
            }
        }

        impl core::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self($default.into())
            }
        }

        impl $name {
            /// Creates a new default initialized bitfield.
            pub fn new() -> Self {
                Self($default.into())
            }

            /// Convert from bits.
            pub fn from_bits(bits: u16) -> Self {
                Self(bits.into())
            }

            /// Convert into bits.
            pub fn into_bits(self) -> u16 {
                self.0.into()
            }

            $(
                #[doc = concat!("Call ", stringify!($with), " on the inner bitfield of this register.")]
                pub fn $with(self, value: $withty) -> Self {
                    Self(self.0.$with(value))
                }
            )*
        }

        impl Register for $name {
            const ADDRESS: u8 = $addr;
        }
    };
}

/// 0x00 Reset.
#[cfg_attr(not(feature = "defmt"), bitfield(u16))]
#[cfg_attr(feature = "defmt", bitfield(u16, defmt = true))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reset {
    #[bits(15)]
    __: u16,

    /// Soft reset, set high then low to reset chip.
    pub reset: bool,
}

impl Register for Reset {
    const ADDRESS: u8 = 0x00;
}

/// 0x02 Interrupt status flags.
///
/// Writing any value to this register clears these flags.
#[cfg_attr(not(feature = "defmt"), bitfield(u16))]
#[cfg_attr(feature = "defmt", bitfield(u16, defmt = true))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interrupts {
    __: bool,

    /// FSK Rx sync.
    #[bits(1, access = RO)]
    pub fsk_rx_sync: bool,
    /// Squelch lost.
    #[bits(1, access = RO)]
    pub squelch_lost: bool,
    /// Squelch found.
    #[bits(1, access = RO)]
    pub squelch_found: bool,
    /// Vox lost.
    #[bits(1, access = RO)]
    pub vox_lost: bool,
    /// Vox found.
    #[bits(1, access = RO)]
    pub vox_found: bool,
    /// CTCSS lost.
    #[bits(1, access = RO)]
    pub ctcss_lost: bool,
    /// CTCSS found.
    #[bits(1, access = RO)]
    pub ctcss_found: bool,
    /// CDCSS lost.
    #[bits(1, access = RO)]
    pub cdcss_lost: bool,
    /// CDCSS found.
    #[bits(1, access = RO)]
    pub cdcss_found: bool,
    /// CTCSS/CDCSS tail found.
    #[bits(1, access = RO)]
    pub tail_found: bool,
    /// DTMF/5TONE found.
    #[bits(1, access = RO)]
    pub tone_found: bool,
    /// FSK FIFO almost full.
    #[bits(1, access = RO)]
    pub fsk_fifo_almost_full: bool,
    /// FSK Rx finished.
    #[bits(1, access = RO)]
    pub fsk_rx_finished: bool,
    /// FSK FIFO almost empty.
    #[bits(1, access = RO)]
    pub fsk_fifo_almost_empty: bool,
    /// FSK Tx finished.
    #[bits(1, access = RO)]
    pub fsk_tx_finished: bool,
}

impl Register for Interrupts {
    const ADDRESS: u8 = 0x02;
}

/// 0x07 CTCSS/CDCSS frequency control.
#[cfg_attr(not(feature = "defmt"), bitfield(u16))]
#[cfg_attr(feature = "defmt", bitfield(u16, defmt = true))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CtcControl {
    /// Frequency control word.
    ///
    /// In [CtcMode::Ctc1] or [CtcMode::Cdcss], this should be:
    ///  * freq_hz * 20.64888 for XTAL 13M/26M
    ///  * freq_hz * 20.97152 for XTAL 12.8M/19.2M/25.6M/38.4M
    ///
    /// In [CtcMode::Ctc2], this should be:
    /// * 25391 / freq_hz for XTAL 13M/26M or
    /// * 25000 / freq_hz for XTAL 12.8M/19.2M/25.6M/38.4M
    #[bits(13)]
    pub frequency: u16,
    /// CTCSS/CDCSS mode.
    #[bits(3, from = CtcMode::from_bits, into = CtcMode::into_bits)]
    pub mode: Result<CtcMode, u8>,
}

impl Register for CtcControl {
    const ADDRESS: u8 = 0x07;
}

/// CTCSS/CDCSS mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CtcMode {
    /// Normal CTCSS.
    Ctc1 = 0,
    /// 55Hz CTCSS tail detection.
    Ctc2 = 1,
    /// Normal CDCSS.
    Cdcss = 2,
}

impl CtcMode {
    pub const fn into_bits(this: Result<Self, u8>) -> u8 {
        match this {
            Ok(v) => v as u8,
            Err(v) => v,
        }
    }

    pub const fn from_bits(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(Self::Ctc1),
            1 => Ok(Self::Ctc2),
            2 => Ok(Self::Cdcss),
            _ => Err(v),
        }
    }
}

/// 0x10 - 0x14 AGC gain table entry.
///
/// Index Max->Min is 3, 2, 1, 0, -1.
///
/// No I don't know what that means either.
#[cfg_attr(not(feature = "defmt"), bitfield(u16))]
#[cfg_attr(feature = "defmt", bitfield(u16, defmt = true))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgcGainTable {
    #[bits(3)]
    pub pga: u8,

    #[bits(2)]
    pub mixer: u8,

    #[bits(3)]
    pub lna: u8,

    #[bits(2)]
    pub lna_short: u8,

    #[bits(6)]
    __: u8,
}

instance!(
    addr = 0x10,
    name = AgcGainTable0,
    inner = AgcGainTable,
    default = 0x0038,
    doc = "0x10 AGC gain table \\[0\\].",
    with_pga : u8,
    with_mixer: u8,
    with_lna: u8,
    with_lna_short: u8,
);

instance!(
    addr = 0x11,
    name = AgcGainTable1,
    inner = AgcGainTable,
    default = 0x025a,
    doc = "0x11 AGC gain table \\[1\\].",
    with_pga : u8,
    with_mixer: u8,
    with_lna: u8,
    with_lna_short: u8,
);

instance!(
    addr = 0x12,
    name = AgcGainTable2,
    inner = AgcGainTable,
    default = 0x037b,
    doc = "0x12 AGC gain table \\[2\\].",
    with_pga : u8,
    with_mixer: u8,
    with_lna: u8,
    with_lna_short: u8,
);

instance!(
    addr = 0x13,
    name = AgcGainTable3,
    inner = AgcGainTable,
    default = 0x03de,
    doc = "0x13 AGC gain table \\[3\\].",
    with_pga : u8,
    with_mixer: u8,
    with_lna: u8,
    with_lna_short: u8,
);

instance!(
    addr = 0x14,
    name = AgcGainTable4,
    inner = AgcGainTable,
    default = 0x0000,
    doc = "0x14 AGC gain table \\[4\\].",
    with_pga : u8,
    with_mixer: u8,
    with_lna: u8,
    with_lna_short: u8,
);

/// 0x33 GPIO output.
#[cfg_attr(not(feature = "defmt"), bitfield(u16))]
#[cfg_attr(feature = "defmt", bitfield(u16, defmt = true))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpioOutput {
    /// GPIO6 output state.
    pub state6: bool,

    /// GPIO5 output state.
    pub state5: bool,

    /// GPIO4 output state.
    pub state4: bool,

    /// GPIO3 output state.
    pub state3: bool,

    /// GPIO2 output state.
    pub state2: bool,

    /// GPIO1 output state.
    pub state1: bool,

    /// GPIO0 output state.
    pub state0: bool,

    #[bits(1)]
    __: u8,

    /// GPIO6 output disabled.
    #[bits(1, default = true)]
    pub disabled6: bool,

    /// GPIO5 output disabled.
    #[bits(1, default = true)]
    pub disabled5: bool,

    /// GPIO4 output disabled.
    #[bits(1, default = true)]
    pub disabled4: bool,

    /// GPIO3 output disabled.
    #[bits(1, default = true)]
    pub disabled3: bool,

    /// GPIO2 output disabled.
    #[bits(1, default = true)]
    pub disabled2: bool,

    /// GPIO1 output disabled.
    #[bits(1, default = true)]
    pub disabled1: bool,

    /// GPIO0 output disabled.
    #[bits(1, default = true)]
    pub disabled0: bool,

    #[bits(1)]
    __: u8,
}

impl Register for GpioOutput {
    const ADDRESS: u8 = 0x33;
}

impl GpioOutput {
    /// Is this GPIO pin output enabled?
    pub fn enabled(&self, pin: u8) -> bool {
        match pin {
            0 => !self.disabled0(),
            1 => !self.disabled1(),
            2 => !self.disabled2(),
            3 => !self.disabled3(),
            4 => !self.disabled4(),
            5 => !self.disabled5(),
            6 => !self.disabled6(),
            _ => false,
        }
    }

    /// Set a GPIO pin output enabled.
    pub fn set_enabled(&mut self, pin: u8, enabled: bool) {
        match pin {
            0 => self.set_disabled0(!enabled),
            1 => self.set_disabled1(!enabled),
            2 => self.set_disabled2(!enabled),
            3 => self.set_disabled3(!enabled),
            4 => self.set_disabled4(!enabled),
            5 => self.set_disabled5(!enabled),
            6 => self.set_disabled6(!enabled),
            _ => {}
        }
    }

    /// Modify self to enable a given GPIO pin output.
    pub fn with_enabled(self, pin: u8, enabled: bool) -> Self {
        match pin {
            0 => self.with_disabled0(!enabled),
            1 => self.with_disabled1(!enabled),
            2 => self.with_disabled2(!enabled),
            3 => self.with_disabled3(!enabled),
            4 => self.with_disabled4(!enabled),
            5 => self.with_disabled5(!enabled),
            6 => self.with_disabled6(!enabled),
            _ => self,
        }
    }

    /// Is a GPIO pin output set high?
    pub fn state(&self, pin: u8) -> bool {
        match pin {
            0 => self.state0(),
            1 => self.state1(),
            2 => self.state2(),
            3 => self.state3(),
            4 => self.state4(),
            5 => self.state5(),
            6 => self.state6(),
            _ => false,
        }
    }

    /// Set a GPIO pin output high.
    pub fn set_state(&mut self, pin: u8, state: bool) {
        match pin {
            0 => self.set_state0(state),
            1 => self.set_state1(state),
            2 => self.set_state2(state),
            3 => self.set_state3(state),
            4 => self.set_state4(state),
            5 => self.set_state5(state),
            6 => self.set_state6(state),
            _ => {}
        }
    }

    /// Modify self to set a given GPIO pin output high.
    pub fn with_state(self, pin: u8, state: bool) -> Self {
        match pin {
            0 => self.with_state0(state),
            1 => self.with_state1(state),
            2 => self.with_state2(state),
            3 => self.with_state3(state),
            4 => self.with_state4(state),
            5 => self.with_state5(state),
            6 => self.with_state6(state),
            _ => self,
        }
    }
}

/// 0x36 Power amplifier control.
#[cfg_attr(not(feature = "defmt"), bitfield(u16))]
#[cfg_attr(feature = "defmt", bitfield(u16, defmt = true))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PaControl {
    /// PA Gain 2 tuning, 0b111 (max) to 0b000 (min).
    #[bits(3, default = 0b111)]
    pub gain2: u8,

    /// PA Gain 1 tuning, 0b111 (max) to 0b000 (min).
    #[bits(3, default = 0b111)]
    pub gain1: u8,

    #[bits(1)]
    __: u8,

    /// PA CTL output enable.
    pub pactl_enable: bool,

    /// PA bias output, 0x00 (0V) to 0xff (3.2V).
    pub bias: u8,
}

impl Register for PaControl {
    const ADDRESS: u8 = 0x36;
}

/// 0x37 Power save settings.
#[cfg_attr(not(feature = "defmt"), bitfield(u16))]
#[cfg_attr(feature = "defmt", bitfield(u16, defmt = true))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PowerControl {
    /// Band-gap enable.
    pub band_gap_enable: bool,

    /// XTAL enable.
    pub xtal_enable: bool,

    /// DSP enable.
    pub dsp_enable: bool,

    /// Unknown (reserved).
    pub unknown_b3: bool,

    /// PLL LDO bypass.
    pub pll_ldo_bypass: bool,

    /// FF LDO bypass.
    pub rf_ldo_bypass: bool,

    /// VCO LDO bypass.
    pub vco_ldo_bypass: bool,

    /// ANA LDO bypass.
    pub ana_ldo_bypass: bool,

    /// PLL LDO voltage selection.
    #[bits(1, default = LdoVoltage::V2_7)]
    pub pll_ldo_select: LdoVoltage,

    /// RF LDO voltage selection.
    #[bits(1, default = LdoVoltage::V2_7)]
    pub rf_ldo_select: LdoVoltage,

    /// VCO LDO voltage selection.
    #[bits(1, default = LdoVoltage::V2_7)]
    pub vco_ldo_select: LdoVoltage,

    /// ANA LDO voltage selection.
    #[bits(1, default = LdoVoltage::V2_7)]
    pub ana_ldo_select: LdoVoltage,

    /// DSP voltage setting.
    #[bits(3, default = 0b001)]
    pub dsp_voltage: u8,

    #[bits(1)]
    __: u8,
}

/// LDO voltage selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum LdoVoltage {
    /// 2.4 volts.
    V2_4 = 0,
    /// 2.7 volts.
    V2_7 = 1,
}

impl LdoVoltage {
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(v: u8) -> Self {
        match v {
            0 => Self::V2_4,
            _ => Self::V2_7,
        }
    }
}

impl Register for PowerControl {
    const ADDRESS: u8 = 0x37;
}

#[cfg(test)]
mod test {
    use super::*;

    // helper to read off bit slices from the datasheet and assert them
    macro_rules! check_bits {
        ($name:ty { $($field:ident[$($spec:tt)*] $(= $default:expr)?),*$(,)? }) => {
            $(
                {
                    paste::paste! {
                        let found = ($name::[<$field:upper _OFFSET>], $name::[<$field:upper _BITS>]);
                    }
                    let expected = check_bits!(@spechelper, [$($spec)*]);
                    assert_eq!(expected, found, "on {}::{}, expected (len, size) of {:?}, found {:?}", stringify!($name), stringify!($field), expected, found);
                    $(
                        let expected = $default;
                        let found = <$name>::new().$field();
                        assert_eq!(expected, found, "on {}::{}, expected default {:?}, found {:?}", stringify!($name), stringify!($field), expected, found);
                    )?
                }
            )*
        };
        (@spechelper, [$bit:literal]) => {
            // start, len
            ($bit, 1)
        };
        (@spechelper, [$end:literal : $start:literal]) => {
            // start, len
            ($start, 1 + $end - $start)
        };
    }

    #[test]
    fn r00_reset() {
        assert_eq!(Reset::ADDRESS, 0x00);
        check_bits!(Reset { reset[15] = false });
        assert_eq!(0x8000, Reset::new().with_reset(true).into_bits());
    }

    #[test]
    fn r02_interrupts() {
        assert_eq!(Interrupts::ADDRESS, 0x02);
        check_bits!(Interrupts {
            fsk_tx_finished[15] = false,
            fsk_fifo_almost_empty[14] = false,
            fsk_rx_finished[13] = false,
            fsk_fifo_almost_full[12] = false,
            tone_found[11] = false,
            tail_found[10] = false,
            cdcss_found[9] = false,
            cdcss_lost[8] = false,
            ctcss_found[7] = false,
            ctcss_lost[6] = false,
            vox_found[5] = false,
            vox_lost[4] = false,
            squelch_found[3] = false,
            squelch_lost[2] = false,
            fsk_rx_sync[1] = false,
        });
    }

    #[test]
    fn r07_ctc_control() {
        assert_eq!(CtcControl::ADDRESS, 0x07);
        check_bits!(CtcControl {
            mode[15:13],
            frequency[12:0],
        });
    }

    #[test]
    fn r10_r14_agc_gain_table() {
        assert_eq!(AgcGainTable0::ADDRESS, 0x10);
        assert_eq!(AgcGainTable1::ADDRESS, 0x11);
        assert_eq!(AgcGainTable2::ADDRESS, 0x12);
        assert_eq!(AgcGainTable3::ADDRESS, 0x13);
        assert_eq!(AgcGainTable4::ADDRESS, 0x14);

        check_bits!(AgcGainTable {
            pga[2:0],
            mixer[4:3],
            lna[7:5],
            lna_short[9:8],
        });

        assert_eq!(AgcGainTable0::new().into_bits(), 0x0038);
        assert_eq!(AgcGainTable1::new().into_bits(), 0x025a);
        assert_eq!(AgcGainTable2::new().into_bits(), 0x037b);
        assert_eq!(AgcGainTable3::new().into_bits(), 0x03de);
        assert_eq!(AgcGainTable4::new().into_bits(), 0x0000);

        assert_eq!(
            0x007a,
            AgcGainTable0::new()
                .with_pga(0b010)
                .with_mixer(0b11)
                .with_lna(0b011)
                .with_lna_short(0b00)
                .into_bits()
        );

        assert_eq!(
            0x027b,
            AgcGainTable1::new()
                .with_pga(0b011)
                .with_mixer(0b11)
                .with_lna(0b011)
                .with_lna_short(0b10)
                .into_bits()
        );

        assert_eq!(
            0x037b,
            AgcGainTable2::new()
                .with_pga(0b011)
                .with_mixer(0b11)
                .with_lna(0b011)
                .with_lna_short(0b11)
                .into_bits()
        );

        assert_eq!(
            0x03be,
            AgcGainTable3::new()
                .with_pga(0b110)
                .with_mixer(0b11)
                .with_lna(0b101)
                .with_lna_short(0b11)
                .into_bits()
        );

        assert_eq!(
            0x0019,
            AgcGainTable0::new()
                .with_pga(0b001)
                .with_mixer(0b11)
                .with_lna(0b000)
                .with_lna_short(0b00)
                .into_bits()
        );
    }

    #[test]
    fn r33_gpio_output() {
        assert_eq!(GpioOutput::ADDRESS, 0x33);
        check_bits!(GpioOutput {
            disabled0[14] = true,
            disabled1[13] = true,
            disabled2[12] = true,
            disabled3[11] = true,
            disabled4[10] = true,
            disabled5[9] = true,
            disabled6[8] = true,
            state0[6] = false,
            state1[5] = false,
            state2[4] = false,
            state3[3] = false,
            state4[2] = false,
            state5[1] = false,
            state6[0] = false,
        });
    }

    #[test]
    fn r36_pa_control() {
        assert_eq!(PaControl::ADDRESS, 0x36);
        check_bits!(PaControl {
            bias[15:8] = 0x00,
            pactl_enable[7] = false,
            gain1[5:3] = 0b111,
            gain2[2:0] = 0b111,
        });

        assert_eq!(
            0x0022,
            PaControl::new()
                .with_gain2(0b010)
                .with_gain1(0b100)
                .into_bits()
        );
    }

    #[test]
    fn r37_power_control() {
        assert_eq!(PowerControl::ADDRESS, 0x37);
        check_bits!(PowerControl {
            dsp_voltage[14:12] = 0b001,
            ana_ldo_select[11] = LdoVoltage::V2_7,
            vco_ldo_select[10] = LdoVoltage::V2_7,
            rf_ldo_select[9] = LdoVoltage::V2_7,
            pll_ldo_select[8] = LdoVoltage::V2_7,
            ana_ldo_bypass[7] = false,
            vco_ldo_bypass[6] = false,
            rf_ldo_bypass[5] = false,
            pll_ldo_bypass[4] = false,
            unknown_b3[3] = false,
            dsp_enable[2] = false,
            xtal_enable[1] = false,
            band_gap_enable[0] = false,
        });

        assert_eq!(
            0x1d0f,
            PowerControl::new()
                .with_band_gap_enable(true)
                .with_xtal_enable(true)
                .with_dsp_enable(true)
                .with_unknown_b3(true)
                .with_rf_ldo_select(LdoVoltage::V2_4)
                .into_bits()
        );
    }
}
