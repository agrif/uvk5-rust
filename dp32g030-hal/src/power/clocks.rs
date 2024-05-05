use crate::pac;

// used internally to cart around source frequencies
#[derive(Debug)]
struct SourceFreqs {
    rchf_high: u32,
    rclf: u32,
    xtal: u32,
}

/// Choices for ADC sample clock, dividing the system clock.
pub type SaradcSel = pac::syscon::clk_sel::SARADC_SMPL_CLK_SEL_W_A;

#[inline(always)]
fn saradc_div(d: SaradcSel) -> u32 {
    // cheating a bit to avoid a match
    // 0 -> 1, 1 -> 2, 2 -> 4, 3 -> 8
    1 << (d as u32)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Choices for RTC clock.
pub enum RtcSel {
    /// Internal RCLF oscillator, 32.768 KHz.
    Rclf,
    /// External XTAL port, 32.768 KHz.
    Xtal,
}

impl RtcSel {
    #[inline(always)]
    fn freq(&self, freqs: &SourceFreqs) -> u32 {
        match self {
            Self::Xtal => freqs.xtal,
            Self::Rclf => freqs.rclf,
        }
    }

    #[inline(always)]
    fn xtal(&self) -> bool {
        match self {
            Self::Xtal => true,
            Self::Rclf => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Choices for system clock.
pub enum SysSel {
    /// Internal RCHF oscillator, 48MHz.
    Rchf48,
    /// Internal RCHF oscillator, 24MHz.
    Rchf24,
    /// Clock divider output.
    Div(DivSel, SrcSel),
}

impl SysSel {
    #[inline(always)]
    fn rchf(&self) -> Option<bool> {
        match self {
            Self::Rchf24 => Some(false),
            Self::Rchf48 => Some(true),
            Self::Div(_, src_sel) => src_sel.rchf(),
        }
    }

    #[inline(always)]
    fn xtah(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Div(_, src_sel) => src_sel.xtah(),
        }
    }

    #[inline(always)]
    fn xtal(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Div(_, src_sel) => src_sel.xtal(),
        }
    }

    #[inline(always)]
    fn freq(&self, freqs: &SourceFreqs) -> u32 {
        match self {
            Self::Rchf24 => freqs.rchf_high / 2,
            Self::Rchf48 => freqs.rchf_high,
            Self::Div(d, src_sel) => src_sel.freq(freqs) / div_div(*d),
        }
    }
}

/// Choices for clock divider amount.
pub type DivSel = pac::syscon::clk_sel::DIV_CLK_SEL_A;

#[inline(always)]
fn div_div(d: DivSel) -> u32 {
    // cheating a bit to avoid a match
    // 0 -> 1, 1 -> 2, 2 -> 4, 3 -> 8
    1 << (d as u32)
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Choices clock divider input.
pub enum SrcSel {
    /// Internal RCHF oscillator, 48MHz.
    Rchf48,
    /// Internal RCHF oscillator, 24MHz.
    Rchf24,
    /// PLL output.
    Pll(PllSel, PllN, PllM),
    /// External XTAH input, 4-32MHz.
    Xtah(u32),
    /// External XTAL input, 32.768kHz.
    Xtal,
    /// Internal RCLF oscillator, 32.768kHz.
    Rclf,
}

impl SrcSel {
    #[inline(always)]
    fn rchf(&self) -> Option<bool> {
        match self {
            Self::Rchf24 => Some(false),
            Self::Rchf48 => Some(true),
            Self::Pll(pll_sel, _, _) => pll_sel.rchf(),
            Self::Xtah(_) => None,
            Self::Xtal => None,
            Self::Rclf => None,
        }
    }

    #[inline(always)]
    fn xtah(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Pll(pll_sel, _, _) => pll_sel.xtah(),
            Self::Xtah(_) => true,
            Self::Xtal => false,
            Self::Rclf => false,
        }
    }

    #[inline(always)]
    fn xtal(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Pll(_, _, _) => false,
            Self::Xtah(_) => false,
            Self::Xtal => true,
            Self::Rclf => false,
        }
    }

    #[inline(always)]
    fn freq(&self, freqs: &SourceFreqs) -> u32 {
        match self {
            Self::Rchf24 => freqs.rchf_high / 2,
            Self::Rchf48 => freqs.rchf_high,
            // overflow safety: u32 can hold up to 80x RCHF, more than we need
            Self::Pll(pll_sel, n, m) => pll_sel.freq(freqs) * pll_n(*n) / pll_m(*m),
            Self::Xtah(f) => *f,
            Self::Xtal => freqs.xtal,
            Self::Rclf => freqs.rclf,
        }
    }
}

/// PLL numerator.
pub type PllN = pac::syscon::pll_ctrl::PLL_N_A;

#[inline(always)]
fn pll_n(n: PllN) -> u32 {
    // cheating, to avoid a huge match table
    // 0 -> 2, 1 -> 4, 2 -> 6, etc.
    2 * (n as u32 + 1)
}

/// PLL denominator.
pub type PllM = pac::syscon::pll_ctrl::PLL_M_A;

#[inline(always)]
fn pll_m(m: PllM) -> u32 {
    // cheating, to avoid a huge match table
    // 0 -> 1, 1 -> 2, 2 -> 3, etc.
    m as u32 + 1
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Choices for PLL input clock.
pub enum PllSel {
    /// Internal RCHF oscillator, 48MHz.
    Rchf48,
    /// Internal RCHF oscillator, 24MHz.
    Rchf24,
    /// External XTAH input, 4-32MHz.
    Xtah(u32),
}

impl PllSel {
    #[inline(always)]
    fn rchf(&self) -> Option<bool> {
        match self {
            Self::Rchf24 => Some(false),
            Self::Rchf48 => Some(true),
            Self::Xtah(_) => None,
        }
    }

    #[inline(always)]
    fn xtah(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Xtah(_) => true,
        }
    }

    #[inline(always)]
    fn freq(&self, freqs: &SourceFreqs) -> u32 {
        match self {
            Self::Rchf24 => freqs.rchf_high / 2,
            Self::Rchf48 => freqs.rchf_high,
            Self::Xtah(f) => *f,
        }
    }
}

/// Clock configuration.
pub struct ClockConfig {
    syscon: pac::SYSCON,
    pmu: pac::PMU,

    xtal: u32,

    saradc_sample: SaradcSel,
    rtc: RtcSel,
    sys: SysSel,
}

impl core::fmt::Debug for ClockConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("ClockConfig")
            .field("xtal", &self.xtal)
            .field("saradc_sample", &self.saradc_sample)
            .field("rtc", &self.rtc)
            .field("sys", &self.sys)
            .finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ClockConfig {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(
            f,
            "ClockConfig {{xtal: {}, saradc_sample: {}, rtc: {}, sys: {}}}",
            self.xtal,
            self.saradc_sample,
            self.rtc,
            self.sys
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Represents frozen, complete information about system clock frequencies.
pub struct Clocks {
    sys_clk: u32,
    saradc_sample_clk: u32,
    rtc_clk: u32,
    iwdt_clk: u32,
}

impl Clocks {
    /// Get the system clock, in Hz.
    #[inline(always)]
    pub fn sys_clk(&self) -> u32 {
        self.sys_clk
    }

    /// Get the ADC sample clock, in Hz.
    #[inline(always)]
    pub fn saradc_sample_clk(&self) -> u32 {
        self.saradc_sample_clk
    }

    /// Get the RTC clock, in Hz.
    #[inline(always)]
    pub fn rtc_clk(&self) -> u32 {
        self.rtc_clk
    }

    /// Get the IWDT clock, in Hz.
    #[inline(always)]
    pub fn iwdt_clk(&self) -> u32 {
        self.iwdt_clk
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

            xtal: 32_768,

            saradc_sample: SaradcSel::Div1,
            rtc: RtcSel::Rclf,
            sys: SysSel::Rchf24,
        }
    }

    /// Set the ADC sample divisor applied to the system clock.
    #[inline(always)]
    pub fn saradc_sample(self, saradc_sample: SaradcSel) -> Self {
        Self {
            saradc_sample,
            ..self
        }
    }

    /// Set the RTC clock source.
    #[inline(always)]
    pub fn rtc(self, rtc: RtcSel) -> Self {
        Self { rtc, ..self }
    }

    /// Set the RTC clock to internal.
    #[inline(always)]
    pub fn rtc_internal(self) -> Self {
        self.rtc(RtcSel::Rclf)
    }

    /// Set the RTC clock to external.
    #[inline(always)]
    pub fn rtc_external(self) -> Self {
        self.rtc(RtcSel::Xtal)
    }

    /// Set the system clock source.
    #[inline(always)]
    pub fn sys(self, sys: SysSel) -> Self {
        Self { sys, ..self }
    }

    /// Set the system clock to be the 24MHz internal clock.
    #[inline(always)]
    pub fn sys_internal_24mhz(self) -> Self {
        self.sys(SysSel::Rchf24)
    }

    /// Set the system clock to be the 48MHz internal clock.
    #[inline(always)]
    pub fn sys_internal_48mhz(self) -> Self {
        self.sys(SysSel::Rchf48)
    }

    /// Set the system clock to be the 24MHz internal clock with divider.
    #[inline(always)]
    pub fn sys_internal_24mhz_div(self, div: DivSel) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Rchf24))
    }

    /// Set the system clock to be the 48MHz internal clock with divider.
    #[inline(always)]
    pub fn sys_internal_48mhz_div(self, div: DivSel) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Rchf48))
    }

    /// Set the system clock to be the external XTAH clock with divider.
    #[inline(always)]
    pub fn sys_external_div(self, xtah: u32, div: DivSel) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Xtah(xtah)))
    }

    /// Set the system clock to be the 24MHz internal clock with divider and PLL.
    #[inline(always)]
    pub fn sys_internal_24mhz_pll(self, div: DivSel, n: PllN, m: PllM) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Pll(PllSel::Rchf24, n, m)))
    }

    /// Set the system clock to be the 48MHz internal clock with divider and PLL.
    #[inline(always)]
    pub fn sys_internal_48mhz_pll(self, div: DivSel, n: PllN, m: PllM) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Pll(PllSel::Rchf48, n, m)))
    }

    /// Set the system clock to be the external XTAH clock with divider and PLL.
    #[inline(always)]
    pub fn sys_external_pll(self, xtah: u32, div: DivSel, n: PllN, m: PllM) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Pll(PllSel::Xtah(xtah), n, m)))
    }

    /// Override the XTAL external crystal frequency.
    #[inline(always)]
    pub fn xtal(self, xtal: u32) -> Self {
        Self { xtal, ..self }
    }

    /// Freeze the clock configuration and return the clock frequencies.
    #[inline(always)]
    pub fn freeze(self) -> Clocks {
        // This is mission-critical code written by using a machine-translated
        // PDF as reference.

        // here be dragons, that is

        // ok, lets get us running on the internal oscillator safely
        // safety: both all of these bits are valid to set/unset
        unsafe {
            // turn on RCHF at 24MHz
            self.pmu
                .src_cfg()
                .set_bits(|w| w.rchf_fsel().f_24mhz().rchf_en().enabled());

            // switch the system clock to use RCHF
            self.syscon.clk_sel().clear_bits(|w| w.sys_clk_sel().rchf());

            // make sure we're on the new clock before continuing
            cortex_m::asm::dsb();

            // turn off the PLL for sure, in case we configure it later
            self.syscon.pll_ctrl().clear_bits(|w| w.pll_en().disabled());
        }

        // no matter what, we want div_clk_gate off for now
        self.syscon
            .div_clk_gate()
            .write(|w| w.div_clk_gate().disabled());

        // paranoia, make sure everything we've done takes effect
        cortex_m::asm::dsb();

        // now we start setting things
        let rchf = self.sys.rchf();
        let xtah = self.sys.xtah();
        let xtal = self.sys.xtal() || self.rtc.xtal();

        self.pmu.src_cfg().write(|w| {
            // this must remain on for now! we're running on it
            w.rchf_en().enabled();
            // but we can switch to 48MHz if that's requested
            if Some(true) == rchf {
                w.rchf_fsel().f_48mhz();
            } else {
                w.rchf_fsel().f_24mhz();
            }
            // turn on xtal and xtah
            w.xtah_en().bit(xtah);
            w.xtal_en().bit(xtal);
            // select rtc clock
            match self.rtc {
                RtcSel::Rclf => w.rtc_clk_sel().rclf(),
                RtcSel::Xtal => w.rtc_clk_sel().xtal(),
            }
        });

        self.syscon.pll_ctrl().write(|w| {
            if let SysSel::Div(_, SrcSel::Pll(_, ref n, ref m)) = self.sys {
                // we're using the pll. keep it disabled, though, until later
                w.pll_en().disabled();
                w.pll_n().variant(*n);
                w.pll_m().variant(*m);
            }
            w
        });

        self.syscon.clk_sel().write(|w| {
            // this must remain rchf for now, we're running on it!
            w.sys_clk_sel().rchf();

            // set up div, src, pll
            if let SysSel::Div(ref d, ref src) = self.sys {
                // use the divider
                w.div_clk_sel().variant(*d);
                match src {
                    SrcSel::Rchf48 => w.src_clk_sel().rchf(),
                    SrcSel::Rchf24 => w.src_clk_sel().rchf(),
                    SrcSel::Pll(ref pll, _, _) => {
                        w.src_clk_sel().pll();
                        match pll {
                            PllSel::Rchf48 => w.pll_clk_sel_w().rchf(),
                            PllSel::Rchf24 => w.pll_clk_sel_w().rchf(),
                            PllSel::Xtah(_) => w.pll_clk_sel_w().xtah(),
                        }
                    }
                    SrcSel::Xtah(_) => w.src_clk_sel().xtah(),
                    SrcSel::Xtal => w.src_clk_sel().xtal(),
                    SrcSel::Rclf => w.src_clk_sel().rclf(),
                };
            }

            // set up saradc_smpl
            w.saradc_smpl_clk_sel_w().variant(self.saradc_sample)
        });

        // the registers are all configured but:
        // rchf_en, pll_en, sys_clk_sel, div_clk_gate

        // before we continue, if needed, wait for the PLL to lock
        if let SysSel::Div(_, SrcSel::Pll(_, _, _)) = self.sys {
            // make sure our config above takes effect
            cortex_m::asm::dsb();

            // enable the pll
            // safety: setting this bit is ok
            unsafe {
                self.syscon.pll_ctrl().set_bits(|w| w.pll_en().enabled());
            }

            // make sure the pll got the enable
            cortex_m::asm::dsb();

            // wait until PLL locks
            while self.syscon.pll_st().read().pll_lock().is_unlocked() {
                // expected to take 30us
                cortex_m::asm::nop();
            }
        }

        // open div_clk_gate if we need to
        if let SysSel::Div(_, _) = self.sys {
            // make sure our config above takes effect
            cortex_m::asm::dsb();

            self.syscon
                .div_clk_gate()
                .write(|w| w.div_clk_gate().enabled());
        }

        // make sure our config above takes effect
        cortex_m::asm::dsb();

        // ok, pll is locked, we can switch over to our real clock
        self.syscon.clk_sel().modify(|_, w| match self.sys {
            SysSel::Rchf24 => w.sys_clk_sel().rchf(),
            SysSel::Rchf48 => w.sys_clk_sel().rchf(),
            SysSel::Div(_, _) => w.sys_clk_sel().div_clk(),
        });

        // now that we're configured, we can turn off RCHF if needed
        if rchf.is_none() {
            // make sure our config above takes effect
            cortex_m::asm::dsb();

            // safety: clearing this bit is fine, we're not using RCHF
            unsafe {
                self.pmu.src_cfg().clear_bits(|w| w.rchf_en().disabled());
            }
        }

        // we should be all set. now just return the frequencies.
        // first, figure out the reference clocks
        let freqs = {
            let delta = self.syscon.rc_freq_delta().read();

            let rchf_pos = delta.rchf_sig().is_positive();
            let mut rchf = delta.rchf_delta().bits();
            if rchf_pos {
                rchf += 48_000_000;
            } else {
                rchf = 48_000_000 - rchf;
            }

            let rclf_pos = delta.rclf_sig().is_positive();
            let mut rclf = delta.rclf_delta().bits() as u32;
            if rclf_pos {
                rclf += 32_768;
            } else {
                rclf = 32_768 - rclf;
            }

            SourceFreqs {
                rchf_high: rchf,
                rclf,
                xtal: self.xtal,
            }
        };

        let sys_clk = self.sys.freq(&freqs);

        Clocks {
            sys_clk,
            saradc_sample_clk: sys_clk / saradc_div(self.saradc_sample),
            rtc_clk: self.rtc.freq(&freqs),
            iwdt_clk: freqs.rclf,
        }
    }
}
