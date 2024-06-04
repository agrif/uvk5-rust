/// Frame start sequence.
pub const FRAME_START: [u8; 2] = [0xab, 0xcd];

/// Frame end sequence.
pub const FRAME_END: [u8; 2] = [0xdc, 0xba];

/// Largest size for an entire frame.
///
/// This is an educated
/// guess. [WriteFlash][messages::bootloader::WriteFlash] payload is
/// at most 0x100 bytes, so this gives a little bit of wiggle room on
/// top of that.
pub const MAX_FRAME_SIZE: usize = 0x200;

/// Default baud rate for radio UART.
pub const BAUD_RATE: u32 = 38400;

pub mod crc;

pub mod obfuscation;

pub mod parse;
pub use parse::{MessageParse, Parse, ParseMut, ParseResult};

pub mod messages;
pub use messages::{HostMessage, Message, MessageType, RadioMessage};

pub mod serialize;
pub use serialize::{MessageSerialize, Serializer};

/// Find a frame, skipping data before the frame. If no frame is
/// found, return None but still consume the input.
///
/// Regardless of whether a frame is found, the token returned by this
/// function can be passed to [parse()] to parse any found frame into
/// a message.
pub fn find_frame<I>(input: I) -> (usize, Option<parse::FoundFrame>)
where
    I: ParseMut,
{
    parse::find_frame(input)
}

/// Parse an entire frame containing a message, checking the CRC. If
/// the frame doesn't parse as this message, or the CRC fails, it will
/// return that error.
///
/// Call this with the result of [find_frame()].
pub fn parse<C, I, M>(crc: &C, input: I, found: &Option<parse::FoundFrame>) -> ParseResult<I, M>
where
    C: crc::CrcStyle,
    I: Parse,
    M: MessageParse<I>,
{
    M::parse_frame(&crc, input, found)
}

/// Serialize a message into a full frame, with obfuscation, CRC, and
/// start/end markers.
pub fn serialize<C, S, M>(crc: &C, serializer: &mut S, message: &M) -> Result<(), S::Error>
where
    C: crc::CrcStyle,
    S: Serializer,
    M: MessageSerialize,
{
    message.frame(&crc, serializer)
}
