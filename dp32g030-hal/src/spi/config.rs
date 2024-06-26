use crate::power::Gate;

use crate::pac;

use super::{Instance, MasterPort, MasterRx, MasterTx};

/// Wrap an SPI peripheral into a configurator.
pub fn new<Spi>(spi: Spi, gate: Gate<Spi>) -> Config<Spi>
where
    Spi: Instance,
{
    Config::new(spi, gate)
}

/// An SPI configurator.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config<Spi> {
    pub(super) spi: Spi,
}

/// Choices for baud rate divider.
pub type ClockDivider = pac::spi0::cr::SPR_A;

/// Choices for clock phase.
pub type Phase = pac::spi0::cr::CPHA_A;

/// Choices for clock polarity.
pub type Polarity = pac::spi0::cr::CPOL_A;

/// An SPI mode describing clock polarity and phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Mode {
    pub polarity: Polarity,
    pub phase: Phase,
}

impl Mode {
    /// SPI mode 0: CPOL = 0, CPHA = 0.
    pub const MODE_0: Self = Self {
        polarity: Polarity::Cpol0,
        phase: Phase::Cpha0,
    };

    /// SPI mode 1: CPOL = 0, CPHA = 1.
    pub const MODE_1: Self = Self {
        polarity: Polarity::Cpol0,
        phase: Phase::Cpha1,
    };

    /// SPI mode 2: CPOL = 1, CPHA = 0.
    pub const MODE_2: Self = Self {
        polarity: Polarity::Cpol1,
        phase: Phase::Cpha0,
    };

    /// SPI mode 3: CPOL = 1, CPHA = 1.
    pub const MODE_3: Self = Self {
        polarity: Polarity::Cpol1,
        phase: Phase::Cpha1,
    };
}

/// Choices for bit order.
pub type BitOrder = pac::spi0::cr::LSB_A;

impl<Spi> Config<Spi>
where
    Spi: Instance,
{
    /// Wrap an SPI register into a configurator.
    pub fn new(spi: Spi, mut gate: Gate<Spi>) -> Self {
        gate.enable();

        // safety: we now own this spi, we can reset what we want
        spi.cr().reset();
        spi.ie().reset();
        spi.if_().reset();

        Self { spi }
    }

    /// Recover the SPI register from a configurator.
    pub fn free(self) -> (Spi, Gate<Spi>) {
        // safety: we own this peripheral in self, and are dropping self
        unsafe {
            let mut gate = Gate::steal();
            gate.disable();
            (self.spi, gate)
        }
    }

    /// Set the clock divider.
    pub fn divider(self, div: ClockDivider) -> Self {
        self.spi.cr().modify(|_r, w| w.spr().variant(div));
        self
    }

    /// Get the clock divider.
    pub fn get_divider(&self) -> ClockDivider {
        self.spi.cr().read().spr().variant()
    }

    /// Set the clock phase.
    pub fn phase(self, phase: Phase) -> Self {
        self.spi.cr().modify(|_r, w| w.cpha().variant(phase));
        self
    }

    /// Get the clock phase.
    pub fn get_phase(&self) -> Phase {
        self.spi.cr().read().cpha().variant()
    }

    /// Set the clock polarity.
    pub fn polarity(self, polarity: Polarity) -> Self {
        self.spi.cr().modify(|_r, w| w.cpol().variant(polarity));
        self
    }

    /// Get the clock polarity.
    pub fn get_polarity(&self) -> Polarity {
        self.spi.cr().read().cpol().variant()
    }

    /// Set the mode.
    pub fn mode(self, mode: Mode) -> Self {
        self.phase(mode.phase).polarity(mode.polarity)
    }

    /// Get the mode.
    pub fn get_mode(&self) -> Mode {
        Mode {
            phase: self.get_phase(),
            polarity: self.get_polarity(),
        }
    }

    /// Set the bit order.
    pub fn bit_order(self, bit_order: BitOrder) -> Self {
        self.spi.cr().modify(|_r, w| w.lsb().variant(bit_order));
        self
    }

    /// Get the bit order.
    pub fn get_bit_order(&self) -> BitOrder {
        self.spi.cr().read().lsb().variant()
    }

    fn master_mode(self) -> Self {
        self.spi.cr().modify(|_r, w| w.mstr().master());
        self
    }

    /// Get the configured [MasterPort] using the provided pins.
    pub fn master(self, clk: Spi::Clk, miso: Spi::Miso, mosi: Spi::Mosi) -> MasterPort<Spi> {
        MasterPort::new_master(self.master_mode(), clk, miso, mosi)
    }

    /// Get the configured [MasterPort] using the provided pins, with
    /// slave select.
    pub fn master_ssn(
        self,
        clk: Spi::Clk,
        miso: Spi::Miso,
        mosi: Spi::Mosi,
        ssn: Spi::Ssn,
    ) -> MasterPort<Spi, Spi::Ssn> {
        MasterPort::new_master_ssn(self.master_mode(), clk, miso, mosi, ssn)
    }

    /// Get the configured [MasterRx] using the provided pins.
    pub fn master_rx(self, clk: Spi::Clk, miso: Spi::Miso) -> MasterRx<Spi> {
        MasterRx::new_master_rx(self.master_mode(), clk, miso)
    }

    /// Get the configured [MasterRx] using the provided pins, with
    /// slave select.
    pub fn master_rx_ssn(
        self,
        clk: Spi::Clk,
        miso: Spi::Miso,
        ssn: Spi::Ssn,
    ) -> MasterRx<Spi, Spi::Ssn> {
        MasterRx::new_master_rx_ssn(self.master_mode(), clk, miso, ssn)
    }

    /// Get the configured [MasterTx] using the provided pins.
    pub fn master_tx(self, clk: Spi::Clk, mosi: Spi::Mosi) -> MasterTx<Spi> {
        MasterTx::new_master_tx(self.master_mode(), clk, mosi)
    }

    /// Get the configured [MasterTx] using the provided pins, with
    /// slave select.
    pub fn master_tx_ssn(
        self,
        clk: Spi::Clk,
        mosi: Spi::Mosi,
        ssn: Spi::Ssn,
    ) -> MasterTx<Spi, Spi::Ssn> {
        MasterTx::new_master_tx_ssn(self.master_mode(), clk, mosi, ssn)
    }
}
