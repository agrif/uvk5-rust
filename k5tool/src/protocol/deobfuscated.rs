use nom::error::Error;

use super::CrcStyle;

pub const OBFUSCATION: [u8; 16] = [
    0x16, 0x6c, 0x14, 0xe6, 0x2e, 0x91, 0x0d, 0x40, 0x21, 0x35, 0xd5, 0x40, 0x13, 0x03, 0xe9, 0x80,
];

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
    Self: nom::InputLength,
{
    pub fn len(&self) -> usize {
        use nom::InputLength;
        self.input_len()
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

            let mut digest = crc.digest();
            for b in prefix.iter() {
                crc.update(&mut digest, &[b]);
            }
            let calculated = crc.finalize(digest);

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
