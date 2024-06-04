//! Messages used in the stock bootloader.

use nom::{error::Error, Parser};

use crate::protocol::parse::{MessageParse, Parse};
use crate::protocol::serialize::{MessageSerialize, Serializer};

use super::{util, MessageType};

/// 0x0518 Bootloader Ready, radio message (bootloader mode).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    I: Parse,
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
            let (input, chip_id) = util::parse_array(nom::number::complete::le_u32)(input)?;
            let (input, version) = util::parse_version(input)?;

            Ok((input, BootloaderReady { chip_id, version }))
        }
    }
}

/// Known good session ID for flash messages. Used in [WriteFlash].
pub const WRITE_FLASH_SESSION_ID: u32 = 0x1d9f8d8a;

/// Size of the data in a [WriteFlash] message.
pub const WRITE_FLASH_LEN: usize = 0x100;

/// 0x0519 Write Flash, host message (bootloader mode).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WriteFlash<I> {
    /// Session ID unique to this flash sequence. Use
    /// [WRITE_FLASH_SESSION_ID] if unsure. This must be the same
    /// for all writes in a given session.
    pub session_id: u32,
    /// Which 0x100 byte page to write. Increments by 1 each message.
    pub page: u16,
    /// Maximum flash page, exclusive. Device boots after writing when
    /// `page + 1 == max_page`.
    pub max_page: u16,
    /// Length of data. Note `data.len()` is always 0x100, this
    /// field instead indicates how much data inside is used.
    ///
    /// This seems to be ignored by the bootloader.
    pub len: u16,
    /// Alignment padding.
    pub _pad: util::Padding<2>,
    /// Data to write to flash. Must be 0x100 / [WRITE_FLASH_LEN] bytes!
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
            _pad: self._pad,
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
            _pad: self._pad,
            data: f(&self.data),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> WriteFlash<I::Owned>
    where
        I: alloc::borrow::ToOwned,
    {
        self.map_ref(I::to_owned)
    }

    pub fn borrow<Borrowed: ?Sized>(&self) -> WriteFlash<&Borrowed>
    where
        I: core::borrow::Borrow<Borrowed>,
    {
        self.map_ref(I::borrow)
    }
}

impl<I> MessageSerialize for WriteFlash<I>
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
        ser.write_le_u32(self.session_id)?;
        ser.write_le_u16(self.page)?;
        ser.write_le_u16(self.max_page)?;
        ser.write_le_u16(self.len)?;
        self._pad.serialize(ser)?;

        // I don't like this assert, but this is better than
        // sending a malformed packet to the bootloader. probably.
        assert_eq!(self.data.input_len(), WRITE_FLASH_LEN);
        ser.write_slice(&self.data)
    }
}

impl<I> MessageParse<I> for WriteFlash<I>
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
            let (input, page) = nom::number::complete::le_u16(input)?;
            let (input, max_page) = nom::number::complete::le_u16(input)?;
            let (input, len) = nom::number::complete::le_u16(input)?;

            let (input, _pad) = util::Padding::parse(input)?;

            // message always has 0x100 bytes here, regardless of len
            let (input, data) = nom::bytes::complete::take(WRITE_FLASH_LEN)(input)?;
            Ok((
                input,
                WriteFlash {
                    session_id,
                    page,
                    max_page,
                    len,
                    _pad,
                    data,
                },
            ))
        }
    }
}

/// 0x051a Write Flash Reply, radio message (bootloader mode).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WriteFlashReply {
    /// Session ID, matches the session id sent in the [WriteFlash] message.
    pub session_id: u32,
    /// Page number, matches the page sent in the [WriteFlash] message.
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
    I: Parse,
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

/// 0x0530 Bootloader Ready Reply, host message (bootloader mode).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

            Ok((input, BootloaderReadyReply { version }))
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
                _pad: util::Padding::arbitrary(g),
                data,
            }
        }
    }

    #[quickcheck]
    fn roundtrip_write_flash(msg: WriteFlash<Vec<u8>>) -> bool {
        RoundTrip::new().run(&msg.borrow())
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
