use core::convert::Infallible;
use embedded_hal_1::spi as hal1;
use embedded_hal_nb::spi as hal1nb;

use crate::block;

use super::{Instance, Master, Port};

impl<Spi, Miso, Mosi, Ssn> hal1::ErrorType for Port<Spi, Master, Miso, Mosi, Ssn>
where
    Spi: Instance,
{
    type Error = Infallible;
}

impl<Spi, Miso, Mosi> hal1::SpiBus<u8> for Port<Spi, Master, Miso, Mosi, ()>
where
    Spi: Instance,
{
    #[inline(always)]
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        Port::read(self, words)
    }

    #[inline(always)]
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        Port::write(self, words)
    }

    #[inline(always)]
    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        Port::transfer(self, read, write)
    }

    #[inline(always)]
    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        Port::transfer_in_place(self, words)
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<(), Self::Error> {
        block::block!(Port::flush(self))
    }
}

impl<Spi, Miso, Mosi> hal1::SpiDevice<u8> for Port<Spi, Master, Miso, Mosi, Spi::Ssn>
where
    Spi: Instance,
{
    #[inline(always)]
    fn transaction(
        &mut self,
        operations: &mut [hal1::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        block::block!(Port::flush(self))?;
        Port::slave_select_active(self);

        // we need to do a try/finally thing on this block of code
        // to reset slave select at the end
        let mut inner = || -> Result<(), Self::Error> {
            for op in operations.iter_mut() {
                match op {
                    hal1::Operation::Read(buf) => Port::read(self, buf)?,
                    hal1::Operation::Write(buf) => Port::write(self, buf)?,
                    hal1::Operation::Transfer(read, write) => Port::transfer(self, read, write)?,
                    hal1::Operation::TransferInPlace(buf) => Port::transfer_in_place(self, buf)?,
                    hal1::Operation::DelayNs(ns) => {
                        // FIXME
                        // I have no idea what uses this, or why.
                        // best effort: at its fastest, cpu is 14ns / cycle
                        // round up to 16ns / cycle, then take
                        // ceil(ns / 16)
                        cortex_m::asm::delay((*ns + 0xf) >> 4);
                    }
                }
            }
            Ok(())
        };

        if let Err(e) = inner() {
            Port::slave_select_inactive(self);
            Err(e)?;
        }

        block::block!(Port::flush(self))?;
        Port::slave_select_inactive(self);
        Ok(())
    }
}

impl<Spi, Miso, Mosi> hal1nb::FullDuplex<u8> for Port<Spi, Master, Miso, Mosi, ()>
where
    Spi: Instance,
{
    #[inline(always)]
    fn read(&mut self) -> block::Result<u8, Self::Error> {
        Port::read_one(self)
    }

    #[inline(always)]
    fn write(&mut self, word: u8) -> block::Result<(), Self::Error> {
        Port::write_one(self, word)
    }
}
