//! Message types used in the protocol.

use nom::{error::Error, Parser};

use crate::protocol::parse::{MessageParse, Parse};
use crate::protocol::serialize::{MessageSerialize, Serializer};

pub mod bootloader;
pub mod custom;
pub mod radio;
pub mod util;

/// A trait for messages that have statically-known message types.
pub trait MessageType {
    const TYPE: u16;
}

/// Any kind of message, either a [HostMessage] or a [RadioMessage].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Message<I> {
    Host(HostMessage<I>),
    Radio(RadioMessage<I>),
}

impl<I> Message<I> {
    pub fn map<F, J>(self, f: F) -> Message<J>
    where
        F: FnOnce(I) -> J,
    {
        match self {
            Self::Host(o) => Message::Host(o.map(f)),
            Self::Radio(o) => Message::Radio(o.map(f)),
        }
    }

    pub fn map_ref<'a, F, J>(&'a self, f: F) -> Message<J>
    where
        F: FnOnce(&'a I) -> J,
    {
        match self {
            Self::Host(o) => Message::Host(o.map_ref(f)),
            Self::Radio(o) => Message::Radio(o.map_ref(f)),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> Message<I::Owned>
    where
        I: alloc::borrow::ToOwned,
    {
        self.map_ref(I::to_owned)
    }

    pub fn borrow<Borrowed: ?Sized>(&self) -> Message<&Borrowed>
    where
        I: core::borrow::Borrow<Borrowed>,
    {
        self.map_ref(I::borrow)
    }
}

impl<I> MessageSerialize for Message<I>
where
    I: Parse,
{
    fn message_type(&self) -> u16 {
        match self {
            Self::Host(m) => m.message_type(),
            Self::Radio(m) => m.message_type(),
        }
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Host(m) => m.message_body(ser),
            Self::Radio(m) => m.message_body(ser),
        }
    }
}

impl<I> MessageParse<I> for Message<I>
where
    I: Parse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        nom::branch::alt((
            nom::combinator::map(HostMessage::parse_body(typ), Message::Host),
            nom::combinator::map(RadioMessage::parse_body(typ), Message::Radio),
        ))
    }
}

/// Messages sent from the host computer to the radio.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HostMessage<I> {
    /// 0x0514 Hello
    Hello(radio::Hello),
    /// 0x0519 Write Flash (bootloader mode)
    WriteFlash(bootloader::WriteFlash<I>),
    /// 0x051b Read EEPROM
    ReadEeprom(radio::ReadEeprom),
    /// 0x0530 Bootloader Ready Reply (bootloader mode)
    BootloaderReadyReply(bootloader::BootloaderReadyReply),

    /// 0x8500 Debug Input (custom)
    DebugInput(custom::DebugInput<I>),
}

impl<I> HostMessage<I> {
    pub fn map<F, J>(self, f: F) -> HostMessage<J>
    where
        F: FnOnce(I) -> J,
    {
        match self {
            Self::Hello(o) => HostMessage::Hello(o),
            Self::WriteFlash(o) => HostMessage::WriteFlash(o.map(f)),
            Self::ReadEeprom(o) => HostMessage::ReadEeprom(o),
            Self::BootloaderReadyReply(o) => HostMessage::BootloaderReadyReply(o),

            Self::DebugInput(o) => HostMessage::DebugInput(o.map(f)),
        }
    }

    pub fn map_ref<'a, F, J>(&'a self, f: F) -> HostMessage<J>
    where
        F: FnOnce(&'a I) -> J,
    {
        match self {
            Self::Hello(o) => HostMessage::Hello(o.clone()),
            Self::WriteFlash(o) => HostMessage::WriteFlash(o.map_ref(f)),
            Self::ReadEeprom(o) => HostMessage::ReadEeprom(o.clone()),
            Self::BootloaderReadyReply(o) => HostMessage::BootloaderReadyReply(o.clone()),

            Self::DebugInput(o) => HostMessage::DebugInput(o.map_ref(f)),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> HostMessage<I::Owned>
    where
        I: alloc::borrow::ToOwned,
    {
        self.map_ref(I::to_owned)
    }

    pub fn borrow<Borrowed: ?Sized>(&self) -> HostMessage<&Borrowed>
    where
        I: core::borrow::Borrow<Borrowed>,
    {
        self.map_ref(I::borrow)
    }
}

impl<I> MessageSerialize for HostMessage<I>
where
    I: Parse,
{
    fn message_type(&self) -> u16 {
        match self {
            Self::Hello(m) => m.message_type(),
            Self::WriteFlash(m) => m.message_type(),
            Self::ReadEeprom(m) => m.message_type(),
            Self::BootloaderReadyReply(m) => m.message_type(),

            Self::DebugInput(m) => m.message_type(),
        }
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Hello(m) => m.message_body(ser),
            Self::WriteFlash(m) => m.message_body(ser),
            Self::ReadEeprom(m) => m.message_body(ser),
            Self::BootloaderReadyReply(m) => m.message_body(ser),

            Self::DebugInput(m) => m.message_body(ser),
        }
    }
}

impl<I> MessageParse<I> for HostMessage<I>
where
    I: Parse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| match typ {
            radio::Hello::TYPE => radio::Hello::parse_body(typ).map(Self::Hello).parse(input),
            bootloader::WriteFlash::<()>::TYPE => bootloader::WriteFlash::parse_body(typ)
                .map(Self::WriteFlash)
                .parse(input),
            radio::ReadEeprom::TYPE => radio::ReadEeprom::parse_body(typ)
                .map(Self::ReadEeprom)
                .parse(input),
            bootloader::BootloaderReadyReply::TYPE => {
                bootloader::BootloaderReadyReply::parse_body(typ)
                    .map(Self::BootloaderReadyReply)
                    .parse(input)
            }

            custom::DebugInput::<()>::TYPE => custom::DebugInput::parse_body(typ)
                .map(Self::DebugInput)
                .parse(input),

            // we don't recognize the message type
            _ => nom::combinator::fail(input),
        }
    }
}

/// Messages sent from the radio to the host computer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RadioMessage<I> {
    /// 0x0515 HelloReply
    HelloReply(radio::HelloReply),
    /// 0x0518 Bootloader Ready (bootloader mode)
    BootloaderReady(bootloader::BootloaderReady),
    /// 0x051a Write Flash Reply (bootloader mode)
    WriteFlashReply(bootloader::WriteFlashReply),
    /// 0x51c Read EEPROM Reply
    ReadEepromReply(radio::ReadEepromReply<I>),

    /// 0x8501 Debug Output (custom)
    DebugOutput(custom::DebugOutput<I>),
}

impl<I> RadioMessage<I> {
    pub fn map<F, J>(self, f: F) -> RadioMessage<J>
    where
        F: FnOnce(I) -> J,
    {
        match self {
            Self::HelloReply(o) => RadioMessage::HelloReply(o),
            Self::BootloaderReady(o) => RadioMessage::BootloaderReady(o),
            Self::WriteFlashReply(o) => RadioMessage::WriteFlashReply(o),
            Self::ReadEepromReply(o) => RadioMessage::ReadEepromReply(o.map(f)),

            Self::DebugOutput(o) => RadioMessage::DebugOutput(o.map(f)),
        }
    }

    pub fn map_ref<'a, F, J>(&'a self, f: F) -> RadioMessage<J>
    where
        F: FnOnce(&'a I) -> J,
    {
        match self {
            Self::HelloReply(o) => RadioMessage::HelloReply(o.clone()),
            Self::BootloaderReady(o) => RadioMessage::BootloaderReady(o.clone()),
            Self::WriteFlashReply(o) => RadioMessage::WriteFlashReply(o.clone()),
            Self::ReadEepromReply(o) => RadioMessage::ReadEepromReply(o.map_ref(f)),

            Self::DebugOutput(o) => RadioMessage::DebugOutput(o.map_ref(f)),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> RadioMessage<I::Owned>
    where
        I: alloc::borrow::ToOwned,
    {
        self.map_ref(I::to_owned)
    }

    pub fn borrow<Borrowed: ?Sized>(&self) -> RadioMessage<&Borrowed>
    where
        I: core::borrow::Borrow<Borrowed>,
    {
        self.map_ref(I::borrow)
    }
}

impl<I> MessageSerialize for RadioMessage<I>
where
    I: Parse,
{
    fn message_type(&self) -> u16 {
        match self {
            Self::HelloReply(m) => m.message_type(),
            Self::BootloaderReady(m) => m.message_type(),
            Self::WriteFlashReply(m) => m.message_type(),
            Self::ReadEepromReply(m) => m.message_type(),

            Self::DebugOutput(m) => m.message_type(),
        }
    }

    fn message_body<S>(&self, ser: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::HelloReply(m) => m.message_body(ser),
            Self::BootloaderReady(m) => m.message_body(ser),
            Self::WriteFlashReply(m) => m.message_body(ser),
            Self::ReadEepromReply(m) => m.message_body(ser),

            Self::DebugOutput(m) => m.message_body(ser),
        }
    }
}

impl<I> MessageParse<I> for RadioMessage<I>
where
    I: Parse,
{
    fn parse_body(typ: u16) -> impl Parser<I, Self, Error<I>> {
        move |input| match typ {
            radio::HelloReply::TYPE => radio::HelloReply::parse_body(typ)
                .map(Self::HelloReply)
                .parse(input),
            bootloader::BootloaderReady::TYPE => bootloader::BootloaderReady::parse_body(typ)
                .map(Self::BootloaderReady)
                .parse(input),
            bootloader::WriteFlashReply::TYPE => bootloader::WriteFlashReply::parse_body(typ)
                .map(Self::WriteFlashReply)
                .parse(input),
            radio::ReadEepromReply::<()>::TYPE => radio::ReadEepromReply::parse_body(typ)
                .map(Self::ReadEepromReply)
                .parse(input),

            custom::DebugOutput::<()>::TYPE => custom::DebugOutput::parse_body(typ)
                .map(Self::DebugOutput)
                .parse(input),

            // we don't recognize the message type
            _ => nom::combinator::fail(input),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "alloc")]
mod test {
    use alloc::vec::Vec;

    use crate::protocol::crc::CrcXModem;
    use crate::protocol::{find_frame, parse, serialize};

    use super::*;

    pub(super) fn roundtrip<M>(msg: M) -> bool
    where
        M: for<'a> MessageParse<&'a [u8]> + MessageSerialize + PartialEq + Eq,
    {
        RoundTrip::new().run(&msg)
    }

    pub(super) struct RoundTrip(Vec<u8>);

    impl RoundTrip {
        pub(super) fn new() -> Self {
            Self(Default::default())
        }

        pub(super) fn run<'a, M>(&'a mut self, msg: &M) -> bool
        where
            M: MessageSerialize + MessageParse<&'a [u8]> + PartialEq + Eq,
        {
            Some(msg) == self.ser(msg).de().as_ref()
        }

        pub(super) fn ser<M>(&mut self, msg: &M) -> &mut Self
        where
            M: MessageSerialize,
        {
            let crc = CrcXModem::new();
            let mut serialized = serialize::SerializerVec::new();
            if serialize(&crc, &mut serialized, msg).is_ok() {
                self.0 = serialized.done();
            }
            self
        }

        pub(super) fn de<'a, M>(&'a mut self) -> Option<M>
        where
            M: MessageParse<&'a [u8]>,
        {
            let crc = CrcXModem::new();
            let len = self.0.len();
            let (amt, found) = find_frame(self.0.as_mut());
            let unserialized = parse(&crc, self.0.as_ref(), &found);
            if amt != len {
                None
            } else {
                unserialized.ok()
            }
        }
    }
}
