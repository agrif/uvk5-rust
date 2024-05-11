use nom::error::Error;
use nom::IResult;
use std::ops::Range;

use super::crc::{CrcDigest, CrcStyle};
use super::obfuscation::Key;
use super::{FRAME_END, FRAME_START, MAX_FRAME_SIZE};

/// A helpful short name for a whole bundle of useful parser traits,
/// plus iterating over slice chunks.
pub trait Parse:
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
    /// Iterate over chunks of byte slices.
    ///
    /// Used to speed up CRC digests and round-trip writes.
    fn iter_slices(&self) -> impl Iterator<Item = &[u8]>;
}

impl<'a> Parse for &'a [u8] {
    fn iter_slices(&self) -> impl Iterator<Item = &[u8]> {
        std::iter::once(*self)
    }
}

/// A trait for something we can deobfuscated and extract frames from.
pub trait ParseMut: Sized {
    /// A non-mutable slice of input, suitable for nom.
    type Input: Parse;

    /// Get the length of this input.
    fn len(&self) -> usize;

    /// Iterate over the bytes in this input.
    fn iter(&self) -> impl Iterator<Item = u8>;

    /// Iterate mutably over the bytes in this input.
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut u8>;

    /// Slice this input.
    fn slice(self, range: Range<usize>) -> Self;

    /// Drop mutability for use with nom.
    fn to_input(self) -> Self::Input;

    /// Deobfuscate the contents of this slice.
    fn deobfuscate(&mut self, key: &mut Key) {
        for b in self.iter_mut() {
            *b = key.apply(*b);
        }
    }
}

impl<'a> ParseMut for &'a mut [u8] {
    type Input = &'a [u8];

    fn len(&self) -> usize {
        <[u8]>::len(self)
    }

    fn iter(&self) -> impl Iterator<Item = u8> {
        <[u8]>::iter(self).copied()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut u8> {
        <[u8]>::iter_mut(self)
    }

    fn slice(self, range: Range<usize>) -> Self {
        &mut self[range]
    }

    fn to_input(self) -> Self::Input {
        self
    }
}

/// A helper to match a sequence of bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Matcher<'a> {
    needle: &'a [u8],
    start: Option<usize>,
    pos: usize,
}

/// Result of match_().
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MatchResult {
    /// Match successful, with the matched range.
    Matched(Range<usize>),
    /// Match not successful.
    NotMatched,
    /// Match incomplete, with index of where match might start.
    Incomplete(usize),
}

impl<'a> Matcher<'a> {
    fn new(needle: &'a [u8]) -> Self {
        Self {
            needle,
            start: None,
            pos: 0,
        }
    }

    fn test(&mut self, i: usize, b: u8) -> Option<Range<usize>> {
        if b == self.needle[self.pos] {
            if self.pos == 0 {
                self.start = Some(i);
            }
            self.pos += 1;
            if self.pos == self.needle.len() {
                return self.start.map(|s| s..s + self.needle.len());
            }
        } else {
            self.start = None;
            self.pos = 0;
        }

        None
    }

    // search the iterator for our needle.
    // not matched means it was not found anywhere inside.
    // incomplete means we found a partial needle at the end
    fn search(&mut self, iter: &mut impl Iterator<Item = (usize, u8)>) -> MatchResult {
        for (i, b) in iter {
            if let Some(range) = self.test(i, b) {
                return MatchResult::Matched(range);
            }
        }

        if let Some(start) = self.start {
            MatchResult::Incomplete(start)
        } else {
            MatchResult::NotMatched
        }
    }

    // match right at the start of the iterator.
    // not matched means it's definitely not here
    // incomplete means it might be here with more data
    fn match_(
        &mut self,
        start_i: usize,
        iter: &mut impl Iterator<Item = (usize, u8)>,
    ) -> MatchResult {
        for (i, b) in iter {
            if let Some(range) = self.test(i, b) {
                return MatchResult::Matched(range);
            }

            if self.start.is_none() {
                return MatchResult::NotMatched;
            }
        }

        MatchResult::Incomplete(self.start.unwrap_or(start_i))
    }
}

/// Helper to grab a le u16 out of an enumerated byte iterator.
fn read_le_u16(iter: &mut impl Iterator<Item = (usize, u8)>) -> Option<u16> {
    Some((iter.next()?.1 as u16) | ((iter.next()?.1 as u16) << 8))
}

/// Find a frame, and return the deobfuscated body and CRC.
///
/// Returns the number of consumed bytes and None if it only skipped
/// data and found no complete frames.
///
/// If a frame is found, return the range for the full frame, and a
/// slice of the deobfuscated body and CRC.
pub fn frame_raw<I>(input: I) -> (usize, Option<(Range<usize>, I::Input)>)
where
    I: ParseMut,
{
    let mut bytes = input.iter().enumerate();

    // loop until we stop advancing or find a frame
    loop {
        // search for the FRAME_START
        let start = match Matcher::new(&FRAME_START).search(&mut bytes) {
            MatchResult::Matched(range) => range,
            MatchResult::NotMatched => {
                // there is no FRAME_START, anywhere
                return (input.len(), None);
            }
            MatchResult::Incomplete(i) => {
                // there might be a FRAME_START here later
                return (i, None);
            }
        };

        // now there is a little-endian u16 length
        let Some(length) = read_le_u16(&mut bytes) else {
            // not enough data yet, consume up to FRAME_START
            return (start.start, None);
        };

        // make sure our length makes sense
        // FRAME_START + u16 len + body + u16 crc + FRAME_END
        if length as usize > MAX_FRAME_SIZE - FRAME_START.len() - FRAME_END.len() - 2 - 2 {
            // this is too big, so this is a false frame
            // Skip 1 past FRAME_START and try again.
            bytes = input.iter().enumerate();
            bytes.nth(start.start + 1);
            continue;
        }

        // keep track of where we are now
        let body_start = start.end + 2;

        // now there is length bytes, then a 2-byte crc (which we skip for now)
        if bytes.nth(length as usize).is_none() || bytes.next().is_none() {
            // not enough data yet, consume up to FRAME_START
            return (start.start, None);
        }

        // ok, where are we now
        let crc_end = body_start + length as usize + 2;

        // search for FRAME_END
        let end = match Matcher::new(&FRAME_END).match_(crc_end, &mut bytes) {
            MatchResult::Matched(range) => range,
            MatchResult::NotMatched => {
                // FRAME_END should be here but is not. This is a false start.
                // Skip 1 past FRAME_START and try again.
                bytes = input.iter().enumerate();
                bytes.nth(start.start + 1);
                continue;
            }
            MatchResult::Incomplete(_) => {
                // not enough data yet, consume up to FRAME_START
                return (start.start, None);
            }
        };

        // it would be neat to be able to do the CRC and parse, and then
        // if that fails, skip 1 past FRAME_START like above.
        // however, these steps need deobfuscated data
        // and lifetimes make it hard to re-obfuscate on failure

        // so, best effort:
        // there are about 6 bytes all with exactly the values they need.
        // this looks like a frame. it's a frame.

        // we have a frame from start.start to end.end
        // the body + crc is inside body_start to crc_end
        drop(bytes);
        let mut frame_body = input.slice(body_start..crc_end);
        frame_body.deobfuscate(&mut Key::new());
        return (end.end, Some((start.start..end.end, frame_body.to_input())));
    }
}

/// A possible result from frame().
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseResult<I, O, E = Error<I>> {
    /// Frame parse result, alongside range where whole frame was located.
    Ok(Range<usize>, O),
    /// Range for full frame, original frame body without CRC, Error.
    ParseErr(Range<usize>, I, E),
    /// CRC check failed, with range for full frame and whole original
    /// frame body including CRC.
    CrcErr(Range<usize>, I),
    /// Only non-frame input was consumed.
    None,
}

impl<I, O, E> ParseResult<I, O, E> {
    pub fn ok(self) -> Option<O> {
        match self {
            Self::Ok(_, o) => Some(o),
            Self::ParseErr(_, _, _) => None,
            Self::CrcErr(_, _) => None,
            Self::None => None,
        }
    }

    pub fn range(&self) -> Option<&Range<usize>> {
        match self {
            Self::Ok(r, _) => Some(r),
            Self::ParseErr(r, _, _) => Some(r),
            Self::CrcErr(r, _) => Some(r),
            Self::None => None,
        }
    }

    pub fn map<F, Op>(self, f: F) -> ParseResult<I, Op, E>
    where
        F: FnOnce(O) -> Op,
    {
        match self {
            Self::Ok(r, o) => ParseResult::Ok(r, f(o)),
            Self::ParseErr(r, frame, err) => ParseResult::ParseErr(r, frame, err),
            Self::CrcErr(r, frame) => ParseResult::CrcErr(r, frame),
            Self::None => ParseResult::None,
        }
    }

    pub fn map_err<F, Ep>(self, f: F) -> ParseResult<I, O, Ep>
    where
        F: FnOnce(E) -> Ep,
    {
        match self {
            Self::Ok(r, o) => ParseResult::Ok(r, o),
            Self::ParseErr(r, frame, err) => ParseResult::ParseErr(r, frame, f(err)),
            Self::CrcErr(r, frame) => ParseResult::CrcErr(r, frame),
            Self::None => ParseResult::None,
        }
    }
}

/// Check the LE u16 CRC at the end of a frame body.
///
/// Return the body (without CRC) on success.
pub fn check_crc<C, I>(crc: C, input: I) -> Option<I>
where
    C: CrcStyle,
    I: Parse,
{
    if input.input_len() < 2 {
        return None;
    }

    let (suffix, prefix) = input.take_split(input.input_len() - 2);
    let mut digest = crc.digest();
    for chunk in prefix.iter_slices() {
        digest.update(chunk);
    }

    let calculated = digest.finalize();

    if let Some(provided) = read_le_u16(&mut suffix.iter_indices()) {
        if crc.validate(calculated, provided) {
            Some(input.slice(0..input.input_len() - 2))
        } else {
            None
        }
    } else {
        None
    }
}

/// Find a frame, and deobfuscate and then parse with the provided parser.
///
/// The parser is always run against an entire frame.
///
/// Returns number of consumed bytes and Ok(..) on successful parse,
/// ParseErr(..) if the provided parser failed, CrcErr(..) if the
/// checksum was wrong, and None if it only skipped data and found no
/// complete frames.  In all cases, any frames passed to the parser
/// are removed from the input.
pub fn frame<C, I, P, O>(crc: C, input: I, parser: P) -> (usize, ParseResult<I::Input, O>)
where
    C: CrcStyle,
    P: nom::Parser<I::Input, O, Error<I::Input>>,
    I: ParseMut,
{
    let mut parser_all = nom::combinator::all_consuming(parser);

    let (consumed, maybe_content) = frame_raw(input);
    match maybe_content {
        Some((r, content)) => {
            // found a frame, wrap it and feed it to our parser
            if let Some(body) = check_crc(&crc, content.clone()) {
                match parser_all(body.clone()) {
                    Ok((_, result)) => (consumed, ParseResult::Ok(r, result)),
                    Err(e) => match e {
                        nom::Err::Incomplete(_) => (
                            consumed,
                            ParseResult::ParseErr(
                                r,
                                body.clone(),
                                Error {
                                    input: body,
                                    code: nom::error::ErrorKind::Complete,
                                },
                            ),
                        ),
                        nom::Err::Error(e) => (consumed, ParseResult::ParseErr(r, body, e)),
                        nom::Err::Failure(e) => (consumed, ParseResult::ParseErr(r, body, e)),
                    },
                }
            } else {
                (consumed, ParseResult::CrcErr(r, content))
            }
        }
        None => {
            // no frame found, only ate input
            (consumed, ParseResult::None)
        }
    }
}

/// Parse a message type and length, and provide it to a function that
/// returns a parser for that message type's body.
///
/// This wraps a function that returns nom parsers for a given message
/// type body into a nom parser for a whole message of any of those
/// types.
pub fn message<I, F, P, O>(mut parser: F) -> impl FnMut(I) -> IResult<I, O>
where
    F: FnMut(u16) -> P,
    P: nom::Parser<I, O, Error<I>>,
    I: Parse,
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
pub trait MessageParse<I>: Sized
where
    I: Parse,
{
    /// Parse the body of a message, given the message type.
    fn parse_body(typ: u16) -> impl nom::Parser<I, Self, Error<I>>;

    /// Parse an entire message, including type and length header.
    fn parse_frame_body() -> impl nom::Parser<I, Self, Error<I>> {
        message(Self::parse_body)
    }

    /// Parse an entire frame containing a message, skipping data
    /// before the frame. If the frame doesn't parse as this message,
    /// or the CRC fails, it will still consume the frame from the
    /// input.
    ///
    /// Returns the number of consumed bytes and the parse or CRC result.
    ///
    /// This parses and handles frame start/end, length, obfuscation, and CRC.
    ///
    fn parse_frame<C, IM>(crc: &C, input: IM) -> (usize, ParseResult<I, Self>)
    where
        C: CrcStyle,
        IM: ParseMut<Input = I>,
    {
        frame(crc, input, Self::parse_frame_body())
    }
}

#[cfg(test)]
mod test {
    use super::super::crc::CrcConstant;
    use super::*;

    #[test]
    fn frame_raw_empty() {
        let mut frame = b"".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (0, None))
    }

    #[test]
    fn frame_raw_discard_garbage() {
        let mut frame = b"abcdef".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (6, None));
    }

    #[test]
    fn frame_raw_incomplete_prefix_imm() {
        let mut frame = b"\xab".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (0, None));
    }

    #[test]
    fn frame_raw_incomplete_imm() {
        let mut frame = b"\xab\xcd\x01\x00\x70\x03\x7b".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (0, None));
    }

    #[test]
    fn frame_raw_complete_imm() {
        let mut frame = b"\xab\xcd\x01\x00\x70\x03\x7b\xdc\xbaafter".to_owned();
        assert_eq!(
            frame_raw(frame.as_mut()),
            (9, Some((0..9, b"foo".as_ref())))
        );
    }

    #[test]
    fn frame_raw_incomplete_prefix() {
        let mut frame = b"abc\xab".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (3, None));
    }

    #[test]
    fn frame_raw_incomplete() {
        let mut frame = b"abc\xab\xcd\x01\x00\x70\x03\x7b".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (3, None));
    }

    #[test]
    fn frame_raw_complete() {
        let mut frame = b"abc\xab\xcd\x01\x00\x70\x03\x7b\xdc\xbaafter".to_owned();
        assert_eq!(
            frame_raw(frame.as_mut()),
            (12, Some((3..12, b"foo".as_ref())))
        );
    }

    #[test]
    fn frame_raw_incomplete_prefix_2() {
        let mut frame = b"abc\xabdef\xab".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (7, None));
    }

    #[test]
    fn frame_raw_incomplete_2() {
        let mut frame = b"abc\xabdef\xab\xcd\x01\x00\x70\x03\x7b".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (7, None));
    }

    #[test]
    fn frame_raw_complete_2() {
        let mut frame = b"abc\xabdef\xab\xcd\x01\x00\x70\x03\x7b\xdc\xbaafter".to_owned();

        assert_eq!(
            frame_raw(frame.as_mut()),
            (16, Some((7..16, b"foo".as_ref())))
        );
    }

    #[test]
    fn frame_raw_bad_length() {
        let mut frame = b"abc\xab\xcd\x00\x02foo".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (10, None));
    }

    #[test]
    fn frame_raw_bad_end() {
        let mut frame = b"abc\xab\xcd\x01\x00\x70\x03\x7b\xdc\xbbafter".to_owned();
        assert_eq!(frame_raw(frame.as_mut()), (17, None));
    }

    #[test]
    fn frame_empty() {
        let mut data = b"".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (0, ParseResult::None))
    }

    #[test]
    fn frame_discard_garbage() {
        let mut data = b"abcdef".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (6, ParseResult::None))
    }

    #[test]
    fn frame_incomplete_prefix_imm() {
        let mut data = b"".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"\xab".as_ref()),
        );
        assert_eq!(res, (0, ParseResult::None))
    }

    #[test]
    fn frame_incomplete_imm() {
        let mut data = b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (0, ParseResult::None))
    }

    #[test]
    fn frame_complete_imm() {
        let mut data = b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4\xdc\xbaafter".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (11, ParseResult::Ok(0..11, b"foo".as_ref())))
    }

    #[test]
    fn frame_incomplete_prefix() {
        let mut data = b"abc\xab".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (3, ParseResult::None))
    }

    #[test]
    fn frame_incomplete() {
        let mut data = b"abc\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (3, ParseResult::None))
    }

    #[test]
    fn frame_complete() {
        let mut data = b"abc\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4\xdc\xbaafter".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (14, ParseResult::Ok(3..14, b"foo".as_ref())))
    }

    #[test]
    fn frame_incomplete_prefix_2() {
        let mut data = b"abc\xabdef\xab".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (7, ParseResult::None))
    }

    #[test]
    fn frame_incomplete_2() {
        let mut data = b"abc\xabdef\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (7, ParseResult::None))
    }

    #[test]
    fn frame_complete_2() {
        let mut data = b"abc\xabdef\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4\xdc\xbaafter".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(res, (18, ParseResult::Ok(7..18, b"foo".as_ref())))
    }

    #[test]
    fn frame_crc_error() {
        let mut data = b"abc\xab\xcd\x03\x00\x70\x03\x7b\x18\xee\xdc\xbaafter".to_owned();
        let res = frame(
            CrcConstant(0xcafe),
            data.as_mut(),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            res,
            (14, ParseResult::CrcErr(3..14, b"foo\xfe\xc0".as_ref()))
        )
    }
}
