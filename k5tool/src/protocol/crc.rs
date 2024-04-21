/// Generic CRC style, for encoding and decoding frames.
pub trait CrcStyle {
    fn calculate<I>(&self, it: I) -> u16
    where
        I: Iterator<Item = u8>;
    fn validate(&self, calculated: u16, provided: u16) -> bool {
        calculated == provided
    }
}

impl<C> CrcStyle for &C
where
    C: CrcStyle,
{
    fn calculate<I>(&self, it: I) -> u16
    where
        I: Iterator<Item = u8>,
    {
        (*self).calculate(it)
    }
    fn validate(&self, calculated: u16, provided: u16) -> bool {
        (*self).validate(calculated, provided)
    }
}

/// A CRC that is one of two possible implementations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrcEither<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> CrcStyle for CrcEither<A, B>
where
    A: CrcStyle,
    B: CrcStyle,
{
    fn calculate<I>(&self, it: I) -> u16
    where
        I: Iterator<Item = u8>,
    {
        match self {
            Self::Left(a) => a.calculate(it),
            Self::Right(b) => b.calculate(it),
        }
    }

    fn validate(&self, calculated: u16, provided: u16) -> bool {
        match self {
            Self::Left(a) => a.validate(calculated, provided),
            Self::Right(b) => b.validate(calculated, provided),
        }
    }
}

/// A CRC that is always a specific given value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrcConstant(pub u16);

impl CrcStyle for CrcConstant {
    fn calculate<I>(&self, _it: I) -> u16
    where
        I: Iterator<Item = u8>,
    {
        self.0
    }
}

/// A CRC that is always a specific given value, and always validates.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrcConstantIgnore(pub u16);

impl CrcStyle for CrcConstantIgnore {
    fn calculate<I>(&self, _it: I) -> u16
    where
        I: Iterator<Item = u8>,
    {
        self.0
    }

    fn validate(&self, _calculated: u16, _provided: u16) -> bool {
        true
    }
}

/// A 16-bit XModem CRC, used for host to radio frames.
#[derive(Clone)]
pub struct CrcXModem(crc::Crc<u16>);

impl CrcXModem {
    pub fn new() -> Self {
        Self(crc::Crc::<u16>::new(&crc::CRC_16_XMODEM))
    }
}

impl CrcStyle for CrcXModem {
    fn calculate<I>(&self, it: I) -> u16
    where
        I: Iterator<Item = u8>,
    {
        let mut digest = self.0.digest();
        for v in it {
            digest.update(&[v]);
        }
        digest.finalize()
    }
}

impl CrcStyle for crc::Crc<u16, crc::NoTable> {
    fn calculate<I>(&self, it: I) -> u16
    where
        I: Iterator<Item = u8>,
    {
        let mut digest = self.digest();
        for v in it {
            digest.update(&[v]);
        }
        digest.finalize()
    }
}

impl CrcStyle for crc::Crc<u16, crc::Table<1>> {
    fn calculate<I>(&self, it: I) -> u16
    where
        I: Iterator<Item = u8>,
    {
        let mut digest = self.digest();
        for v in it {
            digest.update(&[v]);
        }
        digest.finalize()
    }
}

impl CrcStyle for crc::Crc<u16, crc::Table<16>> {
    fn calculate<I>(&self, it: I) -> u16
    where
        I: Iterator<Item = u8>,
    {
        let mut digest = self.digest();
        for v in it {
            digest.update(&[v]);
        }
        digest.finalize()
    }
}
