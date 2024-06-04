use core::convert::Infallible;

use embedded_hal_nb::serial as hal1nb;
use embedded_io as hal1;

use crate::block;

use super::{Instance, Port, Rx, Tx, UartData};

impl<Uart, Data, Pair> hal1::ErrorType for Rx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;
}

impl<Uart, Data, Pair> hal1::ErrorType for Tx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;
}

impl<Uart, Data> hal1::ErrorType for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;
}

impl<Uart, Data, Pair> hal1nb::ErrorType for Rx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;
}

impl<Uart, Data, Pair> hal1nb::ErrorType for Tx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;
}

impl<Uart, Data> hal1nb::ErrorType for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    type Error = Infallible;
}

impl<Uart, Data, Pair> hal1::Read for Rx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        block::block!(Rx::read(self, buf))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), hal1::ReadExactError<Self::Error>> {
        Rx::read_exact(self, buf).map_err(hal1::ReadExactError::Other)
    }
}

impl<Uart, Data> hal1::Read for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        block::block!(Rx::read(&mut self.rx, buf))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), hal1::ReadExactError<Self::Error>> {
        Rx::read_exact(&mut self.rx, buf).map_err(hal1::ReadExactError::Other)
    }
}

impl<Uart, Data, Pair> hal1nb::Read<u8> for Rx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    fn read(&mut self) -> block::Result<u8, Self::Error> {
        Rx::read_one(self)
    }
}

impl<Uart, Data> hal1nb::Read<u8> for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    fn read(&mut self) -> block::Result<u8, Self::Error> {
        Rx::read_one(&mut self.rx)
    }
}

impl<Uart, Data, Pair> hal1::ReadReady for Rx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!Rx::is_empty(self))
    }
}

impl<Uart, Data> hal1::ReadReady for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    fn read_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!Rx::is_empty(&self.rx))
    }
}

impl<Uart, Data, Pair> hal1::Write for Tx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        block::block!(Tx::write(self, buf))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        block::block!(Tx::flush(self))
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        Tx::write_all(self, buf)
    }

    fn write_fmt(
        &mut self,
        fmt: core::fmt::Arguments<'_>,
    ) -> Result<(), hal1::WriteFmtError<Self::Error>> {
        match core::fmt::write(self, fmt) {
            Ok(()) => Ok(()),
            Err(_) => Err(hal1::WriteFmtError::FmtError),
        }
    }
}

impl<Uart, Data> hal1::Write for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        block::block!(Tx::write(&mut self.tx, buf))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        block::block!(Tx::flush(&mut self.tx))
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        Tx::write_all(&mut self.tx, buf)
    }

    fn write_fmt(
        &mut self,
        fmt: core::fmt::Arguments<'_>,
    ) -> Result<(), hal1::WriteFmtError<Self::Error>> {
        match core::fmt::write(&mut self.tx, fmt) {
            Ok(()) => Ok(()),
            Err(_) => Err(hal1::WriteFmtError::FmtError),
        }
    }
}

impl<Uart, Data, Pair> hal1nb::Write for Tx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    fn write(&mut self, word: u8) -> block::Result<(), Self::Error> {
        Tx::write_one(self, word)
    }

    fn flush(&mut self) -> block::Result<(), Self::Error> {
        Tx::flush(self)
    }
}

impl<Uart, Data> hal1nb::Write for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    fn write(&mut self, word: u8) -> block::Result<(), Self::Error> {
        Tx::write_one(&mut self.tx, word)
    }

    fn flush(&mut self) -> block::Result<(), Self::Error> {
        Tx::flush(&mut self.tx)
    }
}

impl<Uart, Data, Pair> hal1::WriteReady for Tx<Uart, Data, Pair>
where
    Uart: Instance,
    Data: UartData,
{
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!Tx::is_full(self))
    }
}

impl<Uart, Data> hal1::WriteReady for Port<Uart, Data>
where
    Uart: Instance,
    Data: UartData,
{
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        Ok(!Tx::is_full(&mut self.tx))
    }
}
