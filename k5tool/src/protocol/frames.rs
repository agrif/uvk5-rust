use nom::error::Error;
use nom::IResult;

use super::CrcStyle;

pub const FRAME_START: [u8; 2] = [0xab, 0xcd];
pub const FRAME_END: [u8; 2] = [0xdc, 0xba];

/// Total guess, here.
pub const MAX_FRAME_SIZE: usize = 0x200;

pub const OBFUSCATION: [u8; 16] = [
    0x16, 0x6c, 0x14, 0xe6, 0x2e, 0x91, 0x0d, 0x40, 0x21, 0x35, 0xd5, 0x40, 0x13, 0x03, 0xe9, 0x80,
];

/// Eats input until it sees a frame start, then leaves it intact.
///
/// Returns true when it found a frame start, false otherwise.
pub fn frame_start<I>(input: I) -> IResult<I, bool>
where
    I: nom::InputTakeAtPosition<Item = u8>
        + nom::Compare<&'static [u8]>
        + nom::InputLength
        + nom::InputTake
        + Clone,
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
                    return Ok((pre_frame_rest, true));
                }
                Err(nom::Err::Incomplete(_)) => {
                    // we didn't see the rest, but we might later, return false
                    return Ok((pre_frame_rest, false));
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
            return Ok((rest, false));
        }
    }
}

/// Find a frame, and return the contents of that frame (still
/// obfuscated and CRC'd).
///
/// Returns None if it only skipped data and found no complete frames.
pub fn frame_raw<I>(input: I) -> IResult<I, Option<I>>
where
    I: nom::InputTakeAtPosition<Item = u8>
        + nom::Compare<&'static [u8]>
        + nom::InputLength
        + nom::InputTake
        + nom::InputIter<Item = u8>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + Clone,
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
        let (rest, frame_found) = frame_start(loop_input)?;
        if frame_found {
            // we found a frame, try the whole message
            match delimited(rest.clone()) {
                Ok((rest, result)) => {
                    // hey, it worked!
                    return Ok((rest, Some(result)));
                }
                Err(nom::Err::Incomplete(_)) => {
                    // it didn't work now, but it might later
                    return Ok((rest, None));
                }
                Err(_) => {
                    // it didn't work and won't, ever
                    // skip the frame start, so we don't get stuck here again
                    // use complete because we know it's there
                    let (rest, _) = nom::bytes::complete::take(FRAME_START.len())(rest)?;
                    // try again
                    loop_input = rest;
                }
            }
        } else {
            // no frame found, only skipped
            return Ok((rest, None));
        }
    }
}

/// Infinite deobfuscation key iterator.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Key {
    index: usize,
}

impl Key {
    fn new() -> Self {
        Self { index: 0 }
    }

    fn next(&mut self) -> u8 {
        let v = OBFUSCATION[self.index];
        self.index += 1;
        if self.index >= OBFUSCATION.len() {
            self.index = 0;
        }
        v
    }

    fn advance(&self, num: usize) -> Self {
        let index = (self.index + num) % OBFUSCATION.len();
        Self { index }
    }
}

/// Wraps a nom input to deobfuscate it on-the-fly.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Deobfuscated<I> {
    inner: I,
    key: Key,
}

impl<I> Deobfuscated<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            key: Key::new(),
        }
    }
}

impl<I> Deobfuscated<I>
where
    Self: nom::InputIter<Item = u8>,
{
    pub fn iter(&self) -> <Self as nom::InputIter>::IterElem {
        use nom::InputIter;
        self.iter_elements()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.iter().collect()
    }
}

impl<I> Deobfuscated<I>
where
    Self: nom::InputIter<Item = u8>
        + nom::InputLength
        + nom::InputTake
        + nom::Slice<std::ops::RangeFrom<usize>>,
{
    /// Returns None if CRC fails, returns body slice if it succeeds.
    pub fn check_crc<C>(&self, crc: C) -> Option<Self>
    where
        C: CrcStyle,
    {
        use nom::{InputLength, InputTake};
        let len = self.input_len();
        if len < 2 {
            None
        } else {
            let (suffix, prefix) = self.take_split(len - 2);
            let calculated = crc.calculate(prefix.iter());
            let (_, provided) = nom::number::complete::le_u16::<Self, Error<Self>>(suffix).ok()?;
            if crc.validate(calculated, provided) {
                Some(prefix)
            } else {
                None
            }
        }
    }
}

// ok now just implement as many of these as I can

// may be possible if I use nom::InputIter
// nom::FindSubstring and nom::FindToken
// ParseTo is probably a lost cause, since [u8] round-trips through str

/// Iterator over indices and elements. See nom::InputIter.
pub struct DeobfuscatedIter<I: nom::InputIter<Item = u8>> {
    inner: I::Iter,
    key: Key,
}

impl<I> Iterator for DeobfuscatedIter<I>
where
    I: nom::InputIter<Item = u8>,
{
    type Item = (usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(i, v)| (i, v ^ self.key.next()))
    }
}

/// Iterator over elements. See nom::InputIter.
pub struct DeobfuscatedIterElem<I: nom::InputIter<Item = u8>> {
    inner: I::IterElem,
    key: Key,
}

impl<I> Iterator for DeobfuscatedIterElem<I>
where
    I: nom::InputIter<Item = u8>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|v| v ^ self.key.next())
    }
}

impl<I> nom::InputIter for Deobfuscated<I>
where
    I: nom::InputIter<Item = u8>,
{
    type Item = u8;
    type Iter = DeobfuscatedIter<I>;
    type IterElem = DeobfuscatedIterElem<I>;

    fn iter_indices(&self) -> Self::Iter {
        DeobfuscatedIter {
            inner: self.inner.iter_indices(),
            key: self.key.clone(),
        }
    }

    fn iter_elements(&self) -> Self::IterElem {
        DeobfuscatedIterElem {
            inner: self.inner.iter_elements(),
            key: self.key.clone(),
        }
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        let mut key = self.key.clone();
        for (i, v) in self.iter_indices() {
            if predicate(v ^ key.next()) {
                return Some(i);
            }
        }
        None
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.inner.slice_index(count)
    }
}

impl<I> nom::InputLength for Deobfuscated<I>
where
    I: nom::InputLength,
{
    fn input_len(&self) -> usize {
        self.inner.input_len()
    }
}

impl<I> nom::InputTake for Deobfuscated<I>
where
    I: nom::InputTake,
{
    fn take(&self, count: usize) -> Self {
        Self {
            inner: self.inner.take(count),
            key: self.key.advance(count),
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (suffix, prefix) = self.inner.take_split(count);
        (
            Self {
                inner: suffix,
                key: self.key.advance(count),
            },
            Self {
                inner: prefix,
                key: self.key.clone(),
            },
        )
    }
}

// this gives us TakeInputAtPosition and Compare
impl<I> nom::UnspecializedInput for Deobfuscated<I> {}

impl<I> nom::Offset for Deobfuscated<I>
where
    I: nom::Offset,
{
    fn offset(&self, second: &Self) -> usize {
        self.inner.offset(&second.inner)
    }
}

impl<I, R> nom::Slice<R> for Deobfuscated<I>
where
    I: nom::Slice<R>,
    R: std::ops::RangeBounds<usize>,
{
    fn slice(&self, range: R) -> Self {
        let start_idx = match range.start_bound() {
            std::ops::Bound::Included(i) => *i,
            std::ops::Bound::Excluded(i) => *i + 1, // can this happen??
            std::ops::Bound::Unbounded => 0,
        };
        Self {
            inner: self.inner.slice(range),
            key: self.key.advance(start_idx),
        }
    }
}

/// A possible result from framed().
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FramedResult<I, O, E = Error<Deobfuscated<I>>> {
    /// Frame parse result
    Ok(O),
    /// (Original frame without CRC, Error)
    ParseErr(Deobfuscated<I>, E),
    /// CRC check failed, with whole original frame including CRC.
    CrcErr(Deobfuscated<I>),
    /// Only non-frame input was consumed.
    None,
}

impl<I, O, E> FramedResult<I, O, E> {
    pub fn ignore_error(self) -> Option<O> {
        match self {
            Self::Ok(o) => Some(o),
            Self::ParseErr(_, _) => None,
            Self::CrcErr(_) => None,
            Self::None => None,
        }
    }

    pub fn map<F, Op>(self, f: F) -> FramedResult<I, Op, E>
    where
        F: FnOnce(O) -> Op,
    {
        match self {
            Self::Ok(o) => FramedResult::Ok(f(o)),
            Self::ParseErr(frame, err) => FramedResult::ParseErr(frame, err),
            Self::CrcErr(frame) => FramedResult::CrcErr(frame),
            Self::None => FramedResult::None,
        }
    }

    pub fn map_parse_err<F, Ep>(self, f: F) -> FramedResult<I, O, Ep>
    where
        F: FnOnce(E) -> Ep,
    {
        match self {
            Self::Ok(o) => FramedResult::Ok(o),
            Self::ParseErr(frame, err) => FramedResult::ParseErr(frame, f(err)),
            Self::CrcErr(frame) => FramedResult::CrcErr(frame),
            Self::None => FramedResult::None,
        }
    }
}

/// Find a frame, and deobfuscate and then parse with the provided parser.
///
/// The parser is always run against an entire frame.
///
/// Returns Some(Ok(..)) on successful parse, and Some(Err(..)) on
/// failure. In both cases, any frames passed to the parser are
/// removed from the input.
///
/// Returns None if it only skipped data and found no complete frames.
pub fn framed<C, I, P, O>(crc: C, parser: P) -> impl FnMut(I) -> IResult<I, FramedResult<I, O>>
where
    C: CrcStyle,
    P: nom::Parser<Deobfuscated<I>, O, Error<Deobfuscated<I>>>,
    I: nom::InputTakeAtPosition<Item = u8>
        + nom::Compare<&'static [u8]>
        + nom::InputLength
        + nom::InputTake
        + nom::InputIter<Item = u8>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + Clone,
{
    // FIXME crc??

    let mut parser_all = nom::combinator::all_consuming(parser);
    move |input| {
        let (rest, maybe_content) = frame_raw(input)?;
        match maybe_content {
            Some(content) => {
                // found a frame, wrap it and feed it to our parser
                let deobfuscated = Deobfuscated::new(content);
                if let Some(body) = deobfuscated.check_crc(&crc) {
                    match parser_all(body.clone()) {
                        Ok((_, result)) => Ok((rest, FramedResult::Ok(result))),
                        Err(e) => match e {
                            nom::Err::Incomplete(_) => Ok((
                                rest,
                                FramedResult::ParseErr(
                                    body.clone(),
                                    Error {
                                        input: body,
                                        code: nom::error::ErrorKind::Complete,
                                    },
                                ),
                            )),
                            nom::Err::Error(e) => Ok((rest, FramedResult::ParseErr(body, e))),
                            nom::Err::Failure(e) => Ok((rest, FramedResult::ParseErr(body, e))),
                        },
                    }
                } else {
                    Ok((rest, FramedResult::CrcErr(deobfuscated)))
                }
            }
            None => {
                // no frame found, only ate input
                Ok((rest, FramedResult::None))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::CrcConstant;
    use super::*;

    #[test]
    fn frame_start_empty() {
        assert_eq!(frame_start(b"".as_ref()), Ok((b"".as_ref(), false)));
    }

    #[test]
    fn frame_start_discard_garbage() {
        assert_eq!(frame_start(b"abcdef".as_ref()), Ok((b"".as_ref(), false)));
    }

    #[test]
    fn frame_start_incomplete_imm() {
        assert_eq!(frame_start(b"\xab".as_ref()), Ok((b"\xab".as_ref(), false)));
    }

    #[test]
    fn frame_start_complete_imm() {
        assert_eq!(
            frame_start(b"\xab\xcd".as_ref()),
            Ok((b"\xab\xcd".as_ref(), true))
        );
    }

    #[test]
    fn frame_start_incomplete() {
        assert_eq!(
            frame_start(b"abc\xab".as_ref()),
            Ok((b"\xab".as_ref(), false))
        );
    }

    #[test]
    fn frame_start_complete() {
        assert_eq!(
            frame_start(b"abc\xab\xcd".as_ref()),
            Ok((b"\xab\xcd".as_ref(), true))
        );
    }

    #[test]
    fn frame_start_incomplete_2() {
        assert_eq!(
            frame_start(b"abc\xabdef\xab".as_ref()),
            Ok((b"\xab".as_ref(), false))
        );
    }

    #[test]
    fn frame_start_complete_2() {
        assert_eq!(
            frame_start(b"abc\xabdef\xab\xcd".as_ref()),
            Ok((b"\xab\xcd".as_ref(), true))
        );
    }

    #[test]
    fn frame_raw_empty() {
        assert_eq!(frame_raw(b"".as_ref()), Ok((b"".as_ref(), None)))
    }

    #[test]
    fn frame_raw_discard_garbage() {
        assert_eq!(frame_raw(b"abcdef".as_ref()), Ok((b"".as_ref(), None)));
    }

    #[test]
    fn frame_raw_incomplete_prefix_imm() {
        assert_eq!(frame_raw(b"\xab".as_ref()), Ok((b"\xab".as_ref(), None)));
    }

    #[test]
    fn frame_raw_incomplete_imm() {
        assert_eq!(
            frame_raw(b"\xab\xcd\x01\x00foo".as_ref()),
            Ok((b"\xab\xcd\x01\x00foo".as_ref(), None))
        );
    }

    #[test]
    fn frame_raw_complete_imm() {
        assert_eq!(
            frame_raw(b"\xab\xcd\x01\x00foo\xdc\xbaafter".as_ref()),
            Ok((b"after".as_ref(), Some(b"foo".as_ref())))
        );
    }

    #[test]
    fn frame_raw_incomplete_prefix() {
        assert_eq!(frame_raw(b"abc\xab".as_ref()), Ok((b"\xab".as_ref(), None)));
    }

    #[test]
    fn frame_raw_incomplete() {
        assert_eq!(
            frame_raw(b"abc\xab\xcd\x01\x00foo".as_ref()),
            Ok((b"\xab\xcd\x01\x00foo".as_ref(), None))
        );
    }

    #[test]
    fn frame_raw_complete() {
        assert_eq!(
            frame_raw(b"abc\xab\xcd\x01\x00foo\xdc\xbaafter".as_ref()),
            Ok((b"after".as_ref(), Some(b"foo".as_ref())))
        );
    }

    #[test]
    fn frame_raw_incomplete_prefix_2() {
        assert_eq!(
            frame_raw(b"abc\xabdef\xab".as_ref()),
            Ok((b"\xab".as_ref(), None))
        );
    }

    #[test]
    fn frame_raw_incomplete_2() {
        assert_eq!(
            frame_raw(b"abc\xabdef\xab\xcd\x01\x00foo".as_ref()),
            Ok((b"\xab\xcd\x01\x00foo".as_ref(), None))
        );
    }

    #[test]
    fn frame_raw_complete_2() {
        assert_eq!(
            frame_raw(b"abc\xabdef\xab\xcd\x01\x00foo\xdc\xbaafter".as_ref()),
            Ok((b"after".as_ref(), Some(b"foo".as_ref())))
        );
    }

    fn apply_obfuscate<'a>(
        result: IResult<&'a [u8], FramedResult<&'a [u8], Deobfuscated<&'a [u8]>>>,
    ) -> IResult<&'a [u8], FramedResult<&'a [u8], Vec<u8>>> {
        result.map(|(r, opt)| (r, opt.map(|deob| deob.to_vec())))
    }

    #[test]
    fn framed_empty() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(foo(b"".as_ref()), Ok((b"".as_ref(), FramedResult::None)))
    }

    #[test]
    fn framed_discard_garbage() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            foo(b"abcdef".as_ref()),
            Ok((b"".as_ref(), FramedResult::None))
        );
    }

    #[test]
    fn framed_incomplete_prefix_imm() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            foo(b"\xab".as_ref()),
            Ok((b"\xab".as_ref(), FramedResult::None))
        );
    }

    #[test]
    fn framed_incomplete_imm() {
        let mut foo = framed(
            CrcConstant(0xcafe),
            nom::bytes::complete::tag(b"foo".as_ref()),
        );
        assert_eq!(
            foo(b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref()),
            Ok((
                b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref(),
                FramedResult::None
            ))
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
            Ok((b"after".as_ref(), FramedResult::Ok(b"foo".to_vec())))
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
            Ok((b"\xab".as_ref(), FramedResult::None))
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
            Ok((
                b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref(),
                FramedResult::None
            ))
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
            Ok((b"after".as_ref(), FramedResult::Ok(b"foo".to_vec())))
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
            Ok((b"\xab".as_ref(), FramedResult::None))
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
            Ok((
                b"\xab\xcd\x03\x00\x70\x03\x7b\x18\xe4".as_ref(),
                FramedResult::None
            ))
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
            Ok((b"after".as_ref(), FramedResult::Ok(b"foo".to_vec())))
        );
    }
}
