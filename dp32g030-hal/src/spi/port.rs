use core::convert::Infallible;

use crate::block;

use super::{Config, Instance};

/// How many bytes there are in an SPI FIFO.
const FIFO_SIZE: u8 = 8;

/// An SPI port, either in master or slave mode.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Port<Spi: Instance, Mode, Miso, Mosi, Ssn> {
    spi: Spi,
    _mode: Mode,
    clk: Spi::Clk,
    miso: Miso,
    mosi: Mosi,
    ssn: Ssn,
}

/// SPI master mode. (type state)
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Master;

/// An SPI port in master mode.
pub type MasterPort<Spi, Ssn = ()> =
    Port<Spi, Master, <Spi as Instance>::Miso, <Spi as Instance>::Mosi, Ssn>;

/// An SPI port in master mode, read only.
pub type MasterRx<Spi, Ssn = ()> = Port<Spi, Master, <Spi as Instance>::Miso, (), Ssn>;

/// An SPI port in master mode, write only.
pub type MasterTx<Spi, Ssn = ()> = Port<Spi, Master, (), <Spi as Instance>::Mosi, Ssn>;

impl<Spi> MasterPort<Spi>
where
    Spi: Instance,
{
    /// Create a port in master mode from the configured SPI.
    pub fn new_master(
        config: Config<Spi>,
        clk: Spi::Clk,
        miso: Spi::Miso,
        mosi: Spi::Mosi,
    ) -> Self {
        Self {
            spi: config.spi,
            _mode: Default::default(),
            clk,
            miso,
            mosi,
            ssn: (),
        }
        .setup()
    }
}

impl<Spi> MasterPort<Spi, Spi::Ssn>
where
    Spi: Instance,
{
    /// Create a port in master mode from the configured SPI, with slave select.
    pub fn new_master_ssn(
        config: Config<Spi>,
        clk: Spi::Clk,
        miso: Spi::Miso,
        mosi: Spi::Mosi,
        ssn: Spi::Ssn,
    ) -> Self {
        Self {
            spi: config.spi,
            _mode: Default::default(),
            clk,
            miso,
            mosi,
            ssn,
        }
        .setup()
    }
}

impl<Spi> MasterRx<Spi>
where
    Spi: Instance,
{
    /// Create a port in read-only master mode from the configured SPI.
    pub fn new_master_rx(config: Config<Spi>, clk: Spi::Clk, miso: Spi::Miso) -> Self {
        Self {
            spi: config.spi,
            _mode: Default::default(),
            clk,
            miso,
            mosi: (),
            ssn: (),
        }
        .setup()
    }
}

impl<Spi> MasterRx<Spi, Spi::Ssn>
where
    Spi: Instance,
{
    /// Create a port in read-only master mode from the configured
    /// SPI, with slave select.
    pub fn new_master_rx_ssn(
        config: Config<Spi>,
        clk: Spi::Clk,
        miso: Spi::Miso,
        ssn: Spi::Ssn,
    ) -> Self {
        Self {
            spi: config.spi,
            _mode: Default::default(),
            clk,
            miso,
            mosi: (),
            ssn,
        }
        .setup()
    }
}

impl<Spi> MasterTx<Spi>
where
    Spi: Instance,
{
    /// Create a port in write-only master mode from the configured SPI.
    pub fn new_master_tx(config: Config<Spi>, clk: Spi::Clk, mosi: Spi::Mosi) -> Self {
        Self {
            spi: config.spi,
            _mode: Default::default(),
            clk,
            miso: (),
            mosi,
            ssn: (),
        }
        .setup()
    }
}

impl<Spi> MasterTx<Spi, Spi::Ssn>
where
    Spi: Instance,
{
    /// Create a port in read-only master mode from the configured
    /// SPI, with slave select.
    pub fn new_master_tx_ssn(
        config: Config<Spi>,
        clk: Spi::Clk,
        mosi: Spi::Mosi,
        ssn: Spi::Ssn,
    ) -> Self {
        Self {
            spi: config.spi,
            _mode: Default::default(),
            clk,
            miso: (),
            mosi,
            ssn,
        }
        .setup()
    }
}

impl<Spi, Mode, Miso, Mosi, Ssn> Port<Spi, Mode, Miso, Mosi, Ssn>
where
    Spi: Instance,
{
    /// Recover the port into a configurator.
    pub fn free(self) -> (Config<Spi>, Spi::Clk, Miso, Mosi, Ssn) {
        self.spi.cr().modify(|_r, w| w.spe().disabled());

        (
            Config { spi: self.spi },
            self.clk,
            self.miso,
            self.mosi,
            self.ssn,
        )
    }
}

impl<Spi, Mode, Miso, Mosi, Ssn> Port<Spi, Mode, Miso, Mosi, Ssn>
where
    Spi: Instance,
{
    fn setup(mut self) -> Self {
        self.spi.cr().modify(|_r, w| w.spe().enabled());
        self.clear();
        self
    }

    /// Clear both FIFOs.
    pub fn clear(&mut self) {
        self.clear_tx();
        self.clear_rx();
    }

    /// Clear the RX FIFO.
    pub fn clear_rx(&mut self) {
        self.spi.cr().modify(|_r, w| w.rf_clr().clear());
    }

    /// Is the RX FIFO full?
    pub fn is_rx_full(&self) -> bool {
        self.spi.fifost().read().rff().bit()
    }

    /// Is the RX FIFO half full?
    pub fn is_rx_half_full(&self) -> bool {
        self.spi.fifost().read().rfhf().bit()
    }

    /// Is the RX FIFO empty?
    pub fn is_rx_empty(&self) -> bool {
        self.spi.fifost().read().rfe().bit()
    }

    /// Get the RX FIFO level, 0 is empty and 8 is full.
    pub fn rx_level(&self) -> u8 {
        match self.spi.fifost().read().rf_level().bits() {
            0 => {
                if self.is_rx_full() {
                    8
                } else {
                    0
                }
            }
            l => l,
        }
    }

    /// Clear the TX FIFO.
    pub fn clear_tx(&mut self) {
        self.spi.cr().modify(|_r, w| w.tf_clr().clear());
    }

    /// Is the TX FIFO full?
    pub fn is_tx_full(&self) -> bool {
        self.spi.fifost().read().tff().bit()
    }

    /// Is the TX FIFO half full?
    pub fn is_tx_half_full(&self) -> bool {
        self.spi.fifost().read().tfhf().bit()
    }

    /// Is the TX FIFO empty?
    pub fn is_tx_empty(&self) -> bool {
        self.spi.fifost().read().tfe().bit()
    }

    /// Get the TX FIFO level, 0 is empty and 8 is full.
    pub fn tx_level(&self) -> u8 {
        match self.spi.fifost().read().tf_level().bits() {
            0 => {
                if self.is_tx_full() {
                    8
                } else {
                    0
                }
            }
            l => l,
        }
    }
}

impl<Spi, Miso, Mosi, Ssn> Port<Spi, Master, Miso, Mosi, Ssn>
where
    Spi: Instance,
{
    /// Read a single byte from SPI.
    ///
    /// To work, this must be preceeded by a [Self::write_one()].
    pub fn read_one(&mut self) -> block::Result<u8, Infallible> {
        if self.is_rx_empty() {
            Err(block::Error::WouldBlock)
        } else {
            Ok(self.spi.rdr().read().data().bits())
        }
    }

    /// Write a single byte to SPI.
    pub fn write_one(&mut self, value: u8) -> block::Result<(), Infallible> {
        if self.is_tx_full() {
            Err(block::Error::WouldBlock)
        } else {
            self.spi.wdr().write(|w| w.data().set(value));
            Ok(())
        }
    }

    /// Flush all pending writes and clear the FIFOs.
    pub fn flush(&mut self) -> block::Result<(), Infallible> {
        if !self.is_tx_empty() {
            Err(block::Error::WouldBlock)
        } else {
            self.clear_rx();
            Ok(())
        }
    }

    /// Write and read to SPI simultaneously.
    ///
    /// If read is shorter than write, discard all incoming bytes
    /// after that point. If write is shorter than read, write 0x00
    /// after the end of write.
    pub fn transfer_iter<'a>(
        &'a mut self,
        mut read: impl Iterator<Item = &'a mut u8>
            + core::iter::ExactSizeIterator
            + core::iter::FusedIterator,
        write: impl Iterator<Item = u8> + core::iter::FusedIterator,
    ) -> Result<(), Infallible> {
        let mut write = write.peekable();
        block::block!(self.flush())?;

        // handle *all* of read first
        let mut remaining_tx = read.len();
        let mut amount = 0;
        while amount > 0 || remaining_tx > 0 {
            // push some bytes into the tx fifo if we can
            while amount < FIFO_SIZE && remaining_tx > 0 {
                match self.write_one(*write.peek().unwrap_or(&0x00)) {
                    Ok(()) => {
                        remaining_tx -= 1;
                        amount += 1;
                        write.next();
                    }
                    Err(block::Error::WouldBlock) => break,
                    Err(block::Error::Other(e)) => Err(e)?,
                }
            }

            // read some bytes from the rx fifo if we can
            while amount > 0 {
                match self.read_one() {
                    Ok(val) => {
                        if let Some(dest) = read.next() {
                            *dest = val
                        }
                        amount -= 1;
                    }
                    Err(block::Error::WouldBlock) => break,
                    Err(block::Error::Other(e)) => Err(e)?,
                }
            }
        }

        // there may still be some bytes left in write
        let mut extra = true;
        while extra || amount > 0 {
            // push some bytes into the tx fifo if we can
            while extra && amount < FIFO_SIZE {
                let Some(val) = write.peek() else {
                    extra = false;
                    break;
                };
                match self.write_one(*val) {
                    Ok(()) => {
                        amount += 1;
                        write.next();
                    }
                    Err(block::Error::WouldBlock) => break,
                    Err(block::Error::Other(e)) => Err(e)?,
                }
            }

            // read some bytes from the rx fifo if we can
            while amount > 0 {
                match self.read_one() {
                    Ok(_) => amount -= 1,
                    Err(block::Error::WouldBlock) => break,
                    Err(block::Error::Other(e)) => Err(e)?,
                }
            }
        }

        Ok(())
    }

    /// Write and read to SPI simultaneously, overwriting the buffer.
    pub fn transfer_in_place(&mut self, buffer: &mut [u8]) -> Result<(), Infallible> {
        block::block!(self.flush())?;

        let mut tx = 0;
        let mut rx = 0;
        while rx < buffer.len() {
            // push some bytes into the tx fifo if we can
            while ((tx - rx) as u8) < FIFO_SIZE && tx < buffer.len() {
                match self.write_one(buffer[tx]) {
                    Ok(()) => tx += 1,
                    Err(block::Error::WouldBlock) => break,
                    Err(block::Error::Other(e)) => Err(e)?,
                }
            }

            // read some bytes from the rx fifo if we can
            while rx < tx {
                match self.read_one() {
                    Ok(val) => {
                        buffer[rx] = val;
                        rx += 1;
                    }
                    Err(block::Error::WouldBlock) => break,
                    Err(block::Error::Other(e)) => Err(e)?,
                }
            }
        }

        Ok(())
    }

    /// Read a buffer from SPI, sending 0x00.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), Infallible> {
        self.transfer_iter(buffer.iter_mut(), core::iter::empty())
    }

    /// Write a buffer to SPI, discarding read values.
    pub fn write(&mut self, buffer: &[u8]) -> Result<(), Infallible> {
        self.transfer_iter(core::iter::empty(), buffer.iter().copied())
    }

    /// Write and read to SPI simultaneously.
    ///
    /// If read is shorter than write, discard all incoming bytes
    /// after that point. If write is shorter than read, write 0x00
    /// after the end of write.
    pub fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Infallible> {
        self.transfer_iter(read.iter_mut(), write.iter().copied())
    }
}

impl<Spi, Miso, Mosi> Port<Spi, Master, Miso, Mosi, Spi::Ssn>
where
    Spi: Instance,
{
    /// Set the slave select line active (low).
    pub fn slave_select_active(&mut self) {
        self.spi.cr().modify(|_r, w| w.msr_ssn().low());
    }

    /// Set the slave select line inactive (high).
    pub fn slave_select_inactive(&mut self) {
        self.spi.cr().modify(|_r, w| w.msr_ssn().high());
    }
}
