use crate::protocol;
use crate::protocol::crc;
use crate::protocol::serialize;
use crate::protocol::{
    HostMessage, Message, MessageParse, MessageSerialize, Parse, ParseMut, ParseResult,
    RadioMessage, MAX_FRAME_SIZE,
};

/// Re-export to allow using [Client] with [std::io] streams.
#[cfg(feature = "std")]
pub use embedded_io_adapters::std::FromStd;

/// An error type for [Client].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClientError<E> {
    /// EOF in underlying stream.
    UnexpectedEof,
    /// Other IO error in underlying stream.
    Io(E),
}

#[cfg(feature = "std")]
impl<E> std::error::Error for ClientError<E> where E: core::fmt::Debug {}

impl<E> core::fmt::Display for ClientError<E>
where
    E: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected eof"),
            Self::Io(e) => write!(f, "io error: {:?}", e),
        }
    }
}

impl<E> From<E> for ClientError<E> {
    fn from(other: E) -> Self {
        Self::Io(other)
    }
}

/// A trait to encapsulate a buffer with filled and unfilled areas.
pub trait ClientBuffer {
    type Slice<'a>: ParseMut
    where
        Self: 'a;

    /// Modify the filled part of the buffer to remove the first `n` bytes.
    fn skip(&mut self, n: usize);

    /// Returns [true] if the buffer is full.
    fn is_full(&self) -> bool;

    /// Read data from a reader into the filled part, consuming unfilled areas.
    fn read<R>(&mut self, reader: &mut R) -> Result<usize, R::Error>
    where
        R: embedded_io::Read;

    /// Get a hold of the accumulated data to do some parsin'.
    fn data_mut(&mut self) -> Self::Slice<'_>;

    /// Get a hold of the accumulated data to do some introspectin'.
    fn data(&self) -> <Self::Slice<'_> as ParseMut>::Input;

    /// Clear the buffer.
    fn clear(&mut self);
}

// would be nice to do this for &'b mut B, but 'static is all we really use
impl<B> ClientBuffer for &'static mut B
where
    B: ClientBuffer,
{
    type Slice<'a> = B::Slice<'a>;

    fn skip(&mut self, n: usize) {
        (**self).skip(n)
    }

    fn is_full(&self) -> bool {
        (**self).is_full()
    }

    fn read<R>(&mut self, reader: &mut R) -> Result<usize, R::Error>
    where
        R: embedded_io::Read,
    {
        (**self).read(reader)
    }

    fn data_mut(&mut self) -> Self::Slice<'_> {
        (**self).data_mut()
    }

    fn data(&self) -> <Self::Slice<'_> as ParseMut>::Input {
        (**self).data()
    }

    fn clear(&mut self) {
        (**self).clear()
    }
}

/// A [ClientBuffer] using a flat array.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ArrayBuffer<const SIZE: usize = MAX_FRAME_SIZE> {
    len: usize,
    buffer: [u8; SIZE],
}

impl<const SIZE: usize> ArrayBuffer<SIZE> {
    pub const fn new() -> Self {
        Self {
            len: 0,
            buffer: [0u8; SIZE],
        }
    }
}

impl<const SIZE: usize> Default for ArrayBuffer<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> ClientBuffer for ArrayBuffer<SIZE> {
    type Slice<'a> = &'a mut [u8];

    fn skip(&mut self, n: usize) {
        self.buffer.copy_within(n..self.len, 0);
        self.len -= n.min(self.len);
    }

    fn is_full(&self) -> bool {
        self.len >= SIZE
    }

    fn read<R>(&mut self, reader: &mut R) -> Result<usize, R::Error>
    where
        R: embedded_io::Read,
    {
        let amt = reader.read(&mut self.buffer[self.len..])?;
        self.len += amt;
        Ok(amt)
    }

    fn data_mut(&mut self) -> Self::Slice<'_> {
        &mut self.buffer[..self.len]
    }

    fn data(&self) -> <Self::Slice<'_> as ParseMut>::Input {
        &self.buffer[..self.len]
    }

    fn clear(&mut self) {
        self.len = 0;
    }
}

/// A client for the UV-K5 serial protocol.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Client<F, B, InC, OutC> {
    port: F,
    buffer: B,
    skip: Option<usize>,
    needs_read: bool,
    in_crc: InC,
    out_crc: OutC,
}

/// A host-sided client.
pub type ClientHost<F, B = ArrayBuffer> = Client<F, B, crc::CrcConstantIgnore, crc::CrcXModem>;

/// A host-sided client using an [std::io] port.
#[cfg(feature = "std")]
pub type ClientHostStd<F, B = ArrayBuffer> = ClientHost<FromStd<F>, B>;

impl<F, B> ClientHost<F, B>
where
    B: ClientBuffer,
{
    /// Create a new host client.
    pub fn new(port: F) -> Self
    where
        B: Default,
    {
        Self::new_with(B::default(), port)
    }

    /// Create a new host client with the provided internal buffer.
    pub fn new_with(buffer: B, port: F) -> Self {
        Client::new_crc_with(
            buffer,
            crc::CrcConstantIgnore(0xffff),
            crc::CrcXModem::new(),
            port,
        )
    }
}

#[cfg(feature = "std")]
impl<F, B> ClientHost<FromStd<F>, B>
where
    B: ClientBuffer,
{
    /// Create a new host client using an [std::io] port.
    pub fn new_std(port: F) -> Self
    where
        B: Default,
    {
        Self::new(FromStd::new(port))
    }

    /// Create a new host client using and [std::io] port and the
    /// provided internal buffer.
    pub fn new_std_with(buffer: B, port: F) -> Self {
        Self::new_with(buffer, FromStd::new(port))
    }
}

/// A radio-sided client.
pub type ClientRadio<F, B = ArrayBuffer> = Client<F, B, crc::CrcXModem, crc::CrcConstantIgnore>;

/// A radio-sided client using an [std::io] port.
#[cfg(feature = "std")]
pub type ClientRadioStd<F, B = ArrayBuffer> = ClientRadio<FromStd<F>, B>;

impl<F, B> ClientRadio<F, B>
where
    B: ClientBuffer,
{
    /// Create a new radio client.
    pub fn new(port: F) -> Self
    where
        B: Default,
    {
        Self::new_with(B::default(), port)
    }

    /// Create a new radio client with the provided internal buffer.
    pub fn new_with(buffer: B, port: F) -> Self {
        Client::new_crc_with(
            buffer,
            crc::CrcXModem::new(),
            crc::CrcConstantIgnore(0xffff),
            port,
        )
    }
}

#[cfg(feature = "std")]
impl<F, B> ClientRadio<FromStd<F>, B>
where
    B: ClientBuffer,
{
    /// Create a new radio client using an [std::io] port.
    pub fn new_std(port: F) -> Self
    where
        B: Default,
    {
        Self::new(FromStd::new(port))
    }

    /// Create a new radio client using and [std::io] port and the
    /// provided internal buffer.
    pub fn new_std_with(buffer: B, port: F) -> Self {
        Self::new_with(buffer, FromStd::new(port))
    }
}

impl<F, B, InC, OutC> Client<F, B, InC, OutC>
where
    B: ClientBuffer,
    InC: crc::CrcStyle,
    OutC: crc::CrcStyle,
{
    /// Create a new client with the provided incoming and
    /// outgoing [crc::CrcStyle]s.
    pub fn new_crc(in_crc: InC, out_crc: OutC, port: F) -> Self
    where
        B: Default,
    {
        Self::new_crc_with(B::default(), in_crc, out_crc, port)
    }

    /// Create a new client with the provided buffer and CRCs.
    pub fn new_crc_with(buffer: B, in_crc: InC, out_crc: OutC, port: F) -> Self {
        Self {
            port,
            buffer,
            skip: None,
            needs_read: true,
            in_crc,
            out_crc,
        }
    }

    /// Release the components used to create this client.
    pub fn free(self) -> (B, InC, OutC, F) {
        (self.buffer, self.in_crc, self.out_crc, self.port)
    }

    /// Get the underlying buffer.
    pub fn buffer(&self) -> &B {
        &self.buffer
    }

    /// Get the underlying buffer, mutably.
    ///
    /// Be careful mutating this, as it may cause the client to become
    /// confused.
    pub fn buffer_mut(&mut self) -> &mut B {
        &mut self.buffer
    }

    /// Get the underlying port.
    pub fn port(&self) -> &F {
        &self.port
    }

    /// Get the underlying port, mutably
    ///
    /// Using this won't confuse the client, but it might cause you to miss
    /// messages if you are not careful.
    pub fn port_mut(&mut self) -> &mut F {
        &mut self.port
    }

    /// Get the incoming [crc::CrcStyle] implementation.
    pub fn in_crc(&self) -> &InC {
        &self.in_crc
    }

    /// Get the outgoing [crc::CrcStyle] implementation.
    pub fn out_crc(&self) -> &OutC {
        &self.out_crc
    }

    /// Read from the port into the internal buffer, if needed. First
    /// half of [Self::read()].
    ///
    /// If you call this while [self.buffer().is_full()][ClientBuffer::is_full],
    /// this will clear the internal buffer to make room for new data.
    pub fn read_into_buffer(&mut self) -> Result<(), ClientError<F::Error>>
    where
        F: embedded_io::Read,
    {
        // apply skip from last read cycle. see parse().
        if let Some(skip) = self.skip.take() {
            self.buffer.skip(skip);
        }

        // if the buffer is full, even now, clear it and restart
        if self.buffer.is_full() {
            self.buffer.clear();
            self.needs_read = true;
        }

        // if we've cleared the buffer, or if the last parse found nothing,
        // we need to read more data
        if self.needs_read {
            let amt = self.buffer.read(&mut self.port)?;
            if amt == 0 {
                // end of file is an error
                return Err(ClientError::UnexpectedEof);
            }
            self.needs_read = false;
        }

        Ok(())
    }

    /// Parse from the internal buffer. Second half of [Self::read()].
    pub fn parse<'a, M, I>(&'a mut self) -> ParseResult<I, M>
    where
        M: MessageParse<I>,
        I: Parse,
        B::Slice<'a>: ParseMut<Input = I>,
    {
        // attempt to parse it
        let (skip, res) = protocol::parse(&self.in_crc, self.buffer.data_mut());

        if let ParseResult::None = res {
            // we didn't find anything, not even an error, so we need more data
            self.needs_read = true;
        }

        // store the skip value until next read_into_buffer, because
        // modifying self.buffer would interfere with the borrow in res
        if skip > 0 {
            self.skip = Some(skip);
        }

        res
    }

    /// Read from the port and attempt to parse a message.
    pub fn read<'a, M, I>(&'a mut self) -> Result<ParseResult<I, M>, ClientError<F::Error>>
    where
        M: MessageParse<I>,
        I: Parse,
        B::Slice<'a>: ParseMut<Input = I>,
        F: embedded_io::Read,
    {
        self.read_into_buffer()?;
        Ok(self.parse())
    }

    /// Read a [Message].
    pub fn read_any<'a, I>(
        &'a mut self,
    ) -> Result<ParseResult<I, Message<I>>, ClientError<F::Error>>
    where
        I: Parse,
        B::Slice<'a>: ParseMut<Input = I>,
        F: embedded_io::Read,
    {
        self.read()
    }

    /// Read a [HostMessage].
    pub fn read_host<'a, I>(
        &'a mut self,
    ) -> Result<ParseResult<I, HostMessage<I>>, ClientError<F::Error>>
    where
        I: Parse,
        B::Slice<'a>: ParseMut<Input = I>,
        F: embedded_io::Read,
    {
        self.read()
    }

    /// Read a [RadioMessage].
    pub fn read_radio<'a, I>(
        &'a mut self,
    ) -> Result<ParseResult<I, RadioMessage<I>>, ClientError<F::Error>>
    where
        I: Parse,
        B::Slice<'a>: ParseMut<Input = I>,
        F: embedded_io::Read,
    {
        self.read()
    }

    /// Write a message to the port.
    pub fn write<M>(&mut self, msg: &M) -> Result<(), ClientError<F::Error>>
    where
        F: embedded_io::Write,
        M: MessageSerialize,
    {
        let mut ser = serialize::SerializerWrap::new(&mut self.port);
        protocol::serialize(&self.out_crc, &mut ser, msg)?;
        self.port.flush()?;
        Ok(())
    }
}
