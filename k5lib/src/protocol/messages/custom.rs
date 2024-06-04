//! Messages not used in the stock firmware, but possibly useful.

use nom::{error::Error, Parser};

use crate::protocol::parse::{MessageParse, Parse};
use crate::protocol::serialize::{MessageSerialize, Serializer};

use super::MessageType;

/// 0x8500 Debug Input, host message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DebugInput<I> {
    pub line: I,
}

impl<I> MessageType for DebugInput<I> {
    const TYPE: u16 = 0x8500;
}

impl<I> DebugInput<I> {
    pub fn map<F, J>(self, f: F) -> DebugInput<J>
    where
        F: FnOnce(I) -> J,
    {
        DebugInput { line: f(self.line) }
    }

    pub fn map_ref<'a, F, J>(&'a self, f: F) -> DebugInput<J>
    where
        F: FnOnce(&'a I) -> J,
    {
        DebugInput {
            line: f(&self.line),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> DebugInput<I::Owned>
    where
        I: alloc::borrow::ToOwned,
    {
        self.map_ref(I::to_owned)
    }

    pub fn borrow<Borrowed: ?Sized>(&self) -> DebugInput<&Borrowed>
    where
        I: core::borrow::Borrow<Borrowed>,
    {
        self.map_ref(I::borrow)
    }
}

impl<I> MessageSerialize for DebugInput<I>
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
        ser.write_slice(&self.line)
    }
}

impl<I> MessageParse<I> for DebugInput<I>
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

            let (input, line) = nom::bytes::complete::take_till(|_| false)(input)?;

            Ok((input, DebugInput { line }))
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

    impl Arbitrary for DebugInput<Vec<u8>> {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                line: Vec::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn roundtrip_debug_input(msg: DebugInput<Vec<u8>>) -> bool {
        RoundTrip::new().run(&msg.borrow())
    }
}
