use crate::pac;

/// Access to the Chip ID.
pub struct ChipId {
    syscon: pac::SYSCON,
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
        Self {
            syscon: pac::SYSCON::steal(),
        }
    }

    #[inline(always)]
    /// Get the Chip ID.
    pub fn get(&self) -> [u32; 4] {
        [
            self.syscon.chip_id0().read().bits(),
            self.syscon.chip_id1().read().bits(),
            self.syscon.chip_id2().read().bits(),
            self.syscon.chip_id3().read().bits(),
        ]
    }
}
