//! Interfaces for the internal registers of the BK4819.

use bitfield_struct::bitfield;

/// A trait describing a register generically.
pub trait Register: Clone + From<u16> + Into<u16> {
    /// The address of this register, 7 bits.
    const ADDRESS: u8;
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

/// CTCSS/CDCSS mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

impl Register for CtcControl {
    const ADDRESS: u8 = 0x07;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reset() {
        assert_eq!(0x8000, Reset::new().with_reset(true).into_bits());
    }

    #[test]
    fn interrupts() {
        assert_eq!(0x0000, Interrupts::new().into_bits());

        let i = Interrupts::from_bits(0xaaaa);

        // odd bits
        assert!(i.fsk_rx_sync());
        assert!(i.squelch_found());
        assert!(i.vox_found());
        assert!(i.ctcss_found());
        assert!(i.cdcss_found());
        assert!(i.tone_found());
        assert!(i.fsk_rx_finished());
        assert!(i.fsk_tx_finished());

        // even bits
        assert!(!i.squelch_lost());
        assert!(!i.vox_lost());
        assert!(!i.ctcss_lost());
        assert!(!i.cdcss_lost());
        assert!(!i.tail_found());
        assert!(!i.fsk_fifo_almost_full());
        assert!(!i.fsk_fifo_almost_empty());
    }
}
