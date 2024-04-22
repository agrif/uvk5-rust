/// Generic CRC style, for encoding and decoding frames.
pub trait CrcStyle {
    type Digest<'a>
    where
        Self: 'a;

    fn digest<'a>(&'a self) -> Self::Digest<'a>;
    fn update<'a>(&'a self, digest: &mut Self::Digest<'a>, bytes: &[u8]);
    fn finalize<'a>(&'a self, digest: Self::Digest<'a>) -> u16;

    fn validate(&self, calculated: u16, provided: u16) -> bool {
        calculated == provided
    }
}

impl<C> CrcStyle for &C
where
    C: CrcStyle,
{
    type Digest<'a> = C::Digest<'a> where Self: 'a;

    fn digest<'a>(&'a self) -> Self::Digest<'a> {
        (*self).digest()
    }

    fn update<'a>(&'a self, digest: &mut Self::Digest<'a>, bytes: &[u8]) {
        (*self).update(digest, bytes)
    }

    fn finalize<'a>(&'a self, digest: Self::Digest<'a>) -> u16 {
        (*self).finalize(digest)
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
    type Digest<'a> = CrcEither<A::Digest<'a>, B::Digest<'a>> where Self: 'a;

    fn digest<'a>(&'a self) -> Self::Digest<'a> {
        match self {
            Self::Left(a) => Self::Digest::Left(a.digest()),
            Self::Right(b) => Self::Digest::Right(b.digest()),
        }
    }

    fn update<'a>(&'a self, digest: &mut Self::Digest<'a>, bytes: &[u8]) {
        match self {
            Self::Left(a) => {
                if let Self::Digest::Left(ref mut ad) = digest {
                    a.update(ad, bytes)
                } else {
                    // left crc always makes left digest
                    unreachable!();
                }
            }
            Self::Right(b) => {
                if let Self::Digest::Right(ref mut bd) = digest {
                    b.update(bd, bytes)
                } else {
                    // right crc always makes right digest
                    unreachable!();
                }
            }
        }
    }

    fn finalize<'a>(&'a self, digest: Self::Digest<'a>) -> u16 {
        match self {
            Self::Left(a) => {
                if let Self::Digest::Left(ad) = digest {
                    a.finalize(ad)
                } else {
                    // left crc always makes left digest
                    unreachable!();
                }
            }
            Self::Right(b) => {
                if let Self::Digest::Right(bd) = digest {
                    b.finalize(bd)
                } else {
                    // right crc always makes right digest
                    unreachable!();
                }
            }
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
    type Digest<'a> = ();

    fn digest<'a>(&'a self) -> Self::Digest<'a> {
        ()
    }

    fn update<'a>(&'a self, _digest: &mut Self::Digest<'a>, _bytes: &[u8]) {}

    fn finalize<'a>(&'a self, _digest: Self::Digest<'a>) -> u16 {
        self.0
    }
}

/// A CRC that is always a specific given value, and always validates.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrcConstantIgnore(pub u16);

impl CrcStyle for CrcConstantIgnore {
    type Digest<'a> = ();

    fn digest<'a>(&'a self) -> Self::Digest<'a> {
        ()
    }

    fn update<'a>(&'a self, _digest: &mut Self::Digest<'a>, _bytes: &[u8]) {}

    fn finalize<'a>(&'a self, _digest: Self::Digest<'a>) -> u16 {
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
    type Digest<'a> = crc::Digest<'a, u16, crc::Table<1>>;

    fn digest<'a>(&'a self) -> Self::Digest<'a> {
        self.0.digest()
    }

    fn update<'a>(&'a self, digest: &mut Self::Digest<'a>, bytes: &[u8]) {
        digest.update(bytes)
    }

    fn finalize<'a>(&'a self, digest: Self::Digest<'a>) -> u16 {
        digest.finalize()
    }
}

impl CrcStyle for crc::Crc<u16, crc::NoTable> {
    type Digest<'a> = crc::Digest<'a, u16, crc::NoTable>;

    fn digest<'a>(&'a self) -> Self::Digest<'a> {
        self.digest()
    }

    fn update<'a>(&'a self, digest: &mut Self::Digest<'a>, bytes: &[u8]) {
        digest.update(bytes)
    }

    fn finalize<'a>(&'a self, digest: Self::Digest<'a>) -> u16 {
        digest.finalize()
    }
}

impl CrcStyle for crc::Crc<u16, crc::Table<1>> {
    type Digest<'a> = crc::Digest<'a, u16, crc::Table<1>>;

    fn digest<'a>(&'a self) -> Self::Digest<'a> {
        self.digest()
    }

    fn update<'a>(&'a self, digest: &mut Self::Digest<'a>, bytes: &[u8]) {
        digest.update(bytes)
    }

    fn finalize<'a>(&'a self, digest: Self::Digest<'a>) -> u16 {
        digest.finalize()
    }
}

impl CrcStyle for crc::Crc<u16, crc::Table<16>> {
    type Digest<'a> = crc::Digest<'a, u16, crc::Table<16>>;

    fn digest<'a>(&'a self) -> Self::Digest<'a> {
        self.digest()
    }

    fn update<'a>(&'a self, digest: &mut Self::Digest<'a>, bytes: &[u8]) {
        digest.update(bytes)
    }

    fn finalize<'a>(&'a self, digest: Self::Digest<'a>) -> u16 {
        digest.finalize()
    }
}
