//! Messages used in the stock radio firmware.

use nom::{error::Error, Parser};

use crate::protocol::parse::{MessageParse, Parse};
use crate::protocol::serialize::{MessageSerialize, Serializer};

use super::{util, MessageType};

/// Known good session ID for host messages. Introduced by [Hello].
pub const HELLO_SESSION_ID: u32 = 0x6457396a;

/// 0x0514 Hello, host message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Hello {
    /// Session ID on all host messages. All further messages must use
    /// this same ID or they will be ignored.
    ///
    /// If unsure, use [HELLO_SESSION_ID].
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
    I: Parse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>>
    where
        I: Parse,
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HelloReply {
    /// Version provided by the radio.
    /// Assume UTF-8, or at least, ASCII, padded by zeros.
    pub version: crate::Version,

    /// Radio is using custom AES key.
    pub has_custom_aes_key: bool,

    /// Radio is in the lock screen.
    pub is_in_lock_screen: bool,

    /// Alignment padding.
    pub _pad: util::Padding<2>,

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
        self._pad.serialize(ser)?;
        for c in self.challenge.iter() {
            ser.write_le_u32(*c)?;
        }
        Ok(())
    }
}

impl<I> MessageParse<I> for HelloReply
where
    I: Parse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| {
            let input = if typ != Self::TYPE {
                nom::combinator::fail::<_, (), _>(input)?.0
            } else {
                input
            };

            let (input, version) = util::parse_version(input)?;

            let (input, has_custom_aes_key) = nom::number::complete::u8(input)?;
            let has_custom_aes_key = has_custom_aes_key > 0;

            let (input, is_in_lock_screen) = nom::number::complete::u8(input)?;
            let is_in_lock_screen = is_in_lock_screen > 0;

            let (input, _pad) = util::Padding::parse(input)?;
            let (input, challenge) = util::parse_array(nom::number::complete::le_u32)(input)?;

            Ok((
                input,
                HelloReply {
                    version,
                    has_custom_aes_key,
                    is_in_lock_screen,
                    _pad,
                    challenge,
                },
            ))
        }
    }
}

/// 0x051b Read EEPROM, host message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadEeprom {
    /// Address to read.
    pub address: u16,
    /// Number of bytes to read from address, usually 0x80.
    pub len: u8,
    /// Alignment padding.
    pub _pad: util::Padding<1>,
    /// Session ID, must match the one provided by initial [Hello].
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
        self._pad.serialize(ser)?;
        ser.write_le_u32(self.session_id)
    }
}

impl<I> MessageParse<I> for ReadEeprom
where
    I: Parse,
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
            let (input, _pad) = util::Padding::parse(input)?;
            let (input, session_id) = nom::number::complete::le_u32(input)?;
            Ok((
                input,
                ReadEeprom {
                    address,
                    len,
                    _pad,
                    session_id,
                },
            ))
        }
    }
}

/// 0x051c Read Eeprom Reply, radio message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadEepromReply<I> {
    /// Address of data read.
    pub address: u16,
    /// Number of bytes of data read.
    pub len: u8,
    /// Alignment padding.
    pub _pad: util::Padding<1>,
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
            _pad: self._pad,
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
            _pad: self._pad,
            data: f(&self.data),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> ReadEepromReply<I::Owned>
    where
        I: alloc::borrow::ToOwned,
    {
        self.map_ref(I::to_owned)
    }

    pub fn borrow<Borrowed: ?Sized>(&self) -> ReadEepromReply<&Borrowed>
    where
        I: core::borrow::Borrow<Borrowed>,
    {
        self.map_ref(I::borrow)
    }
}

impl<I> MessageSerialize for ReadEepromReply<I>
where
    I: Parse,
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
        self._pad.serialize(ser)?;
        ser.write_slice(&self.data)
    }
}

impl<I> MessageParse<I> for ReadEepromReply<I>
where
    I: Parse,
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
            let (input, _pad) = util::Padding::parse(input)?;
            let (input, data) = nom::bytes::complete::take(len as usize)(input)?;
            Ok((
                input,
                ReadEepromReply {
                    address,
                    len,
                    _pad,
                    data,
                },
            ))
        }
    }
}

#[cfg(test)]
#[cfg(feature = "alloc")]
mod test {
    use alloc::vec::Vec;

    use super::super::test::*;
    use super::*;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

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
                _pad: util::Padding::arbitrary(g),
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

    impl Arbitrary for ReadEeprom {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                address: u16::arbitrary(g),
                len: u8::arbitrary(g),
                _pad: util::Padding::arbitrary(g),
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
                _pad: util::Padding::arbitrary(g),
                data,
            }
        }
    }

    #[quickcheck]
    fn roundtrip_read_eeprom_reply(msg: ReadEepromReply<Vec<u8>>) -> bool {
        RoundTrip::new().run(&msg.borrow())
    }
}
