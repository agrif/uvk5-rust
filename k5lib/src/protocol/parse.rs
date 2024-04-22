use nom::error::Error;
use nom::IResult;

use super::crc::CrcStyle;
use super::obfuscation::Deobfuscated;
use super::{FRAME_END, FRAME_START, MAX_FRAME_SIZE};

/// A helpful short name for a whole bundle of useful parser traits.
pub trait InputParse:
    nom::InputTakeAtPosition<Item = u8>
    + nom::Compare<&'static [u8]>
    + nom::InputLength
    + nom::InputTake
    + nom::InputIter<Item = u8>
    + nom::Slice<std::ops::Range<usize>>
    + nom::Slice<std::ops::RangeFrom<usize>>
    + nom::Slice<std::ops::RangeFull>
    + nom::Slice<std::ops::RangeTo<usize>>
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
        + nom::Slice<std::ops::Range<usize>>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeFull>
        + nom::Slice<std::ops::RangeTo<usize>>
        + Clone
        + PartialEq
{
}

/// Eats input until it sees a frame start, then leaves it intact.
///
/// Returns the unconsumed input and true when it found a frame start,
/// false otherwise. Wrap in Result::Ok to turn this into a nom parser.
fn frame_start<I>(input: I) -> (I, bool)
where
    I: InputParse,
{
    let mut loop_input = input;
    loop {
        // parse completely everything that isn't a first byte of frame start
        // careful: is_not will fail on empty strings, but we want success
        let rest =
            nom::bytes::complete::is_not::<_, _, Error<I>>(&FRAME_START[0..1])(loop_input.clone())
                .map(|(r, _)| r)
                .unwrap_or(loop_input);

        // save this for later, this is before the frame starts
        let pre_frame_rest = rest.clone();

        // try to parse a complete first byte
        let first: IResult<_, _, Error<I>> =
            nom::bytes::complete::tag(&FRAME_START[0..1])(rest.clone());
        if let Ok((rest, _)) = first {
            // ok, now parse the rest of the frame start
            // but use streaming, we may have incomplete data
            match nom::bytes::streaming::tag(&FRAME_START[1..])(rest.clone()) {
                Ok(_) => {
                    // we saw the rest of the frame start! return true
                    return (pre_frame_rest, true);
                }
                Err(nom::Err::Incomplete(_)) => {
                    // we didn't see the rest, but we might later, return false
                    return (pre_frame_rest, false);
                }
                Err(e) => {
                    let _: nom::Err<Error<_>> = e;
                    // we just fully do not match the rest, so keep looking
                    // for a real frame start
                    loop_input = rest;
                    continue;
                }
            }
        } else {
            // we didn't see a first byte, no frame start here
            // this can only happen at the end of input
            // because we parsed until !(first_char) then tried first_char
            return (rest, false);
        }
    }
}

/// Find a frame, and return the contents of that frame (still
/// obfuscated and CRC'd).
///
/// Returns unconsumed input and None if it only skipped data and
/// found no complete frames. Wrap in Result::Ok to turn this into a
/// nom parser.
fn frame_raw<I>(input: I) -> (I, Option<I>)
where
    I: InputParse,
{
    // a chunk of data, prefixed by a little-endian u16 length
    // only succeeds when frame is smaller than MAX_FRAME_SIZE
    let data = nom::combinator::flat_map(
        nom::combinator::verify(nom::number::streaming::le_u16::<I, Error<I>>, |l| {
            *l < MAX_FRAME_SIZE as u16
        }),
        // CRC at the end adds two bytes
        |l| nom::bytes::streaming::take(l + 2),
    );

    // the above chunk, inside FRAME_START and FRAME_END
    let mut delimited = nom::sequence::delimited(
        nom::bytes::streaming::tag(&FRAME_START[..]),
        data,
        nom::bytes::streaming::tag(&FRAME_END[..]),
    );

    // keep looking until we find a chunk or run out
    let mut loop_input = input;
    loop {
        // unwrap: frame_start never returns an Err.
        let (rest, frame_found) = frame_start(loop_input);
        if frame_found {
            // we found a frame, try the whole message
            match delimited(rest.clone()) {
                Ok((rest, result)) => {
                    // hey, it worked!
                    return (rest, Some(result));
                }
                Err(nom::Err::Incomplete(_)) => {
                    // it didn't work now, but it might later
                    return (rest, None);
                }
                Err(_) => {
                    // it didn't work and won't, ever
                    // skip the frame start, so we don't get stuck here again
                    // use complete because we know it's there
                    let (rest, _) =
                        nom::bytes::complete::take::<usize, I, Error<I>>(FRAME_START.len())(rest)
                            .map_err(|_| "take FRAME_START unexpected Err")
                            .unwrap();
                    // try again
                    loop_input = rest;
                }
            }
        } else {
            // no frame found, only skipped
            return (rest, None);
        }
    }
}

/// A possible result from framed().
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParseResult<I, O, E = Error<Deobfuscated<I>>> {
    /// Frame parse result
    Ok(O),
    /// (Original frame without CRC, Error)
    ParseErr(Deobfuscated<I>, E),
    /// CRC check failed, with whole original frame including CRC.
    CrcErr(Deobfuscated<I>),
    /// Only non-frame input was consumed.
    None,
}

impl<I, O, E> ParseResult<I, O, E> {
    pub fn ignore_error(self) -> Option<O> {
        match self {
            Self::Ok(o) => Some(o),
            Self::ParseErr(_, _) => None,
            Self::CrcErr(_) => None,
            Self::None => None,
        }
    }

    pub fn map<F, Op>(self, f: F) -> ParseResult<I, Op, E>
    where
        F: FnOnce(O) -> Op,
    {
        match self {
            Self::Ok(o) => ParseResult::Ok(f(o)),
            Self::ParseErr(frame, err) => ParseResult::ParseErr(frame, err),
            Self::CrcErr(frame) => ParseResult::CrcErr(frame),
            Self::None => ParseResult::None,
        }
    }

    pub fn map_parse_err<F, Ep>(self, f: F) -> ParseResult<I, O, Ep>
    where
        F: FnOnce(E) -> Ep,
    {
        match self {
            Self::Ok(o) => ParseResult::Ok(o),
            Self::ParseErr(frame, err) => ParseResult::ParseErr(frame, f(err)),
            Self::CrcErr(frame) => ParseResult::CrcErr(frame),
            Self::None => ParseResult::None,
        }
    }
}

/// Find a frame, and deobfuscate and then parse with the provided parser.
///
/// The parser is always run against an entire frame.
///
/// Returns unconsumed input and Ok(..) on successful parse,
/// ParseErr(..) if the provided parser failed, CrcErr(..) if the
/// checksum was wrong, and None if it only skipped data and found no
/// complete frames.  In all cases, any frames passed to the parser
/// are removed from the input.
///
/// Wrap in Result::Ok to turn this into a nom parser.
pub fn framed<C, I, P, O>(crc: C, parser: P) -> impl FnMut(I) -> (I, ParseResult<I, O>)
where
    C: CrcStyle,
    P: nom::Parser<Deobfuscated<I>, O, Error<Deobfuscated<I>>>,
    I: InputParse,
{
    let mut parser_all = nom::combinator::all_consuming(parser);

    move |input| {
        let (rest, maybe_content) = frame_raw(input);
        match maybe_content {
            Some(content) => {
                // found a frame, wrap it and feed it to our parser
                let deobfuscated = Deobfuscated::new(content);
                if let Some(body) = deobfuscated.check_crc(&crc) {
                    match parser_all(body.clone()) {
                        Ok((_, result)) => (rest, ParseResult::Ok(result)),
                        Err(e) => match e {
                            nom::Err::Incomplete(_) => (
                                rest,
                                ParseResult::ParseErr(
                                    body.clone(),
                                    Error {
                                        input: body,
                                        code: nom::error::ErrorKind::Complete,
                                    },
                                ),
                            ),
                            nom::Err::Error(e) => (rest, ParseResult::ParseErr(body, e)),
                            nom::Err::Failure(e) => (rest, ParseResult::ParseErr(body, e)),
                        },
                    }
                } else {
                    (rest, ParseResult::CrcErr(deobfuscated))
                }
            }
            None => {
                // no frame found, only ate input
                (rest, ParseResult::None)
            }
        }
    }
}

/// Parse a message type and length, and provide it to a function that
/// returns a parser for that message type.
pub fn message<I, F, P, O>(mut parser: F) -> impl FnMut(I) -> IResult<I, O>
where
    F: FnMut(u16) -> P,
    P: nom::Parser<I, O, Error<I>>,
    I: InputParse,
{
    move |input| {
        // u16le message type
        let (rest, typ) = nom::number::complete::le_u16(input)?;
        // u16le message length (which should be everything)
        // we could use all_consuming here, but:
        //  1. this will fail if there is not enough data
        //  2. if this is wrapped in framed(..), it will also fail if there
        //     is too much data.
        // So, we don't.
        let (_, body) = nom::multi::length_data(nom::number::complete::le_u16)(rest)?;
        parser(typ).parse(body)
    }
}

/// A trait for parseable messages.
pub trait MessageParse: Sized {
    /// Parse the body of a message, given the message type.
    fn parse_body<I>(typ: u16) -> impl nom::Parser<I, Self, Error<I>>
    where
        I: InputParse;

    /// Parse an entire message, including type and length header.
    fn parse_frame_body<I>() -> impl nom::Parser<I, Self, Error<I>>
    where
        I: InputParse,
    {
        message(Self::parse_body)
    }

    /// Parse an entire frame containing a message, skipping data
    /// before the frame. If the frame doesn't parse as this message,
    /// or the CRC fails, it will still consume the frame from the
    /// input.
    ///
    /// Returns the unconsumed input and the parse or CRC result.
    ///
    /// This includes frame start/end, length, obfuscation, and CRC.
    ///
    /// Wrap in Result::Ok to turn this into a nom parser.
    fn parse_frame<C, I>(crc: &C, input: I) -> (I, ParseResult<I, Self>)
    where
        C: CrcStyle,
        I: InputParse,
    {
        framed(crc, Self::parse_frame_body())(input)
    }
}

#[cfg(test)]
mod test {
    use super::super::crc::CrcConstant;
    use super::*;

    #[test]
    fn frame_start_empty() {
        assert_eq!(frame_start(b"".as_ref()), (b"".as_ref(), false));
    }

    #[test]
    fn frame_start_discard_garbage() {
        assert_eq!(frame_start(b"abcdef".as_ref()), (b"".as_ref(), false));
    }

    #[test]
    fn frame_start_incomplete_imm() {
        assert_eq!(frame_start(b"\xab".as_ref()), (b"\xab".as_ref(), false));
    }

    #[test]
    fn frame_start_complete_imm() {
        assert_eq!(
            frame_start(b"\xab\xcd".as_ref()),
            (b"\xab\xcd".as_ref(), true)
        );
    }

    #[test]
    fn frame_start_incomplete() {
        assert_eq!(frame_start(b"abc\xab".as_ref()), (b"\xab".as_ref(), false));
    }

    #[test]
    fn frame_start_complete() {
        assert_eq!(
            frame_start(b"abc\xab\xcd".as_ref()),
            (b"\xab\xcd".as_ref(), true)
        );
    }

    #[test]
    fn frame_start_incomplete_2() {
        assert_eq!(
            frame_start(b"abc\xabdef\xab".as_ref()),
            (b"\xab".as_ref(), false)
        );
    }

    #[test]
    fn frame_start_complete_2() {
        assert_eq!(
            frame_start(b"abc\xabdef\xab\xcd".as_ref()),
            (b"\xab\xcd".as_ref(), true)
        );
    }

    #[test]
    fn frame_raw_empty() {
        assert_eq!(frame_raw(b"".as_ref()), (b"".as_ref(), None))
    }

    #[test]
    fn frame_raw_discard_garbage() {
        assert_eq!(frame_raw(b"abcdef".as_ref()), (b"".as_ref(), None));
    }

    #[test]
    fn frame_raw_incomplete_prefix_imm() {
        assert_eq!(frame_raw(b"\xab".as_ref()), (b"\xab".as_ref(), None));
    }

    #[test]
    fn frame_raw_incomplete_imm() {
        assert_eq!(
            frame_raw(b"\xab\xcd\x01\x00foo".as_ref()),
            (b"\xab\xcd\x01\x00foo".as_ref(), None)
        );
    }

    #[test]
    fn frame_raw_complete_imm() {
        assert_eq!(
            frame_raw(b"\xab\xcd\x01\x00foo\xdc\xbaafter".as_ref()),
            (b"after".as_ref(), Some(b"foo".as_ref()))
        );
    }

    #[test]
    fn frame_raw_incomplete_prefix() {
        assert_eq!(frame_raw(b"abc\xab".as_ref()), (b"\xab".as_ref(), None));
    }

    #[test]
    fn frame_raw_incomplete() {
        assert_eq!(
            frame_raw(b"abc\xab\xcd\x01\x00foo".as_ref()),
            (b"\xab\xcd\x01\x00foo".as_ref(), None)
        );
    }

    #[test]
    fn frame_raw_complete() {
        assert_eq!(
            frame_raw(b"abc\xab\xcd\x01\x00foo\xdc\xbaafter".as_ref()),
            (b"after".as_ref(), Some(b"foo".as_ref()))
        );
    }

    #[test]
    fn frame_raw_incomplete_prefix_2() {
        assert_eq!(
            frame_raw(b"abc\xabdef\xab".as_ref()),
            (b"\xab".as_ref(), None)
        );
    }

    #[test]
    fn frame_raw_incomplete_2() {
        assert_eq!(
            frame_raw(b"abc\xabdef\xab\xcd\x01\x00foo".as_ref()),
            (b"\xab\xcd\x01\x00foo".as_ref(), None)
        );
    }

    #[test]
    fn frame_raw_complete_2() {
        assert_eq!(
            frame_raw(b"abc\xabdef\xab\xcd\x01\x00foo\xdc\xbaafter".as_ref()),
            (b"after".as_ref(), Some(b"foo".as_ref()))
        );
    }

    fn apply_obfuscate<'a>(
        result: (&'a [u8], ParseResult<&'a [u8], Deobfuscated<&'a [u8]>>),
    ) -> (&'a [u8], ParseResult<&'a [u8], Vec<u8>>) {
        (result.0, result.1.map(|deob| deob.to_vec()))
    }

    #[test]
    fn framed_empty() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(foo(b"".as_ref()), (b"".as_ref(), ParseResult::None))
    }

    #[test]
    fn framed_discard_garbage() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(foo(b"abcdef".as_ref()), (b"".as_ref(), ParseResult::None));
    }

    #[test]
    fn framed_incomplete_prefix_imm() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(foo(b"\xab".as_ref()), (b"\xab".as_ref(), ParseResult::None));
    }

    #[test]
    fn framed_incomplete_imm() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            foo(b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref()),
            (
                b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref(),
                ParseResult::None
            )
        );
    }

    #[test]
    fn framed_complete_imm() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            apply_obfuscate(foo(
                b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4\xdc\xbaafter".as_ref()
            )),
            (b"after".as_ref(), ParseResult::Ok(b"foo".to_vec()))
        );
    }

    #[test]
    fn framed_incomplete_prefix() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            foo(b"abc\xab".as_ref()),
            (b"\xab".as_ref(), ParseResult::None)
        );
    }

    #[test]
    fn framed_incomplete() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            foo(b"abc\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref()),
            (
                b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref(),
                ParseResult::None
            )
        );
    }

    #[test]
    fn framed_complete() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            apply_obfuscate(foo(
                b"abc\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4\xdc\xbaafter".as_ref()
            )),
            (b"after".as_ref(), ParseResult::Ok(b"foo".to_vec()))
        );
    }

    #[test]
    fn framed_incomplete_prefix_2() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            foo(b"abc\xabdef\xab".as_ref()),
            (b"\xab".as_ref(), ParseResult::None)
        );
    }

    #[test]
    fn framed_incomplete_2() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            foo(b"abc\xabdef\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref()),
            (
                b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref(),
                ParseResult::None
            )
        );
    }

    #[test]
    fn framed_complete_2() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            apply_obfuscate(foo(
                b"abc\xabdef\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4\xdc\xbaafter".as_ref()
            )),
            (b"after".as_ref(), ParseResult::Ok(b"foo".to_vec()))
        );
    }
}
