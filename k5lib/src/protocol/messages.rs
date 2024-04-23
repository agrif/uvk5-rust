use nom::{error::Error, Parser};

use super::parse::{InputParse, MessageParse};
use super::serialize::{MessageSerialize, Serializer};

/// Any kind of message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Message<I> {
    Host(HostMessage),
    Radio(RadioMessage<I>),
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
pub enum HostMessage {
    /// 0x0514 Hello
    Hello(Hello),
    /// 0x051b Read EEPROM
    ReadEeprom(ReadEeprom),
}

impl<I> MessageParse<I> for HostMessage
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| match typ {
            0x0514 => Hello::parse_body(typ).map(HostMessage::Hello).parse(input),
            0x051b => ReadEeprom::parse_body(typ)
                .map(HostMessage::ReadEeprom)
                .parse(input),

            // we don't recognize the message type
            _ => nom::combinator::fail(input),
        }
    }
}

/// Messages sent from the radio.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RadioMessage<I> {
    /// 0x0515 Version
    Version(Version),
    /// 0x0518 Bootloader Ready (bootloader mode)
    BootloaderReady(BootloaderReady),
    /// 0x51c Read EEPROM Reply
    ReadEepromReply(ReadEepromReply<I>),
}

impl<I> MessageParse<I> for RadioMessage<I>
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| match typ {
            0x0515 => Version::parse_body(typ)
                .map(RadioMessage::Version)
                .parse(input),
            0x0518 => BootloaderReady::parse_body(typ)
                .map(RadioMessage::BootloaderReady)
                .parse(input),
            0x051c => ReadEepromReply::parse_body(typ)
                .map(RadioMessage::ReadEepromReply)
                .parse(input),

            // we don't recognize the message type
            _ => nom::combinator::fail(input),
        }
    }
}

/// 0x0514 Hello, host message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hello {
    /// Timestamp on all host messages. All further messages must use
    /// this same timestamp or they will be ignored.
    pub timestamp: u32,
}

impl MessageSerialize for Hello {
    fn message_type(&self) -> u16 {
        0x0514
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_le_u32(self.timestamp)
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
        assert_eq!(typ, 0x0514);
        move |input| {
            let (input, timestamp) = nom::number::complete::le_u32(input)?;
            Ok((input, Hello { timestamp }))
        }
    }
}

/// 0x0515 Version, radio message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    /// Version, provided by the radio.
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

impl MessageSerialize for Version {
    fn message_type(&self) -> u16 {
        0x0515
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

impl<I> MessageParse<I> for Version
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        assert_eq!(typ, 0x0515);
        move |input| {
            let mut version = crate::Version::new_empty();
            let (input, _) =
                nom::multi::fill(nom::number::complete::u8, version.as_mut_bytes())(input)?;

            let (input, has_custom_aes_key) = nom::number::complete::u8(input)?;
            let has_custom_aes_key = has_custom_aes_key > 0;

            let (input, is_in_lock_screen) = nom::number::complete::u8(input)?;
            let is_in_lock_screen = is_in_lock_screen > 0;

            let mut padding = [0; 2];
            let (input, _) = nom::multi::fill(nom::number::complete::u8, &mut padding[..])(input)?;

            let mut challenge = [0; 4];
            let (input, _) =
                nom::multi::fill(nom::number::complete::le_u32, &mut challenge[..])(input)?;

            Ok((
                input,
                Version {
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

impl MessageSerialize for BootloaderReady {
    fn message_type(&self) -> u16 {
        0x0518
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
        assert_eq!(typ, 0x0518);
        move |input| {
            // FIXME some bootloaders have different packet formats
            // I suspect they vary the chip_id field size, but...
            // I don't have any examples, so I can't know.
            let mut chip_id = [0; 4];
            let (input, _) =
                nom::multi::fill(nom::number::complete::le_u32, &mut chip_id[..])(input)?;

            let mut version = crate::Version::new_empty();
            let (input, _) =
                nom::multi::fill(nom::number::complete::u8, version.as_mut_bytes())(input)?;

            Ok((input, BootloaderReady { chip_id, version }))
        }
    }
}

/// 0x051b Read EEPROM, host message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadEeprom {
    /// Address to read.
    pub address: u16,
    /// Number of bytes to read from address.
    pub len: u8,
    /// Unknown or unused.
    pub padding: u8,
    /// Timestamp, must match the one provided by initial Hello.
    pub timestamp: u32,
}

impl MessageSerialize for ReadEeprom {
    fn message_type(&self) -> u16 {
        0x051b
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_le_u16(self.address)?;
        ser.write_u8(self.len)?;
        ser.write_u8(self.padding)?;
        ser.write_le_u32(self.timestamp)
    }
}

impl<I> MessageParse<I> for ReadEeprom
where
    I: InputParse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        assert_eq!(typ, 0x051b);
        move |input| {
            let (input, address) = nom::number::complete::le_u16(input)?;
            let (input, len) = nom::number::complete::u8(input)?;
            let (input, padding) = nom::number::complete::u8(input)?;
            let (input, timestamp) = nom::number::complete::le_u32(input)?;
            Ok((
                input,
                ReadEeprom {
                    address,
                    len,
                    padding,
                    timestamp,
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
        0x051c
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
        assert_eq!(typ, 0x051c);
        move |input| {
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

#[cfg(test)]
mod test {
    use super::super::crc::CrcXModem;
    use super::super::obfuscation::Deobfuscated;
    use super::super::{parse, serialize};
    use super::*;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

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
        let (rest, unserialized) = parse(&crc, &serialized[..]);
        if rest.len() != 0 {
            None
        } else {
            unserialized.ignore_error()
        }
    }

    impl Arbitrary for Hello {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                timestamp: u32::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn roundtrip_hello(msg: Hello) -> bool {
        roundtrip(msg)
    }

    impl Arbitrary for Version {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut version = Vec::<u8>::arbitrary(g);
            version.truncate(crate::VERSION_LEN);

            Self {
                version: crate::Version::from_bytes(&version).unwrap(),
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
    fn roundtrip_version(msg: Version) -> bool {
        roundtrip(msg)
    }

    impl Arbitrary for BootloaderReady {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut version = Vec::<u8>::arbitrary(g);
            version.truncate(crate::VERSION_LEN);

            Self {
                chip_id: [
                    u32::arbitrary(g),
                    u32::arbitrary(g),
                    u32::arbitrary(g),
                    u32::arbitrary(g),
                ],
                version: crate::Version::from_bytes(&version).unwrap(),
            }
        }
    }

    #[quickcheck]
    fn roundtrip_bootloader_ready(msg: BootloaderReady) -> bool {
        roundtrip(msg)
    }

    impl Arbitrary for ReadEeprom {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                address: u16::arbitrary(g),
                len: u8::arbitrary(g),
                padding: u8::arbitrary(g),
                timestamp: u32::arbitrary(g),
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
}
