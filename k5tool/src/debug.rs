use k5lib::protocol::crc;
use k5lib::protocol::serialize::{Serializer, SerializerWrap};
use k5lib::protocol::{
    parse, serialize, HostMessage, Message, MessageParse, MessageSerialize, ParseResult,
    RadioMessage,
};
use k5lib::ClientBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ClientDirection {
    Radio = 0,
    Host = 1,
}

impl ClientDirection {
    pub fn flip(&self) -> Self {
        match self {
            ClientDirection::Host => ClientDirection::Radio,
            ClientDirection::Radio => ClientDirection::Host,
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct DebugClientArgs {
    #[arg(short, long, action=clap::ArgAction::Count)]
    pub debug: u8,
    #[arg(long)]
    dump: Option<String>,
}

pub struct DebugClient<F, InC, OutC> {
    args: DebugClientArgs,
    client: k5lib::Client<k5lib::FromStd<F>, k5lib::ArrayBuffer, InC, OutC>,
    dump: Option<SerializerWrap<k5lib::FromStd<std::fs::File>>>,
    // direction is the message type this client *writes*
    direction: ClientDirection,
}

pub type DebugClientHost<F> = DebugClient<F, crc::CrcConstantIgnore, crc::CrcXModem>;
pub type DebugClientRadio<F> = DebugClient<F, crc::CrcXModem, crc::CrcConstantIgnore>;

impl DebugClientArgs {
    pub fn wrap_host<F>(
        &self,
        client: k5lib::ClientHostStd<F>,
    ) -> anyhow::Result<DebugClientHost<F>> {
        self.wrap(ClientDirection::Host, client)
    }

    pub fn wrap_radio<F>(
        &self,
        client: k5lib::ClientRadioStd<F>,
    ) -> anyhow::Result<DebugClientRadio<F>> {
        self.wrap(ClientDirection::Radio, client)
    }

    // direction is the type of message this client *writes*
    pub fn wrap<F, InC, OutC>(
        &self,
        direction: ClientDirection,
        client: k5lib::Client<k5lib::FromStd<F>, k5lib::ArrayBuffer, InC, OutC>,
    ) -> anyhow::Result<DebugClient<F, InC, OutC>> {
        let mut dump = None;
        if let Some(ref path) = self.dump {
            dump = Some(SerializerWrap::new(k5lib::FromStd::new(
                std::fs::File::options()
                    .create(true)
                    .append(true)
                    .open(path)?,
            )));
        }

        Ok(DebugClient {
            args: self.clone(),
            client,
            dump,
            direction,
        })
    }
}

impl<F, InC, OutC> DebugClient<F, InC, OutC>
where
    InC: crc::CrcStyle,
    OutC: crc::CrcStyle,
{
    pub fn read<'a, M>(
        &'a mut self,
    ) -> Result<ParseResult<&'a [u8], M>, k5lib::ClientError<std::io::Error>>
    where
        M: MessageParse<&'a [u8]> + std::fmt::Debug,
        F: std::io::Read,
    {
        // two-step read, to grab the buffer to look into
        self.client.read_into_buffer()?;
        // only make a copy of the data if we need it later
        let data = (self.args.debug >= 2 || self.dump.is_some())
            .then(|| self.client.buffer().data().to_owned());
        let res = self.client.parse();

        if let Some(mut data) = data {
            // if this data produced a frame, we have to find it.
            if let Some(range) = res.range() {
                if self.args.debug >= 3 || self.dump.is_some() {
                    let raw = &data[range.clone()];

                    if let Some(ref mut dump) = self.dump {
                        dump.write_u8(self.direction.flip() as u8)?;
                        dump.write_le_u16(raw.len() as u16)?;
                        dump.write_bytes(raw)?;
                    }

                    if self.args.debug >= 3 {
                        eprintln!("<<< raw frame:");
                        crate::hexdump::ehexdump_prefix("<<<   ", raw);
                    }
                }

                if self.args.debug >= 2 {
                    if let (_, Some(found)) = parse::find_frame(data.as_mut()) {
                        eprintln!("<<< deobfuscated:");
                        crate::hexdump::ehexdump_prefix("<<<   ", &data[found.full_frame]);
                    }
                }
            }
        }

        if self.args.debug >= 1 {
            match res {
                ParseResult::Ok(_, ref m) => {
                    eprintln!("<<< {:?}", m);
                    eprintln!();
                }
                ParseResult::ParseErr(_, inp, ref e) => {
                    eprintln!("!!! parse error: {:?}", e);
                    crate::hexdump::ehexdump_prefix("!!!   ", inp.to_vec().as_ref());
                    eprintln!();
                }
                ParseResult::CrcErr(_, inp) => {
                    eprintln!("!!! crc error:");
                    crate::hexdump::ehexdump_prefix("!!!   ", inp.to_vec().as_ref());
                    eprintln!();
                }
                ParseResult::None => {}
            }
        }
        Ok(res)
    }

    /// Read a Message.
    #[allow(clippy::type_complexity)]
    pub fn read_any(
        &mut self,
    ) -> Result<ParseResult<&[u8], Message<&[u8]>>, k5lib::ClientError<std::io::Error>>
    where
        F: std::io::Read,
    {
        self.read()
    }

    /// Read a HostMessage.
    #[allow(clippy::type_complexity)]
    pub fn read_host(
        &mut self,
    ) -> Result<ParseResult<&[u8], HostMessage<&[u8]>>, k5lib::ClientError<std::io::Error>>
    where
        F: std::io::Read,
    {
        self.read()
    }

    /// Read a RadioMessage.
    #[allow(clippy::type_complexity)]
    pub fn read_radio(
        &mut self,
    ) -> Result<ParseResult<&[u8], RadioMessage<&[u8]>>, k5lib::ClientError<std::io::Error>>
    where
        F: std::io::Read,
    {
        self.read()
    }

    /// Write a message to the port.
    pub fn write<M>(&mut self, msg: &M) -> Result<(), k5lib::ClientError<std::io::Error>>
    where
        F: std::io::Write,
        M: MessageSerialize + std::fmt::Debug,
    {
        // also making some assumptions here about how write is implemented
        if self.args.debug >= 3 || self.dump.is_some() {
            let mut ser = serialize::SerializerVec::new();
            msg.frame(self.client.out_crc(), &mut ser)
                .unwrap_or_else(|e| match e {});
            let raw = ser.done();

            if let Some(ref mut dump) = self.dump {
                dump.write_u8(self.direction as u8)?;
                dump.write_le_u16(raw.len() as u16)?;
                dump.write_bytes(&raw)?;
            }

            if self.args.debug >= 3 {
                eprintln!(">>> raw frame:");
                crate::hexdump::ehexdump_prefix(">>>   ", &raw);
            }
        }
        if self.args.debug >= 2 {
            let mut ser = serialize::SerializerVec::new();
            msg.frame_body_crc(self.client.out_crc(), &mut ser)
                .unwrap_or_else(|e| match e {});
            eprintln!(">>> deobfuscated:");
            crate::hexdump::ehexdump_prefix(">>>   ", &ser.done());
        }
        if self.args.debug >= 1 {
            eprintln!(">>> {:?}", msg);
            eprintln!();
        }
        self.client.write(msg)
    }
}
