use super::crc::{CrcDigest, CrcStyle};
use super::parse::InputParse;

/// A trait for serializing messages.
pub trait Serializer {
    type Error;

    fn write_u8(&mut self, val: u8) -> Result<(), Self::Error>;

    // everything else can be written in terms of write_u8
    // (although they probably should be specialized in some impls)

    // Note: they *definitely should* be specialized in
    // SerializerLength and &mut S so if you add a method here, add
    // one there.

    fn write_bytes(&mut self, val: &[u8]) -> Result<(), Self::Error> {
        for b in val.iter() {
            self.write_u8(*b)?;
        }
        Ok(())
    }

    // this one is tricky. if this becomes a bottleneck, split out a
    // new trait that can be implemented one-by-one on special-case I
    fn write_slice<I>(&mut self, val: &I) -> Result<(), Self::Error>
    where
        I: InputParse,
    {
        for b in val.iter_elements() {
            self.write_u8(b)?;
        }
        Ok(())
    }

    fn write_le_u16(&mut self, val: u16) -> Result<(), Self::Error> {
        self.write_bytes(&[(val & 0xff) as u8, (val >> 8) as u8])
    }

    fn write_le_i16(&mut self, val: i16) -> Result<(), Self::Error> {
        self.write_le_u16(val as u16)
    }

    fn write_le_u32(&mut self, val: u32) -> Result<(), Self::Error> {
        self.write_bytes(&[
            (val & 0xff) as u8,
            ((val >> 8) & 0xff) as u8,
            ((val >> 16) & 0xff) as u8,
            ((val >> 24) & 0xff) as u8,
        ])
    }

    fn write_le_i32(&mut self, val: i32) -> Result<(), Self::Error> {
        self.write_le_u32(val as u32)
    }
}

impl<S> Serializer for &mut S
where
    S: Serializer,
{
    type Error = S::Error;

    fn write_u8(&mut self, val: u8) -> Result<(), Self::Error> {
        (*self).write_u8(val)
    }

    fn write_bytes(&mut self, val: &[u8]) -> Result<(), Self::Error> {
        (*self).write_bytes(val)
    }

    fn write_le_u16(&mut self, val: u16) -> Result<(), Self::Error> {
        (*self).write_le_u16(val)
    }

    fn write_le_i16(&mut self, val: i16) -> Result<(), Self::Error> {
        (*self).write_le_i16(val)
    }

    fn write_le_u32(&mut self, val: u32) -> Result<(), Self::Error> {
        (*self).write_le_u32(val)
    }

    fn write_le_i32(&mut self, val: i32) -> Result<(), Self::Error> {
        (*self).write_le_i32(val)
    }
}

/// Wrap an std::io::Write to become a Serializer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SerializerWrap<T> {
    inner: T,
}

impl<T> SerializerWrap<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn done(self) -> T {
        self.inner
    }
}

impl<T> std::ops::Deref for SerializerWrap<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for SerializerWrap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Serializer for SerializerWrap<T>
where
    T: std::io::Write,
{
    type Error = std::io::Error;

    fn write_u8(&mut self, val: u8) -> Result<(), Self::Error> {
        self.inner.write_all(&[val])
    }

    fn write_bytes(&mut self, val: &[u8]) -> Result<(), Self::Error> {
        self.inner.write_all(val)
    }
}

/// A serializer that only counts bytes written.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SerializerLength {
    len: usize,
}

impl SerializerLength {
    pub fn new() -> Self {
        SerializerLength { len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Serializer for SerializerLength {
    type Error = void::Void;

    fn write_u8(&mut self, _val: u8) -> Result<(), Self::Error> {
        self.len += 1;
        Ok(())
    }

    fn write_bytes(&mut self, val: &[u8]) -> Result<(), Self::Error> {
        self.len += val.len();
        Ok(())
    }

    fn write_le_u16(&mut self, _val: u16) -> Result<(), Self::Error> {
        self.len += 2;
        Ok(())
    }

    fn write_le_i16(&mut self, _val: i16) -> Result<(), Self::Error> {
        self.len += 2;
        Ok(())
    }

    fn write_le_u32(&mut self, _val: u32) -> Result<(), Self::Error> {
        self.len += 4;
        Ok(())
    }

    fn write_le_i32(&mut self, _val: i32) -> Result<(), Self::Error> {
        self.len += 4;
        Ok(())
    }
}

/// A serializer that also computes a CRC on the side.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SerializerCrc<'a, C, T>
where
    C: CrcStyle + 'a,
{
    digest: C::Digest<'a>,
    inner: T,
}

impl<'a, C, T> SerializerCrc<'a, C, T>
where
    C: CrcStyle + 'a,
{
    pub fn new(crc: &'a C, inner: T) -> Self {
        Self {
            digest: crc.digest(),
            inner,
        }
    }

    pub fn finalize(self) -> (u16, T) {
        (self.digest.finalize(), self.inner)
    }
}

impl<'a, C, T> std::ops::Deref for SerializerCrc<'a, C, T>
where
    C: CrcStyle + 'a,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, C, T> std::ops::DerefMut for SerializerCrc<'a, C, T>
where
    C: CrcStyle + 'a,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, C, T> Serializer for SerializerCrc<'a, C, T>
where
    C: CrcStyle + 'a,
    T: Serializer,
{
    type Error = T::Error;

    fn write_u8(&mut self, val: u8) -> Result<(), Self::Error> {
        self.digest.update(&[val]);
        self.inner.write_u8(val)
    }

    fn write_bytes(&mut self, val: &[u8]) -> Result<(), Self::Error> {
        self.digest.update(val);
        self.inner.write_bytes(val)
    }
}

/// A serializer that also computes a CRC on the side.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SerializerObfuscated<T> {
    key: super::obfuscation::Key,
    inner: T,
}

impl<T> SerializerObfuscated<T> {
    pub fn new(inner: T) -> Self {
        Self {
            key: super::obfuscation::Key::new(),
            inner,
        }
    }

    pub fn done(self) -> T {
        self.inner
    }
}

impl<T> std::ops::Deref for SerializerObfuscated<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for SerializerObfuscated<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Serializer for SerializerObfuscated<T>
where
    T: Serializer,
{
    type Error = T::Error;

    fn write_u8(&mut self, val: u8) -> Result<(), Self::Error> {
        self.inner.write_u8(self.key.apply(val))
    }

    fn write_bytes(&mut self, val: &[u8]) -> Result<(), Self::Error> {
        for b in val.iter() {
            self.inner.write_u8(self.key.apply(*b))?;
        }
        Ok(())
    }
}

/// A trait for serializing messages.
pub trait MessageSerialize {
    /// The message type for this message.
    fn message_type(&self) -> u16;

    /// Serialize just the message body.
    ///
    /// For this to work correctly, it *must* perform the same actions
    /// every time it is called with the same message. That means no
    /// IO, no funny business.
    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer;

    // these can all use default implementations

    /// Serialize the message into a frame body, with type and length header.
    fn frame_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        use void::ResultVoidExt;

        // run it once to get a length
        let mut len_ser = SerializerLength::new();
        self.message_body(&mut len_ser).void_unwrap();
        let len = len_ser.len() as u16;

        // frame is type, length, body
        ser.write_le_u16(self.message_type())?;
        ser.write_le_u16(len)?;
        self.message_body(ser)
    }

    /// Serialize the message into a full frame, with obfuscation,
    /// CRC, and start/end markers.
    fn frame<C, S>(&self, crc: &C, ser: &mut S) -> Result<(), S::Error>
    where
        C: CrcStyle,
        S: Serializer,
    {
        use void::ResultVoidExt;

        // run it once to get a length
        let mut len_ser = SerializerLength::new();
        self.frame_body(&mut len_ser).void_unwrap();
        let len = len_ser.len() as u16;

        // frame is start, len, obfuscated(body, crc), end
        ser.write_bytes(&super::FRAME_START)?;
        ser.write_le_u16(len)?;

        let obfuscate = SerializerObfuscated::new(ser);

        let mut crc_ser = SerializerCrc::new(crc, obfuscate);
        self.frame_body(&mut crc_ser)?;
        let (crc_val, mut obfuscate) = crc_ser.finalize();

        obfuscate.write_le_u16(crc_val)?;
        let ser = obfuscate.done();

        ser.write_bytes(&super::FRAME_END)
    }
}
