/// Generic CRC style, for encoding and decoding frames.
pub trait CrcStyle {
    type Digest<'a>: CrcDigest
    where
        Self: 'a;

    fn digest(&self) -> Self::Digest<'_>;

    fn validate(&self, calculated: u16, provided: u16) -> bool {
        calculated == provided
    }
}

/// Interface for a CRC digest created by a [CrcStyle].
pub trait CrcDigest {
    fn update(&mut self, bytes: &[u8]);
    fn finalize(self) -> u16;
}

impl<C> CrcStyle for &C
where
    C: CrcStyle,
{
    type Digest<'a> = C::Digest<'a> where Self: 'a;

    fn digest(&self) -> Self::Digest<'_> {
        (*self).digest()
    }

    fn validate(&self, calculated: u16, provided: u16) -> bool {
        (*self).validate(calculated, provided)
    }
}

/// A [CrcStyle] that is one of two possible implementations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

    fn digest(&self) -> Self::Digest<'_> {
        match self {
            Self::Left(a) => Self::Digest::Left(a.digest()),
            Self::Right(b) => Self::Digest::Right(b.digest()),
        }
    }

    fn validate(&self, calculated: u16, provided: u16) -> bool {
        match self {
            Self::Left(a) => a.validate(calculated, provided),
            Self::Right(b) => b.validate(calculated, provided),
        }
    }
}

impl<A, B> CrcDigest for CrcEither<A, B>
where
    A: CrcDigest,
    B: CrcDigest,
{
    fn update(&mut self, bytes: &[u8]) {
        match self {
            Self::Left(a) => a.update(bytes),
            Self::Right(b) => b.update(bytes),
        }
    }

    fn finalize(self) -> u16 {
        match self {
            Self::Left(a) => a.finalize(),
            Self::Right(b) => b.finalize(),
        }
    }
}

/// A [CrcStyle] that is always a specific given value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CrcConstant(pub u16);

impl CrcStyle for CrcConstant {
    type Digest<'a> = CrcConstant;

    fn digest(&self) -> Self::Digest<'_> {
        CrcConstant(self.0)
    }
}

impl CrcDigest for CrcConstant {
    fn update(&mut self, _bytes: &[u8]) {}

    fn finalize(self) -> u16 {
        self.0
    }
}

/// A [CrcStyle] that writes a specific given value, and always validates.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CrcConstantIgnore(pub u16);

impl CrcStyle for CrcConstantIgnore {
    type Digest<'a> = CrcConstant;

    fn digest(&self) -> Self::Digest<'_> {
        CrcConstant(self.0)
    }

    fn validate(&self, _calculated: u16, _provided: u16) -> bool {
        true
    }
}

/// A 16-bit XModem [CrcStyle], used for host to radio frames.
#[derive(Clone)]
pub struct CrcXModem(crc::Crc<u16>);

/// A 16-bit XModem [CrcDigest].
#[derive(Clone)]
pub struct CrcXModemDigest<'a>(crc::Digest<'a, u16, crc::Table<1>>);

impl CrcXModem {
    pub fn new() -> Self {
        Self(crc::Crc::<u16>::new(&crc::CRC_16_XMODEM))
    }
}

impl Default for CrcXModem {
    fn default() -> Self {
        Self::new()
    }
}

impl CrcStyle for CrcXModem {
    type Digest<'a> = CrcXModemDigest<'a>;

    fn digest(&self) -> Self::Digest<'_> {
        CrcXModemDigest(self.0.digest())
    }
}

impl<'a> CrcDigest for CrcXModemDigest<'a> {
    fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes)
    }

    fn finalize(self) -> u16 {
        self.0.finalize()
    }
}

impl CrcStyle for crc::Crc<u16, crc::NoTable> {
    type Digest<'a> = crc::Digest<'a, u16, crc::NoTable>;

    fn digest(&self) -> Self::Digest<'_> {
        self.digest()
    }
}

impl<'a> CrcDigest for crc::Digest<'a, u16, crc::NoTable> {
    fn update(&mut self, bytes: &[u8]) {
        self.update(bytes)
    }

    fn finalize(self) -> u16 {
        self.finalize()
    }
}

impl CrcStyle for crc::Crc<u16, crc::Table<1>> {
    type Digest<'a> = crc::Digest<'a, u16, crc::Table<1>>;

    fn digest(&self) -> Self::Digest<'_> {
        self.digest()
    }
}

impl<'a> CrcDigest for crc::Digest<'a, u16, crc::Table<1>> {
    fn update(&mut self, bytes: &[u8]) {
        self.update(bytes)
    }

    fn finalize(self) -> u16 {
        self.finalize()
    }
}

impl CrcStyle for crc::Crc<u16, crc::Table<16>> {
    type Digest<'a> = crc::Digest<'a, u16, crc::Table<16>>;

    fn digest(&self) -> Self::Digest<'_> {
        self.digest()
    }
}

impl<'a> CrcDigest for crc::Digest<'a, u16, crc::Table<16>> {
    fn update(&mut self, bytes: &[u8]) {
        self.update(bytes)
    }

    fn finalize(self) -> u16 {
        self.finalize()
    }
}
