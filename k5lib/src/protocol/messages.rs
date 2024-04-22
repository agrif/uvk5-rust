use nom::{error::Error, Parser};

use super::{InputParse, MessageParse, MessageSerialize, Serializer};

/// Any kind of message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Message {
    Host(HostMessage),
    Radio(RadioMessage),
}

impl MessageParse for Message {
    fn parse_body<I>(typ: u16) -> impl nom::Parser<I, Self, Error<I>>
    where
        I: InputParse,
    {
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
}

impl MessageParse for HostMessage {
    fn parse_body<I>(typ: u16) -> impl nom::Parser<I, Self, Error<I>>
    where
        I: InputParse,
    {
        move |input| match typ {
            0x0514 => Hello::parse_body(typ).map(HostMessage::Hello).parse(input),

            // we don't recognize the message type
            _ => nom::combinator::fail(input),
        }
    }
}

/// Messages sent from the radio.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RadioMessage {
    Version(Version),
}

impl MessageParse for RadioMessage {
    fn parse_body<I>(typ: u16) -> impl nom::Parser<I, Self, Error<I>>
    where
        I: InputParse,
    {
        move |input| match typ {
            0x0515 => Version::parse_body(typ)
                .map(RadioMessage::Version)
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

impl MessageParse for Hello {
    fn parse_body<I>(typ: u16) -> impl nom::Parser<I, Self, Error<I>>
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

impl MessageParse for Version {
    fn parse_body<I>(typ: u16) -> impl nom::Parser<I, Self, Error<I>>
    where
        I: InputParse,
    {
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

#[cfg(test)]
mod test {
    use super::*;

    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    fn roundtrip<M>(msg: M) -> bool
    where
        M: MessageParse + MessageSerialize + PartialEq + Eq,
    {
        let crc = super::super::CrcXModem::new();
        let mut ser = super::super::SerializerWrap::new(Vec::new());
        msg.frame(&mut ser, &crc).unwrap();
        let serialized = ser.done();

        let (rest, unserialized) = M::parse_frame(&crc, &serialized[..]);
        let unserialized = unserialized.ignore_error().unwrap();
        return (rest.len() == 0) && (msg == unserialized);
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
}
