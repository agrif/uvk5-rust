use crate::pac;

/// Clock configuration.
pub struct ClockConfig {
    syscon: pac::SYSCON,
    pmu: pac::PMU,
}

impl core::fmt::Debug for ClockConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("ClockConfig")
            .field(&self.syscon)
            .field(&self.pmu)
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ClockConfig {
    fn format(&self, f: defmt::Formatter) {
        // FIXME
        defmt::write!(f, "ClockConfig");
    }
}

impl ClockConfig {
    /// safety: this peripheral reads and writes:
    ///  * SYSCON: clk_sel, div_clk_gate, rc_freq_delta, pll_ctrl, pll_st
    ///  * PMU: src_cfg
    #[inline(always)]
    pub(crate) unsafe fn steal() -> Self {
        Self {
            syscon: pac::SYSCON::steal(),
            pmu: pac::PMU::steal(),
        }
    }
}
