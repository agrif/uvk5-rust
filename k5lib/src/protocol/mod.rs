pub const FRAME_START: [u8; 2] = [0xab, 0xcd];
pub const FRAME_END: [u8; 2] = [0xdc, 0xba];

/// Total guess, here.
pub const MAX_FRAME_SIZE: usize = 0x200;

pub const BAUD_RATE: u32 = 38400;

pub mod crc;

pub mod obfuscation;

pub mod parse;
pub use parse::{InputParse, MessageParse, ParseResult};

mod messages;
pub use messages::*;

pub mod serialize;
pub use serialize::MessageSerialize;

/// Parse an entire frame containing a message, skipping data before
/// the frame. If the frame doesn't parse as this message, or the CRC
/// fails, it will still consume the frame from the input.
///
/// Returns the *number of consumed bytes* and the parse or CRC
/// result. Note this is different from the MessageParse trait!
///
/// This includes frame start/end, length, obfuscation, and CRC.
pub fn parse<C, I, M>(crc: &C, input: I) -> (usize, ParseResult<I, M>)
where
    C: crc::CrcStyle,
    I: InputParse,
    M: MessageParse<obfuscation::Deobfuscated<I>>,
{
    let start_len = input.input_len();
    let (rest, res) = parse::message_parse_frame(&crc, input);

    (start_len - rest.input_len(), res)
}

/// Serialize a message into a full frame, with obfuscation, CRC, and
/// start/end markers.
pub fn serialize<C, W, M>(crc: &C, writer: &mut W, message: &M) -> std::io::Result<()>
where
    C: crc::CrcStyle,
    W: std::io::Write,
    M: MessageSerialize,
{
    let mut ser = serialize::SerializerWrap::new(writer);
    message.frame(&crc, &mut ser)
}
