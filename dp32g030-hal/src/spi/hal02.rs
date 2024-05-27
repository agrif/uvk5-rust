use core::convert::Infallible;
use embedded_hal_02::blocking::spi as hal02blocking;
use embedded_hal_02::spi as hal02;

use crate::block;

use super::{Instance, Master, Port};

impl<Spi, Miso, Mosi> hal02::FullDuplex<u8> for Port<Spi, Master, Miso, Mosi, ()>
where
    Spi: Instance,
{
    type Error = Infallible;

    fn read(&mut self) -> block::Result<u8, Self::Error> {
        Port::read_one(self)
    }

    fn send(&mut self, word: u8) -> block::Result<(), Self::Error> {
        Port::write_one(self, word)
    }
}

impl<Spi, Miso, Mosi> hal02blocking::Transactional<u8> for Port<Spi, Master, Miso, Mosi, ()>
where
    Spi: Instance,
{
    type Error = Infallible;

    fn exec(&mut self, operations: &mut [hal02blocking::Operation<u8>]) -> Result<(), Self::Error> {
        use hal02blocking::Operation;

        for op in operations.iter_mut() {
            match op {
                Operation::Write(buf) => {
                    Port::write(self, buf)?;
                }
                Operation::Transfer(buf) => {
                    Port::transfer_in_place(self, buf)?;
                }
            }
        }

        Ok(())
    }
}

impl<Spi, Miso, Mosi> hal02blocking::Transfer<u8> for Port<Spi, Master, Miso, Mosi, ()>
where
    Spi: Instance,
{
    type Error = Infallible;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        Port::transfer_in_place(self, words)?;
        Ok(words)
    }
}

impl<Spi, Miso, Mosi> hal02blocking::Write<u8> for Port<Spi, Master, Miso, Mosi, ()>
where
    Spi: Instance,
{
    type Error = Infallible;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        Port::write(self, words)
    }
}

impl<Spi, Miso, Mosi> hal02blocking::WriteIter<u8> for Port<Spi, Master, Miso, Mosi, ()>
where
    Spi: Instance,
{
    type Error = Infallible;

    fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item = u8>,
    {
        Port::transfer_iter(self, core::iter::empty(), words.into_iter().fuse())
    }
}
