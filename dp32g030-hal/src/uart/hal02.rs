use core::convert::Infallible;
use embedded_hal_02::blocking::serial as hal02blocking;
use embedded_hal_02::serial as hal02;

use crate::block;

use super::{Instance, Port, Rx, Tx, UartData};

impl<Uart, Data, Pair> hal02::Read<u8> for Rx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;

    fn read(&mut self) -> block::Result<u8, Self::Error> {
        self.read_one()
    }
}

impl<Uart, Data> hal02::Read<u8> for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;

    fn read(&mut self) -> block::Result<u8, Self::Error> {
        self.rx.read_one()
    }
}

impl<Uart, Data, Pair> hal02::Write<u8> for Tx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;

    fn write(&mut self, word: u8) -> block::Result<(), Self::Error> {
        self.write_one(word)
    }

    fn flush(&mut self) -> block::Result<(), Self::Error> {
        Tx::flush(self)
    }
}

impl<Uart, Data> hal02::Write<u8> for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;

    fn write(&mut self, word: u8) -> block::Result<(), Self::Error> {
        self.tx.write_one(word)
    }

    fn flush(&mut self) -> block::Result<(), Self::Error> {
        Tx::flush(&mut self.tx)
    }
}

impl<Uart, Data, Pair> hal02blocking::Write<u8> for Tx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.write_all(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        block::block!(Tx::flush(self))
    }
}

impl<Uart, Data> hal02blocking::Write<u8> for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.tx.write_all(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        block::block!(Tx::flush(&mut self.tx))
    }
}
