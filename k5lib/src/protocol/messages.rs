use nom::{error::Error, Parser};

use super::parse::{InputParse, MessageParse};
use super::serialize::{MessageSerialize, Serializer};

/// Parse a version.
fn parse_version<I>(input: I) -> nom::IResult<I, crate::Version>
where
    I: InputParse,
{
    let (input, data) = parse_array(nom::number::complete::u8)(input)?;
    Ok((input, crate::Version::new(data)))
}

/// Parse a statically-sized array with a parser.
fn parse_array<I, P, A, const SIZE: usize>(parser: P) -> impl FnMut(I) -> nom::IResult<I, [A; SIZE]>
where
    I: InputParse,
    P: Fn(I) -> nom::IResult<I, A>,
    A: Default + Copy,
{
    move |input| {
        let mut data = [A::default(); SIZE];
        let (input, _) = nom::multi::fill(&parser, &mut data[..])(input)?;
        Ok((input, data))
    }
}

/// A trait for messages that have statically-known message types.
pub trait MessageType {
    const TYPE: u16;
}

/// Any kind of message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Message<I> {
    Host(HostMessage<I>),
    Radio(RadioMessage<I>),
}

impl<I> Message<I> {
    pub fn map<F, J>(self, f: F) -> Message<J>
    where
        F: FnOnce(I) -> J,
    {
        match self {
            Self::Host(o) => Message::Host(o.map(f)),
            Self::Radio(o) => Message::Radio(o.map(f)),
        }
    }

    pub fn map_ref<'a, F, J>(&'a self, f: F) -> Message<J>
    where
        F: FnOnce(&'a I) -> J,
    {
        match self {
            Self::Host(o) => Message::Host(o.map_ref(f)),
            Self::Radio(o) => Message::Radio(o.map_ref(f)),
        }
    }
}

impl<I> MessageSerialize for Message<I>
where
    I: InputParse,
{
    fn message_type(&self) -> u16 {
        match self {
            Self::Host(m) => m.message_type(),
            Self::Radio(m) => m.message_type(),
        }
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Host(m) => m.message_body(ser),
            Self::Radio(m) => m.message_body(ser),
        }
    }
}

impl<I> MessageParse<I> for Message<I>
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        nom::branch::alt((
            nom::combinator::map(HostMessage::parse_body(typ), Message::Host),
            nom::combinator::map(RadioMessage::parse_body(typ), Message::Radio),
        ))
    }
}

/// Messages sent from the host computer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HostMessage<I> {
    /// 0x0514 Hello
    Hello(Hello),
    /// 0x0519 Write Flash (bootloader mode)
    WriteFlash(WriteFlash<I>),
    /// 0x051b Read EEPROM
    ReadEeprom(ReadEeprom),
    /// 0x0530 Bootloader Ready Reply (bootloader mode)
    BootloaderReadyReply(BootloaderReadyReply),
}

impl<I> HostMessage<I> {
    pub fn map<F, J>(self, f: F) -> HostMessage<J>
    where
        F: FnOnce(I) -> J,
    {
        match self {
            Self::Hello(o) => HostMessage::Hello(o),
            Self::WriteFlash(o) => HostMessage::WriteFlash(o.map(f)),
            Self::ReadEeprom(o) => HostMessage::ReadEeprom(o),
            Self::BootloaderReadyReply(o) => HostMessage::BootloaderReadyReply(o),
        }
    }

    pub fn map_ref<'a, F, J>(&'a self, f: F) -> HostMessage<J>
    where
        F: FnOnce(&'a I) -> J,
    {
        match self {
            Self::Hello(o) => HostMessage::Hello(o.clone()),
            Self::WriteFlash(o) => HostMessage::WriteFlash(o.map_ref(f)),
            Self::ReadEeprom(o) => HostMessage::ReadEeprom(o.clone()),
            Self::BootloaderReadyReply(o) => HostMessage::BootloaderReadyReply(o.clone()),
        }
    }
}

impl<I> MessageSerialize for HostMessage<I>
where
    I: InputParse,
{
    fn message_type(&self) -> u16 {
        match self {
            Self::Hello(m) => m.message_type(),
            Self::WriteFlash(m) => m.message_type(),
            Self::ReadEeprom(m) => m.message_type(),
            Self::BootloaderReadyReply(m) => m.message_type(),
        }
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Hello(m) => m.message_body(ser),
            Self::WriteFlash(m) => m.message_body(ser),
            Self::ReadEeprom(m) => m.message_body(ser),
            Self::BootloaderReadyReply(m) => m.message_body(ser),
        }
    }
}

impl<I> MessageParse<I> for HostMessage<I>
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| match typ {
            Hello::TYPE => Hello::parse_body(typ).map(Self::Hello).parse(input),
            WriteFlash::<()>::TYPE => WriteFlash::parse_body(typ)
                .map(Self::WriteFlash)
                .parse(input),
            ReadEeprom::TYPE => ReadEeprom::parse_body(typ)
                .map(Self::ReadEeprom)
                .parse(input),
            BootloaderReadyReply::TYPE => BootloaderReadyReply::parse_body(typ)
                .map(Self::BootloaderReadyReply)
                .parse(input),

            // we don't recognize the message type
            _ => nom::combinator::fail(input),
        }
    }
}

/// Messages sent from the radio.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RadioMessage<I> {
    /// 0x0515 HelloReply
    HelloReply(HelloReply),
    /// 0x0518 Bootloader Ready (bootloader mode)
    BootloaderReady(BootloaderReady),
    /// 0x051a Write Flash Reply (bootloader mode)
    WriteFlashReply(WriteFlashReply),
    /// 0x51c Read EEPROM Reply
    ReadEepromReply(ReadEepromReply<I>),
}

impl<I> RadioMessage<I> {
    pub fn map<F, J>(self, f: F) -> RadioMessage<J>
    where
        F: FnOnce(I) -> J,
    {
        match self {
            Self::HelloReply(o) => RadioMessage::HelloReply(o),
            Self::BootloaderReady(o) => RadioMessage::BootloaderReady(o),
            Self::WriteFlashReply(o) => RadioMessage::WriteFlashReply(o),
            Self::ReadEepromReply(o) => RadioMessage::ReadEepromReply(o.map(f)),
        }
    }

    pub fn map_ref<'a, F, J>(&'a self, f: F) -> RadioMessage<J>
    where
        F: FnOnce(&'a I) -> J,
    {
        match self {
            Self::HelloReply(o) => RadioMessage::HelloReply(o.clone()),
            Self::BootloaderReady(o) => RadioMessage::BootloaderReady(o.clone()),
            Self::WriteFlashReply(o) => RadioMessage::WriteFlashReply(o.clone()),
            Self::ReadEepromReply(o) => RadioMessage::ReadEepromReply(o.map_ref(f)),
        }
    }
}

impl<I> MessageSerialize for RadioMessage<I>
where
    I: InputParse,
{
    fn message_type(&self) -> u16 {
        match self {
            Self::HelloReply(m) => m.message_type(),
            Self::BootloaderReady(m) => m.message_type(),
            Self::WriteFlashReply(m) => m.message_type(),
            Self::ReadEepromReply(m) => m.message_type(),
        }
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::HelloReply(m) => m.message_body(ser),
            Self::BootloaderReady(m) => m.message_body(ser),
            Self::WriteFlashReply(m) => m.message_body(ser),
            Self::ReadEepromReply(m) => m.message_body(ser),
        }
    }
}

impl<I> MessageParse<I> for RadioMessage<I>
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| match typ {
            HelloReply::TYPE => HelloReply::parse_body(typ)
                .map(Self::HelloReply)
                .parse(input),
            BootloaderReady::TYPE => BootloaderReady::parse_body(typ)
                .map(Self::BootloaderReady)
                .parse(input),
            WriteFlashReply::TYPE => WriteFlashReply::parse_body(typ)
                .map(Self::WriteFlashReply)
                .parse(input),
            ReadEepromReply::<()>::TYPE => ReadEepromReply::parse_body(typ)
                .map(Self::ReadEepromReply)
                .parse(input),

            // we don't recognize the message type
            _ => nom::combinator::fail(input),
        }
    }
}

/// Session ID for host messages. Should be the same as the one used
/// in Hello in all messages.
pub const HELLO_SESSION_ID: u32 = 0x6457396a;

/// 0x0514 Hello, host message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hello {
    /// Session ID on all host messages. All further messages must use
    /// this same ID or they will be ignored.
    ///
    /// If unsure, use HELLO_SESSION_ID.
    pub session_id: u32,
}

impl MessageType for Hello {
    const TYPE: u16 = 0x0514;
}

impl MessageSerialize for Hello {
    fn message_type(&self) -> u16 {
        Self::TYPE
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_le_u32(self.session_id)
    }
}

impl<I> MessageParse<I> for Hello
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>>
    where
        I: InputParse,
    {
        move |input| {
            let input = if typ != Self::TYPE {
                nom::combinator::fail::<_, (), _>(input)?.0
            } else {
                input
            };

            let (input, session_id) = nom::number::complete::le_u32(input)?;
            Ok((input, Hello { session_id }))
        }
    }
}

/// 0x0515 HelloReply, radio message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HelloReply {
    /// HelloReply, provided by the radio.
    /// Assume UTF-8, or at least, ASCII, padded by zeros.
    pub version: crate::Version,

    /// Radio is using custom AES key.
    pub has_custom_aes_key: bool,

    /// Radio is in the lock screen.
    pub is_in_lock_screen: bool,

    /// Unknown or unused.
    pub padding: [u8; 2],

    /// AES challenge. See 0x052D.
    pub challenge: [u32; 4],
}

impl MessageType for HelloReply {
    const TYPE: u16 = 0x0515;
}

impl MessageSerialize for HelloReply {
    fn message_type(&self) -> u16 {
        Self::TYPE
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_bytes(self.version.as_bytes())?;
        ser.write_u8(self.has_custom_aes_key as u8)?;
        ser.write_u8(self.is_in_lock_screen as u8)?;
        ser.write_bytes(&self.padding)?;
        for c in self.challenge.iter() {
            ser.write_le_u32(*c)?;
        }
        Ok(())
    }
}

impl<I> MessageParse<I> for HelloReply
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| {
            let input = if typ != Self::TYPE {
                nom::combinator::fail::<_, (), _>(input)?.0
            } else {
                input
            };

            let (input, version) = parse_version(input)?;

            let (input, has_custom_aes_key) = nom::number::complete::u8(input)?;
            let has_custom_aes_key = has_custom_aes_key > 0;

            let (input, is_in_lock_screen) = nom::number::complete::u8(input)?;
            let is_in_lock_screen = is_in_lock_screen > 0;

            let (input, padding) = parse_array(nom::number::complete::u8)(input)?;

            let (input, challenge) = parse_array(nom::number::complete::le_u32)(input)?;

            Ok((
                input,
                HelloReply {
                    version,
                    has_custom_aes_key,
                    is_in_lock_screen,
                    padding,
                    challenge,
                },
            ))
        }
    }
}

/// 0x0518 Bootloader Ready, radio message (bootloader mode).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BootloaderReady {
    /// Chip ID for the radio's CPU.
    pub chip_id: [u32; 4],
    /// Bootloader version.
    pub version: crate::Version,
}

impl MessageType for BootloaderReady {
    const TYPE: u16 = 0x0518;
}

impl MessageSerialize for BootloaderReady {
    fn message_type(&self) -> u16 {
        Self::TYPE
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        for v in self.chip_id.iter() {
            ser.write_le_u32(*v)?;
        }
        ser.write_bytes(self.version.as_bytes())
    }
}

impl<I> MessageParse<I> for BootloaderReady
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| {
            let input = if typ != Self::TYPE {
                nom::combinator::fail::<_, (), _>(input)?.0
            } else {
                input
            };

            // FIXME some bootloaders have different packet formats
            // I suspect they vary the chip_id field size, but...
            // I don't have any examples, so I can't know.
            let (input, chip_id) = parse_array(nom::number::complete::le_u32)(input)?;
            let (input, version) = parse_version(input)?;

            Ok((input, BootloaderReady { chip_id, version }))
        }
    }
}

/// Unknown value in WriteFlash messages.
pub const WRITE_FLASH_SESSION_ID: u32 = 0x1d9f8d8a;

/// Size of the data in a WriteFlash message.
pub const WRITE_FLASH_LEN: usize = 0x100;

/// 0x0519 Write Flash, host message (bootloader mode).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WriteFlash<I> {
    /// Session ID unique to this flash sequence. Use
    /// WRITE_FLASH_SESSION_ID if unsure.
    pub session_id: u32,
    /// Which 0x100 byte page to write. Increments by 1 each message.
    pub page: u16,
    /// Maximum flash page, exclusive. Device boots after writing when
    /// page + 1 == max_page.
    pub max_page: u16,
    /// Length of data. Note data.len() is always 0x100, this
    /// indicates how much data inside is used.
    ///
    /// This seems to be ignored by the bootloader.
    pub len: u16,
    /// Padding.
    pub padding: [u8; 2],
    /// Data to write to flash. Must be 0x100 bytes!
    pub data: I,
}

impl<I> MessageType for WriteFlash<I> {
    const TYPE: u16 = 0x0519;
}

impl<I> WriteFlash<I> {
    pub fn map<F, J>(self, f: F) -> WriteFlash<J>
    where
        F: FnOnce(I) -> J,
    {
        WriteFlash {
            session_id: self.session_id,
            page: self.page,
            max_page: self.max_page,
            len: self.len,
            padding: self.padding,
            data: f(self.data),
        }
    }

    pub fn map_ref<'a, F, J>(&'a self, f: F) -> WriteFlash<J>
    where
        F: FnOnce(&'a I) -> J,
    {
        WriteFlash {
            session_id: self.session_id,
            page: self.page,
            max_page: self.max_page,
            len: self.len,
            padding: self.padding,
            data: f(&self.data),
        }
    }
}

impl<I> MessageSerialize for WriteFlash<I>
where
    I: InputParse,
{
    fn message_type(&self) -> u16 {
        Self::TYPE
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_le_u32(self.session_id)?;
        ser.write_le_u16(self.page)?;
        ser.write_le_u16(self.max_page)?;
        ser.write_le_u16(self.len)?;
        ser.write_bytes(&self.padding)?;

        // I don't like this assert, but this is better than
        // sending a malformed packet to the bootloader. probably.
        assert_eq!(self.data.input_len(), WRITE_FLASH_LEN);
        ser.write_slice(&self.data)
    }
}

impl<I> MessageParse<I> for WriteFlash<I>
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>>
    where
        I: InputParse,
    {
        move |input| {
            let input = if typ != Self::TYPE {
                nom::combinator::fail::<_, (), _>(input)?.0
            } else {
                input
            };

            let (input, session_id) = nom::number::complete::le_u32(input)?;
            let (input, page) = nom::number::complete::le_u16(input)?;
            let (input, max_page) = nom::number::complete::le_u16(input)?;
            let (input, len) = nom::number::complete::le_u16(input)?;

            let (input, padding) = parse_array(nom::number::complete::u8)(input)?;

            // message always has 0x100 bytes here, regardless of len
            let (input, data) = nom::bytes::complete::take(WRITE_FLASH_LEN)(input)?;
            Ok((
                input,
                WriteFlash {
                    session_id,
                    page,
                    max_page,
                    len,
                    padding,
                    data,
                },
            ))
        }
    }
}

/// 0x051a Write Flash Reply, radio message (bootloader mode).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WriteFlashReply {
    /// Session ID, matches the session id sent in the WriteFlash message.
    pub session_id: u32,
    /// Page number, matches the page sent in the WriteFlash message.
    pub page: u16,
    /// Error, 0 indicates success and non-zero indiates error.
    pub error: u16,
}

impl MessageType for WriteFlashReply {
    const TYPE: u16 = 0x051a;
}

impl MessageSerialize for WriteFlashReply {
    fn message_type(&self) -> u16 {
        Self::TYPE
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_le_u32(self.session_id)?;
        ser.write_le_u16(self.page)?;
        ser.write_le_u16(self.error)
    }
}

impl<I> MessageParse<I> for WriteFlashReply
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| {
            let input = if typ != Self::TYPE {
                nom::combinator::fail::<_, (), _>(input)?.0
            } else {
                input
            };

            let (input, session_id) = nom::number::complete::le_u32(input)?;
            let (input, page) = nom::number::complete::le_u16(input)?;
            let (input, error) = nom::number::complete::le_u16(input)?;

            Ok((
                input,
                WriteFlashReply {
                    session_id,
                    page,
                    error,
                },
            ))
        }
    }
}

/// 0x051b Read EEPROM, host message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadEeprom {
    /// Address to read.
    pub address: u16,
    /// Number of bytes to read from address, usually 0x80.
    pub len: u8,
    /// Unknown or unused.
    pub padding: u8,
    /// Session ID, must match the one provided by initial Hello.
    pub session_id: u32,
}

impl MessageType for ReadEeprom {
    const TYPE: u16 = 0x051b;
}

impl MessageSerialize for ReadEeprom {
    fn message_type(&self) -> u16 {
        Self::TYPE
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_le_u16(self.address)?;
        ser.write_u8(self.len)?;
        ser.write_u8(self.padding)?;
        ser.write_le_u32(self.session_id)
    }
}

impl<I> MessageParse<I> for ReadEeprom
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| {
            let input = if typ != Self::TYPE {
                nom::combinator::fail::<_, (), _>(input)?.0
            } else {
                input
            };

            let (input, address) = nom::number::complete::le_u16(input)?;
            let (input, len) = nom::number::complete::u8(input)?;
            let (input, padding) = nom::number::complete::u8(input)?;
            let (input, session_id) = nom::number::complete::le_u32(input)?;
            Ok((
                input,
                ReadEeprom {
                    address,
                    len,
                    padding,
                    session_id,
                },
            ))
        }
    }
}

/// 0x051c Read Eeprom Reply, radio message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadEepromReply<I> {
    /// Address of data read.
    pub address: u16,
    /// Number of bytes of data read.
    pub len: u8,
    /// Unknown or unused.
    pub padding: u8,
    /// Data read from EEPROM.
    pub data: I,
}

impl<I> MessageType for ReadEepromReply<I> {
    const TYPE: u16 = 0x051c;
}

impl<I> ReadEepromReply<I> {
    pub fn map<F, J>(self, f: F) -> ReadEepromReply<J>
    where
        F: FnOnce(I) -> J,
    {
        ReadEepromReply {
            address: self.address,
            len: self.len,
            padding: self.padding,
            data: f(self.data),
        }
    }

    pub fn map_ref<'a, F, J>(&'a self, f: F) -> ReadEepromReply<J>
    where
        F: FnOnce(&'a I) -> J,
    {
        ReadEepromReply {
            address: self.address,
            len: self.len,
            padding: self.padding,
            data: f(&self.data),
        }
    }
}

impl<I> MessageSerialize for ReadEepromReply<I>
where
    I: InputParse,
{
    fn message_type(&self) -> u16 {
        Self::TYPE
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_le_u16(self.address)?;
        ser.write_u8(self.len)?;
        ser.write_u8(self.padding)?;
        ser.write_slice(&self.data)
    }
}

impl<I> MessageParse<I> for ReadEepromReply<I>
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| {
            let input = if typ != Self::TYPE {
                nom::combinator::fail::<_, (), _>(input)?.0
            } else {
                input
            };

            let (input, address) = nom::number::complete::le_u16(input)?;
            let (input, len) = nom::number::complete::u8(input)?;
            let (input, padding) = nom::number::complete::u8(input)?;
            let (input, data) = nom::bytes::complete::take(len as usize)(input)?;
            Ok((
                input,
                ReadEepromReply {
                    address,
                    len,
                    padding,
                    data,
                },
            ))
        }
    }
}

/// 0x0530 Bootloader Ready Reply, host message (bootloader mode).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BootloaderReadyReply {
    /// Incoming firmware version.
    pub version: crate::Version,
}

impl MessageType for BootloaderReadyReply {
    const TYPE: u16 = 0x0530;
}

impl MessageSerialize for BootloaderReadyReply {
    fn message_type(&self) -> u16 {
        Self::TYPE
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_bytes(self.version.as_bytes())
    }
}

impl<I> MessageParse<I> for BootloaderReadyReply
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| {
            let input = if typ != Self::TYPE {
                nom::combinator::fail::<_, (), _>(input)?.0
            } else {
                input
            };

            let (input, version) = parse_version(input)?;

            Ok((input, BootloaderReadyReply { version }))
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::crc::CrcXModem;
    use super::super::obfuscation::Deobfuscated;
    use super::super::{parse, serialize};
    use super::*;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for crate::Version {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut version = Vec::<u8>::arbitrary(g);
            version.truncate(crate::VERSION_LEN);
            crate::Version::from_bytes(&version).unwrap()
        }
    }

    fn roundtrip<M>(msg: M) -> bool
    where
        M: for<'a> MessageParse<Deobfuscated<&'a [u8]>> + MessageSerialize + PartialEq + Eq,
    {
        let a = roundtrip_a(&msg);
        let b = a.as_ref().and_then(|ser| roundtrip_b(ser));
        Some(msg) == b
    }

    fn roundtrip_a<M>(msg: &M) -> Option<Vec<u8>>
    where
        M: MessageSerialize,
    {
        let crc = CrcXModem::new();
        let mut serialized = Vec::new();
        serialize(&crc, &mut serialized, msg)
            .ok()
            .map(|_| serialized)
    }

    fn roundtrip_b<'a, M>(serialized: &'a [u8]) -> Option<M>
    where
        M: MessageParse<Deobfuscated<&'a [u8]>>,
    {
        let crc = CrcXModem::new();
        let (amt, unserialized) = parse(&crc, &serialized[..]);
        if amt != serialized.len() {
            None
        } else {
            unserialized.ignore_error()
        }
    }

    impl Arbitrary for Hello {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                session_id: u32::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn roundtrip_hello(msg: Hello) -> bool {
        roundtrip(msg)
    }

    impl Arbitrary for HelloReply {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                version: crate::Version::arbitrary(g),
                has_custom_aes_key: bool::arbitrary(g),
                is_in_lock_screen: bool::arbitrary(g),
                padding: [u8::arbitrary(g), u8::arbitrary(g)],
                challenge: [
                    u32::arbitrary(g),
                    u32::arbitrary(g),
                    u32::arbitrary(g),
                    u32::arbitrary(g),
                ],
            }
        }
    }

    #[quickcheck]
    fn roundtrip_hello_reply(msg: HelloReply) -> bool {
        roundtrip(msg)
    }

    impl Arbitrary for BootloaderReady {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                chip_id: [
                    u32::arbitrary(g),
                    u32::arbitrary(g),
                    u32::arbitrary(g),
                    u32::arbitrary(g),
                ],
                version: crate::Version::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn roundtrip_bootloader_ready(msg: BootloaderReady) -> bool {
        roundtrip(msg)
    }

    impl Arbitrary for WriteFlash<Vec<u8>> {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut data = Vec::<u8>::arbitrary(g);
            data.truncate(WRITE_FLASH_LEN);
            data.resize(WRITE_FLASH_LEN, 0);
            Self {
                session_id: u32::arbitrary(g),
                page: u16::arbitrary(g),
                max_page: u16::arbitrary(g),
                len: data.len() as u16,
                padding: [u8::arbitrary(g), u8::arbitrary(g)],
                data,
            }
        }
    }

    #[quickcheck]
    fn roundtrip_write_flash(msg: WriteFlash<Vec<u8>>) -> bool {
        let a = roundtrip_a(&msg.map_ref(|d| &d[..]));
        let b = a
            .as_ref()
            .and_then(|ser| roundtrip_b(&ser))
            .map(|m: WriteFlash<Deobfuscated<_>>| m.map(|d| d.to_vec()));
        Some(msg) == b
    }

    impl Arbitrary for WriteFlashReply {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                session_id: u32::arbitrary(g),
                page: u16::arbitrary(g),
                error: u16::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn roundtrip_write_flash_reply(msg: WriteFlashReply) -> bool {
        roundtrip(msg)
    }

    impl Arbitrary for ReadEeprom {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                address: u16::arbitrary(g),
                len: u8::arbitrary(g),
                padding: u8::arbitrary(g),
                session_id: u32::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn roundtrip_read_eeprom(msg: ReadEeprom) -> bool {
        roundtrip(msg)
    }

    impl Arbitrary for ReadEepromReply<Vec<u8>> {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut data = Vec::<u8>::arbitrary(g);
            data.truncate(0xff);
            Self {
                address: u16::arbitrary(g),
                len: data.len() as u8,
                padding: u8::arbitrary(g),
                data,
            }
        }
    }

    #[quickcheck]
    fn roundtrip_read_eeprom_reply(msg: ReadEepromReply<Vec<u8>>) -> bool {
        let a = roundtrip_a(&msg.map_ref(|d| &d[..]));
        let b = a
            .as_ref()
            .and_then(|ser| roundtrip_b(&ser))
            .map(|m: ReadEepromReply<Deobfuscated<_>>| m.map(|d| d.to_vec()));
        Some(msg) == b
    }

    impl Arbitrary for BootloaderReadyReply {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                version: crate::Version::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn roundtrip_bootloader_ready_reply(msg: BootloaderReadyReply) -> bool {
        roundtrip(msg)
    }
}
