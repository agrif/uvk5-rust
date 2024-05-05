use crate::pac;

/// Access to the Chip ID.
pub struct ChipId {
    _private: (),
}

impl core::fmt::Debug for ChipId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("ChipId").field(&self.get()).finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ChipId {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "ChipId({})", &self.get());
    }
}

impl ChipId {
    /// safety: this peripheral reads SYSCON.chip_idN()
    #[inline(always)]
    pub(crate) unsafe fn steal() -> Self {
        Self { _private: () }
    }

    #[inline(always)]
    /// Get the Chip ID.
    pub fn get(&self) -> [u32; 4] {
        // safety: we only access chip_id registers, which we own
        let syscon = unsafe { pac::SYSCON::steal() };
        [
            syscon.chip_id0().read().bits(),
            syscon.chip_id1().read().bits(),
            syscon.chip_id2().read().bits(),
            syscon.chip_id3().read().bits(),
        ]
    }
}
