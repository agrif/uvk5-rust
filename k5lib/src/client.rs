use crate::protocol;
use crate::protocol::crc;
use crate::protocol::obfuscation;
use crate::protocol::{
    HostMessage, Message, MessageParse, MessageSerialize, ParseResult, RadioMessage, MAX_FRAME_SIZE,
};

/// A trait to encapsulate a buffer with filled and unfilled areas.
pub trait ClientBuffer {
    type Slice<'a>: protocol::InputParse
    where
        Self: 'a;

    /// Modify the filled part of the buffer to keep only the last n bytes.
    fn keep_last(&mut self, n: usize);

    /// Returns true if the buffer is full.
    fn is_full(&self) -> bool;

    /// Read data from a reader into the filled part, consuming unfilled areas.
    fn read<R>(&mut self, reader: &mut R) -> std::io::Result<()>
    where
        R: std::io::Read;

    /// Get a hold of the accumulated data to do some parsin'.
    fn data(&self) -> Self::Slice<'_>;

    /// Clear the buffer.
    fn clear(&mut self) {
        self.keep_last(0);
    }
}

/// A ClientBuffer using a flat array.
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
    type Slice<'a> = &'a [u8];

    fn keep_last(&mut self, n: usize) {
        let new_start = self.len - n.min(self.len);
        self.buffer.copy_within(new_start..self.len, 0);
        self.len = n;
    }

    fn is_full(&self) -> bool {
        self.len >= SIZE
    }

    fn read<R>(&mut self, reader: &mut R) -> std::io::Result<()>
    where
        R: std::io::Read,
    {
        let amt = reader.read(&mut self.buffer[self.len..])?;
        self.len += amt;
        Ok(())
    }

    fn data(&self) -> Self::Slice<'_> {
        &self.buffer[..self.len]
    }
}

/// A client for the UV-K5 serial protocol.
pub struct Client<F, B, InC, OutC> {
    port: F,
    buffer: B,
    keep_last: Option<usize>,
    in_crc: InC,
    out_crc: OutC,
}

/// A host-sided client.
pub type ClientHost<F, B> = Client<F, B, crc::CrcConstantIgnore, crc::CrcXModem>;

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
pub type ClientRadio<F, B> = Client<F, B, crc::CrcXModem, crc::CrcConstantIgnore>;

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
            keep_last: None,
            in_crc,
            out_crc,
        }
    }

    /// Read from the port and attempt to parse a message.
    pub fn read<'a, M>(&'a mut self) -> std::io::Result<ParseResult<B::Slice<'a>, M>>
    where
        M: MessageParse<obfuscation::Deobfuscated<B::Slice<'a>>>,
        F: std::io::Read,
    {
        use nom::InputLength;

        // apply keep_last from last read cycle. see below.
        if let Some(keep_last) = self.keep_last.take() {
            self.buffer.keep_last(keep_last);
        }

        // if the buffer is full, even now, clear it and restart
        if self.buffer.is_full() {
            self.buffer.clear();
        }

        // fill some buffer and attempt to parse it
        self.buffer.read(&mut self.port)?;
        let (rest, res) = protocol::parse(&self.in_crc, self.buffer.data());

        // store the keep_last value until next read, because
        // modifying self.buffer would interfere with the borrow in res
        self.keep_last = Some(rest.input_len());
        Ok(res)
    }

    /// Read a Message.
    pub fn read_any(
        &mut self,
    ) -> std::io::Result<ParseResult<B::Slice<'_>, Message<obfuscation::Deobfuscated<B::Slice<'_>>>>>
    where
        F: std::io::Read,
    {
        self.read()
    }

    /// Read a HostMessage.
    pub fn read_host(&mut self) -> std::io::Result<ParseResult<B::Slice<'_>, HostMessage>>
    where
        F: std::io::Read,
    {
        self.read()
    }

    /// Read a RadioMessage.
    pub fn read_radio(
        &mut self,
    ) -> std::io::Result<
        ParseResult<B::Slice<'_>, RadioMessage<obfuscation::Deobfuscated<B::Slice<'_>>>>,
    >
    where
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
        Ok(())
    }
}
