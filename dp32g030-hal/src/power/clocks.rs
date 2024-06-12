use crate::pac;

use crate::gpio::alt::{xtah, xtal};
use crate::time::{Hertz, RateExtU32};

/// Holding this token means the XTAL port is configured and
/// has a known frequency.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XtalPort {
    _private: (),
}

impl core::fmt::Debug for XtalPort {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("XtalPort").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for XtalPort {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "XtalPort");
    }
}

/// Holding this token means the XTAH port is configured and
/// has a known frequency.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XtahPort {
    _private: (),
}

impl core::fmt::Debug for XtahPort {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("XtahPort").finish()
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for XtahPort {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "XtahPort");
    }
}

// used internally to cart around source frequencies
#[derive(Debug)]
struct SourceFreqs {
    rchf_high: Hertz,
    rclf: Hertz,
    xtal: Option<Hertz>, // is_some() if XtalPort is held
    xtah: Option<Hertz>, // is_some() if XtahPort is held
}

/// Choices for ADC sample clock, dividing the system clock.
pub type SaradcSel = pac::syscon::clk_sel::SARADC_SMPL_CLK_SEL_W_A;

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
    Xtal(XtalPort),
}

impl RtcSel {
    fn freq(&self, freqs: &SourceFreqs) -> Hertz {
        match self {
            // unwrap: we have an XtalPort token
            Self::Xtal(_) => freqs.xtal.unwrap(),
            Self::Rclf => freqs.rclf,
        }
    }

    fn xtal(&self) -> bool {
        match self {
            Self::Xtal(_) => true,
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
    fn rchf(&self) -> Option<bool> {
        match self {
            Self::Rchf24 => Some(false),
            Self::Rchf48 => Some(true),
            Self::Div(_, src_sel) => src_sel.rchf(),
        }
    }

    fn xtah(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Div(_, src_sel) => src_sel.xtah(),
        }
    }

    fn xtal(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Div(_, src_sel) => src_sel.xtal(),
        }
    }

    fn freq(&self, freqs: &SourceFreqs) -> Hertz {
        match self {
            Self::Rchf24 => freqs.rchf_high / 2,
            Self::Rchf48 => freqs.rchf_high,
            Self::Div(d, src_sel) => src_sel.freq(freqs) / div_div(*d),
        }
    }
}

/// Choices for clock divider amount.
pub type DivSel = pac::syscon::clk_sel::DIV_CLK_SEL_A;

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
    Xtah(XtahPort),
    /// External XTAL input, 32.768kHz.
    Xtal(XtalPort),
    /// Internal RCLF oscillator, 32.768kHz.
    Rclf,
}

impl SrcSel {
    fn rchf(&self) -> Option<bool> {
        match self {
            Self::Rchf24 => Some(false),
            Self::Rchf48 => Some(true),
            Self::Pll(pll_sel, _, _) => pll_sel.rchf(),
            Self::Xtah(_) => None,
            Self::Xtal(_) => None,
            Self::Rclf => None,
        }
    }

    fn xtah(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Pll(pll_sel, _, _) => pll_sel.xtah(),
            Self::Xtah(_) => true,
            Self::Xtal(_) => false,
            Self::Rclf => false,
        }
    }

    fn xtal(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Pll(_, _, _) => false,
            Self::Xtah(_) => false,
            Self::Xtal(_) => true,
            Self::Rclf => false,
        }
    }

    fn freq(&self, freqs: &SourceFreqs) -> Hertz {
        match self {
            Self::Rchf24 => freqs.rchf_high / 2,
            Self::Rchf48 => freqs.rchf_high,
            // overflow safety: u32 can hold up to 80x RCHF, more than we need
            Self::Pll(pll_sel, n, m) => pll_sel.freq(freqs) * pll_n(*n) / pll_m(*m),
            // unwrap: we have an XtahPort token
            Self::Xtah(_) => freqs.xtah.unwrap(),
            // unwrap: we have an XtalPort token
            Self::Xtal(_) => freqs.xtal.unwrap(),
            Self::Rclf => freqs.rclf,
        }
    }
}

/// PLL numerator.
pub type PllN = pac::syscon::pll_ctrl::PLL_N_A;

fn pll_n(n: PllN) -> u32 {
    // cheating, to avoid a huge match table
    // 0 -> 2, 1 -> 4, 2 -> 6, etc.
    2 * (n as u32 + 1)
}

/// PLL denominator.
pub type PllM = pac::syscon::pll_ctrl::PLL_M_A;

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
    Xtah(XtahPort),
}

impl PllSel {
    fn rchf(&self) -> Option<bool> {
        match self {
            Self::Rchf24 => Some(false),
            Self::Rchf48 => Some(true),
            Self::Xtah(_) => None,
        }
    }

    fn xtah(&self) -> bool {
        match self {
            Self::Rchf24 => false,
            Self::Rchf48 => false,
            Self::Xtah(_) => true,
        }
    }

    fn freq(&self, freqs: &SourceFreqs) -> Hertz {
        match self {
            Self::Rchf24 => freqs.rchf_high / 2,
            Self::Rchf48 => freqs.rchf_high,
            // unwrap: we have an XtahPort token
            Self::Xtah(_) => freqs.xtah.unwrap(),
        }
    }
}

/// Clock configuration.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ClockConfig {
    xtal: Option<Hertz>,
    xtah: Option<Hertz>,

    saradc_sample: SaradcSel,
    rtc: RtcSel,
    sys: SysSel,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Represents frozen, complete information about system clock frequencies.
pub struct Clocks {
    sys_clk: Hertz,
    saradc_sample_clk: Hertz,
    rtc_clk: Hertz,
    iwdt_clk: Hertz,
}

impl Clocks {
    /// Get the system clock, in Hz.
    pub fn sys_clk(&self) -> Hertz {
        self.sys_clk
    }

    /// Get the ADC sample clock, in Hz.
    pub fn saradc_sample_clk(&self) -> Hertz {
        self.saradc_sample_clk
    }

    /// Get the RTC clock, in Hz.
    pub fn rtc_clk(&self) -> Hertz {
        self.rtc_clk
    }

    /// Get the IWDT clock, in Hz.
    pub fn iwdt_clk(&self) -> Hertz {
        self.iwdt_clk
    }
}

impl ClockConfig {
    /// # Safety:
    /// This peripheral reads and writes:
    ///  * `SYSCON`: `clk_sel`, `div_clk_gate`, `rc_freq_delta`, `pll_ctrl`, `pll_st`
    ///  * `PMU`: `src_cfg`
    pub(crate) unsafe fn steal() -> Self {
        Self {
            xtal: None,
            xtah: None,

            saradc_sample: SaradcSel::Div1,
            rtc: RtcSel::Rclf,
            sys: SysSel::Rchf24,
        }
    }

    /// Set the ADC sample divisor applied to the system clock.
    pub fn saradc_sample(self, saradc_sample: SaradcSel) -> Self {
        Self {
            saradc_sample,
            ..self
        }
    }

    /// Set the RTC clock source.
    pub fn rtc(self, rtc: RtcSel) -> Self {
        Self { rtc, ..self }
    }

    /// Set the RTC clock to internal.
    pub fn rtc_internal(self) -> Self {
        self.rtc(RtcSel::Rclf)
    }

    /// Set the RTC clock to external.
    pub fn rtc_external(self, xtal: XtalPort) -> Self {
        self.rtc(RtcSel::Xtal(xtal))
    }

    /// Set the system clock source.
    pub fn sys(self, sys: SysSel) -> Self {
        Self { sys, ..self }
    }

    /// Set the system clock to be the 24MHz internal clock.
    pub fn sys_internal_24mhz(self) -> Self {
        self.sys(SysSel::Rchf24)
    }

    /// Set the system clock to be the 48MHz internal clock.
    pub fn sys_internal_48mhz(self) -> Self {
        self.sys(SysSel::Rchf48)
    }

    /// Set the system clock to be the 24MHz internal clock with divider.
    pub fn sys_internal_24mhz_div(self, div: DivSel) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Rchf24))
    }

    /// Set the system clock to be the 48MHz internal clock with divider.
    pub fn sys_internal_48mhz_div(self, div: DivSel) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Rchf48))
    }

    /// Set the system clock to be the external XTAH clock with divider.
    pub fn sys_external_div(self, xtah: XtahPort, div: DivSel) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Xtah(xtah)))
    }

    /// Set the system clock to be the 24MHz internal clock with divider and PLL.
    pub fn sys_internal_24mhz_pll(self, div: DivSel, n: PllN, m: PllM) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Pll(PllSel::Rchf24, n, m)))
    }

    /// Set the system clock to be the 48MHz internal clock with divider and PLL.
    pub fn sys_internal_48mhz_pll(self, div: DivSel, n: PllN, m: PllM) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Pll(PllSel::Rchf48, n, m)))
    }

    /// Set the system clock to be the external XTAH clock with divider and PLL.
    pub fn sys_external_pll(self, xtah: XtahPort, div: DivSel, n: PllN, m: PllM) -> Self {
        self.sys(SysSel::Div(div, SrcSel::Pll(PllSel::Xtah(xtah), n, m)))
    }

    /// Use the XTAL port at 32.768kHz.
    pub fn xtal(&mut self, xi: xtal::Xi, xo: xtal::Xo) -> XtalPort {
        self.xtal_with(xi, xo, 32_786.Hz())
    }

    /// Use the XTAL port with a custom frequency.
    pub fn xtal_with(&mut self, _xi: xtal::Xi, _xo: xtal::Xo, xtal: Hertz) -> XtalPort {
        self.xtal = Some(xtal);
        XtalPort { _private: () }
    }

    /// Use the XTAH port with the given frequency.
    pub fn xtah(&mut self, _xi: xtah::Xi, _xo: xtah::Xo, xtah: Hertz) -> XtahPort {
        self.xtah = Some(xtah);
        XtahPort { _private: () }
    }

    /// Freeze the clock configuration and return the clock frequencies.
    pub fn freeze(self) -> Clocks {
        // not strictly needed, as we own all of these registers and
        // should be able to modify them safely. however, this section is
        // indeed quite cricital, and we'd rather not be interrupted.
        critical_section::with(|cs| self.freeze_critical(cs))
    }

    fn freeze_critical(self, _cs: critical_section::CriticalSection) -> Clocks {
        // This is mission-critical code written by using a machine-translated
        // PDF as reference.

        // here be dragons, that is

        // safety: we will only access the clock registers of syscon/pmu
        // which we own
        let syscon = unsafe { pac::SYSCON::steal() };
        let pmu = unsafe { pac::PMU::steal() };

        // ok, lets get us running on the internal oscillator safely

        // turn on RCHF at 24MHz
        pmu.src_cfg()
            .modify(|_r, w| w.rchf_fsel().f_24mhz().rchf_en().enabled());

        // switch the system clock to use RCHF
        syscon.clk_sel().modify(|_r, w| w.sys_clk_sel().rchf());

        // make sure we're on the new clock before continuing
        cortex_m::asm::dsb();

        // turn off the PLL for sure, in case we configure it later
        syscon.pll_ctrl().modify(|_r, w| w.pll_en().disabled());

        // no matter what, we want div_clk_gate off for now
        syscon.div_clk_gate().write(|w| w.div_clk_gate().disabled());

        // paranoia, make sure everything we've done takes effect
        cortex_m::asm::dsb();

        // now we start setting things
        let rchf = self.sys.rchf();
        let xtah = self.sys.xtah();
        let xtal = self.sys.xtal() || self.rtc.xtal();

        pmu.src_cfg().write(|w| {
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
                RtcSel::Xtal(_) => w.rtc_clk_sel().xtal(),
            }
        });

        syscon.pll_ctrl().write(|w| {
            if let SysSel::Div(_, SrcSel::Pll(_, ref n, ref m)) = self.sys {
                // we're using the pll. keep it disabled, though, until later
                w.pll_en().disabled();
                w.pll_n().variant(*n);
                w.pll_m().variant(*m);
            }
            w
        });

        syscon.clk_sel().write(|w| {
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
                    SrcSel::Xtal(_) => w.src_clk_sel().xtal(),
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
            syscon.pll_ctrl().modify(|_r, w| w.pll_en().enabled());

            // make sure the pll got the enable
            cortex_m::asm::dsb();

            // wait until PLL locks
            while syscon.pll_st().read().pll_lock().is_unlocked() {
                // expected to take 30us
                cortex_m::asm::nop();
            }
        }

        // open div_clk_gate if we need to
        if let SysSel::Div(_, _) = self.sys {
            // make sure our config above takes effect
            cortex_m::asm::dsb();

            syscon.div_clk_gate().write(|w| w.div_clk_gate().enabled());
        }

        // make sure our config above takes effect
        cortex_m::asm::dsb();

        // ok, pll is locked, we can switch over to our real clock
        syscon.clk_sel().modify(|_, w| match self.sys {
            SysSel::Rchf24 => w.sys_clk_sel().rchf(),
            SysSel::Rchf48 => w.sys_clk_sel().rchf(),
            SysSel::Div(_, _) => w.sys_clk_sel().div_clk(),
        });

        // now that we're configured, we can turn off RCHF if needed
        if rchf.is_none() {
            // make sure our config above takes effect
            cortex_m::asm::dsb();

            pmu.src_cfg().modify(|_r, w| w.rchf_en().disabled());
        }

        // we should be all set. now just return the frequencies.
        // first, figure out the reference clocks
        let freqs = {
            let delta = syscon.rc_freq_delta().read();

            let rchf_pos = delta.rchf_sig().is_positive();
            let mut rchf = delta.rchf_delta().bits().Hz();
            let rchf_base = 48.MHz();
            if rchf_pos {
                rchf += rchf_base;
            } else {
                rchf = rchf_base - rchf;
            }

            let rclf_pos = delta.rclf_sig().is_positive();
            let mut rclf = (delta.rclf_delta().bits() as u32).Hz();
            let rclf_base = 32_768.Hz();
            if rclf_pos {
                rclf += rclf_base;
            } else {
                rclf = rclf_base - rclf;
            }

            SourceFreqs {
                rchf_high: rchf,
                rclf,
                xtal: self.xtal,
                xtah: self.xtah,
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
