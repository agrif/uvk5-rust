//! General parsing utilities.

use crate::protocol::parse::Parse;
use crate::protocol::serialize::Serializer;

/// Parse a [crate::Version].
pub fn parse_version<I>(input: I) -> nom::IResult<I, crate::Version>
where
    I: Parse,
{
    let (input, data) = parse_array(nom::number::complete::u8)(input)?;
    Ok((input, crate::Version::new(data)))
}

/// Parse a statically-sized array with a parser.
pub fn parse_array<I, P, A, const LEN: usize>(
    parser: P,
) -> impl FnMut(I) -> nom::IResult<I, [A; LEN]>
where
    I: Parse,
    P: Fn(I) -> nom::IResult<I, A>,
    A: Default + Copy,
{
    move |input| {
        let mut data = [A::default(); LEN];
        let (input, _) = nom::multi::fill(&parser, &mut data[..])(input)?;
        Ok((input, data))
    }
}

/// Padding, in a struct.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Padding<const LEN: usize>([u8; LEN]);

impl<const LEN: usize> Padding<LEN> {
    pub fn new() -> Self {
        Self::new_data([0; LEN])
    }

    pub fn new_data(data: [u8; LEN]) -> Self {
        Self(data)
    }

    pub fn data(&self) -> &[u8; LEN] {
        &self.0
    }

    pub fn data_mut(&mut self) -> &mut [u8; LEN] {
        &mut self.0
    }

    pub fn parse<I>(input: I) -> nom::IResult<I, Self>
    where
        I: Parse,
    {
        let (input, data) = parse_array(nom::number::complete::u8)(input)?;
        Ok((input, Self::new_data(data)))
    }

    pub fn serialize<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        ser.write_bytes(&self.0)
    }
}

impl<const LEN: usize> Default for Padding<LEN> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const LEN: usize> core::fmt::Debug for Padding<LEN> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        if self.0.iter().all(|b| *b == 0) {
            f.debug_tuple("Padding").finish()
        } else {
            f.debug_tuple("Padding").field(&self.0).finish()
        }
    }
}

#[cfg(feature = "defmt")]
impl<const LEN: usize> defmt::Format for Padding<LEN> {
    fn format(&self, f: defmt::Formatter) {
        if self.0.iter().all(|b| *b == 0) {
            defmt::write!(f, "Padding");
        } else {
            defmt::write!(f, "Padding({})", self.0);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "alloc")]
mod test {
    use alloc::vec::Vec;

    use quickcheck::{Arbitrary, Gen};

    use super::*;

    impl Arbitrary for crate::Version {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut version = Vec::<u8>::arbitrary(g);
            version.truncate(crate::VERSION_LEN);
            crate::Version::new_from_bytes(&version).unwrap()
        }
    }

    impl<const LEN: usize> Arbitrary for Padding<LEN> {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut data = [0; LEN];
            for b in data.iter_mut() {
                *b = u8::arbitrary(g);
            }
            Padding::new_data(data)
        }
    }
}
