use crate::protocol;
use crate::protocol::crc;
use crate::protocol::{
    HostMessage, Message, MessageParse, MessageSerialize, Parse, ParseMut, ParseResult,
    RadioMessage, MAX_FRAME_SIZE,
};

/// A trait to encapsulate a buffer with filled and unfilled areas.
pub trait ClientBuffer {
    type Slice<'a>: ParseMut
    where
        Self: 'a;

    /// Modify the filled part of the buffer to remove the first n bytes.
    fn skip(&mut self, n: usize);

    /// Returns true if the buffer is full.
    fn is_full(&self) -> bool;

    /// Read data from a reader into the filled part, consuming unfilled areas.
    fn read<R>(&mut self, reader: &mut R) -> std::io::Result<usize>
    where
        R: std::io::Read;

    /// Get a hold of the accumulated data to do some parsin'.
    fn data_mut(&mut self) -> Self::Slice<'_>;

    /// Get a hold of the accumulated data to do some introspectin'.
    fn data(&self) -> <Self::Slice<'_> as ParseMut>::Input;

    /// Clear the buffer.
    fn clear(&mut self);
}

/// A ClientBuffer using a flat array.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayBuffer<const SIZE: usize = { MAX_FRAME_SIZE }> {
    len: usize,
    buffer: [u8; SIZE],
}

impl<const SIZE: usize> ArrayBuffer<SIZE> {
    pub fn new() -> Self {
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

    fn read<R>(&mut self, reader: &mut R) -> std::io::Result<usize>
    where
        R: std::io::Read,
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

/// A radio-sided client.
pub type ClientRadio<F, B = ArrayBuffer> = Client<F, B, crc::CrcXModem, crc::CrcConstantIgnore>;

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

impl<F, B, InC, OutC> Client<F, B, InC, OutC>
where
    B: ClientBuffer,
    InC: crc::CrcStyle,
    OutC: crc::CrcStyle,
{
    /// Create a new client with the provided incoming and outgoing CRCs.
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

    /// Get the underlying buffer.
    pub fn buffer(&self) -> &B {
        &self.buffer
    }

    /// Get the incoming CRC implementation.
    pub fn in_crc(&self) -> &InC {
        &self.in_crc
    }

    /// Get the outgoing CRC implementation.
    pub fn out_crc(&self) -> &OutC {
        &self.out_crc
    }

    /// Read from the port into the internal buffer, if needed. First
    /// half of read().
    ///
    /// If you call this while self.buffer().is_full(), this will clear
    /// the internal buffer to make room for new data.
    pub fn read_into_buffer(&mut self) -> std::io::Result<()>
    where
        F: std::io::Read,
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
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "end of stream",
                ));
            }
            self.needs_read = false;
        }

        Ok(())
    }

    /// Parse from the internal buffer. Second half of read().
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

    /// Read from the port and attempt to parse a message, while also
    /// returning the buffer used for parsing.
    pub fn read<'a, M, I>(&'a mut self) -> std::io::Result<ParseResult<I, M>>
    where
        M: MessageParse<I>,
        I: Parse,
        B::Slice<'a>: ParseMut<Input = I>,
        F: std::io::Read,
    {
        self.read_into_buffer()?;
        Ok(self.parse())
    }

    /// Read a Message.
    pub fn read_any<'a, I>(&'a mut self) -> std::io::Result<ParseResult<I, Message<I>>>
    where
        I: Parse,
        B::Slice<'a>: ParseMut<Input = I>,
        F: std::io::Read,
    {
        self.read()
    }

    /// Read a HostMessage.
    pub fn read_host<'a, I>(&'a mut self) -> std::io::Result<ParseResult<I, HostMessage<I>>>
    where
        I: Parse,
        B::Slice<'a>: ParseMut<Input = I>,
        F: std::io::Read,
    {
        self.read()
    }

    /// Read a RadioMessage.
    pub fn read_radio<'a, I>(&'a mut self) -> std::io::Result<ParseResult<I, RadioMessage<I>>>
    where
        I: Parse,
        B::Slice<'a>: ParseMut<Input = I>,
        F: std::io::Read,
    {
        self.read()
    }

    /// Write a message to the port.
    pub fn write<M>(&mut self, msg: &M) -> std::io::Result<()>
    where
        F: std::io::Write,
        M: MessageSerialize,
    {
        protocol::serialize(&self.out_crc, &mut self.port, msg)?;
        self.port.flush()
    }
}
