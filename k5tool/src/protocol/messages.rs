/// A helpful short name for a whole bundle of useful parser traits.
pub trait InputParse:
    nom::InputTakeAtPosition<Item = u8>
    + nom::Compare<&'static [u8]>
    + nom::InputLength
    + nom::InputTake
    + nom::InputIter<Item = u8>
    + nom::Slice<std::ops::RangeFrom<usize>>
    + Clone
    + PartialEq
{
}

impl<T> InputParse for T where
    T: nom::InputTakeAtPosition<Item = u8>
        + nom::Compare<&'static [u8]>
        + nom::InputLength
        + nom::InputTake
        + nom::InputIter<Item = u8>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + Clone
        + PartialEq
{
}

/// Any kind of message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Message {
    Host(HostMessage),
    Radio(RadioMessage),
}

/// Parses any message, given the message type.
pub fn any_message_body<I>(typ: u16) -> impl FnMut(I) -> nom::IResult<I, Message>
where
    I: InputParse,
{
    nom::branch::alt((
        nom::combinator::map(host_message_body(typ), Message::Host),
        nom::combinator::map(radio_message_body(typ), Message::Radio),
    ))
}

/// Messages sent from the host computer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HostMessage {
    /// 0x0514 Hello
    Hello(Hello),
}

/// Parses a host message, given the message type.
pub fn host_message_body<I>(typ: u16) -> impl FnMut(I) -> nom::IResult<I, HostMessage>
where
    I: InputParse,
{
    move |input| match typ {
        0x0514 => Hello::parse(input).map(|(i, r)| (i, HostMessage::Hello(r))),

        // we don't recognize the message type
        _ => nom::combinator::fail(input),
    }
}

/// Messages sent from the radio.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RadioMessage {
    Version(Version),
}

/// Parses a radio message, given the message type.
pub fn radio_message_body<I>(typ: u16) -> impl FnMut(I) -> nom::IResult<I, RadioMessage>
where
    I: InputParse,
{
    move |input| match typ {
        0x0515 => Version::parse(input).map(|(i, r)| (i, RadioMessage::Version(r))),

        // we don't recognize the message type
        _ => nom::combinator::fail(input),
    }
}

/// A trait for information about message types.
pub trait MessageInfo: Sized {
    /// Message type.
    fn message_type() -> u16;
}

/// 0x0514 Hello, host message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hello {
    /// Timestamp on all host messages. All further messages must use
    /// this same timestamp or they will be ignored.
    timestamp: u32,
}

impl MessageInfo for Hello {
    fn message_type() -> u16 {
        0x0514
    }
}

impl Hello {
    fn parse<I>(input: I) -> nom::IResult<I, Self>
    where
        I: InputParse,
    {
        let (input, timestamp) = nom::number::complete::le_u32(input)?;
        Ok((input, Hello { timestamp }))
    }
}

/// 0x0515 Version, radio message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    /// Version, provided by the radio.
    /// Assume UTF-8, or at least, ASCII, padded by zeros.
    version: crate::Version,

    /// Radio is using custom AES key.
    has_custom_aes_key: bool,

    /// Radio is in the lock screen.
    is_in_lock_screen: bool,

    /// Unknown or unused.
    padding: [u8; 2],

    /// AES challenge. See 0x052D.
    challenge: [u32; 4],
}

impl MessageInfo for Version {
    fn message_type() -> u16 {
        0x0515
    }
}

impl Version {
    fn parse<I>(input: I) -> nom::IResult<I, Self>
    where
        I: InputParse,
    {
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
