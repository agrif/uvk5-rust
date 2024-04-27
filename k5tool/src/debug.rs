use k5lib::protocol::crc;
use k5lib::protocol::obfuscation;
use k5lib::protocol::serialize::{Serializer, SerializerWrap};
use k5lib::protocol::{
    parse, serialize, HostMessage, Message, MessageParse, MessageSerialize, ParseResult,
    RadioMessage, FRAME_END, FRAME_START,
};
use k5lib::ClientBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ClientDirection {
    Host = 0,
    Radio = 1,
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

#[derive(Debug)]
pub struct DebugClient<F, InC, OutC> {
    args: DebugClientArgs,
    client: k5lib::Client<F, k5lib::ArrayBuffer, InC, OutC>,
    dump: Option<SerializerWrap<std::fs::File>>,
    // direction is the message type this client *writes*
    direction: ClientDirection,
}

pub type DebugClientHost<F> = DebugClient<F, crc::CrcConstantIgnore, crc::CrcXModem>;
pub type DebugClientRadio<F> = DebugClient<F, crc::CrcXModem, crc::CrcConstantIgnore>;

impl DebugClientArgs {
    pub fn wrap_host<F>(&self, client: k5lib::ClientHost<F>) -> anyhow::Result<DebugClientHost<F>> {
        self.wrap(ClientDirection::Host, client)
    }

    pub fn wrap_radio<F>(
        &self,
        client: k5lib::ClientRadio<F>,
    ) -> anyhow::Result<DebugClientRadio<F>> {
        self.wrap(ClientDirection::Radio, client)
    }

    // direction is the type of message this client *writes*
    pub fn wrap<F, InC, OutC>(
        &self,
        direction: ClientDirection,
        client: k5lib::Client<F, k5lib::ArrayBuffer, InC, OutC>,
    ) -> anyhow::Result<DebugClient<F, InC, OutC>> {
        let mut dump = None;
        if let Some(ref path) = self.dump {
            dump = Some(SerializerWrap::new(
                std::fs::File::options()
                    .create(true)
                    .append(true)
                    .open(path)?,
            ));
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
    pub fn read<'a, M>(&'a mut self) -> std::io::Result<ParseResult<&'a [u8], M>>
    where
        M: MessageParse<obfuscation::Deobfuscated<&'a [u8]>> + std::fmt::Debug,
        F: std::io::Read,
    {
        let (buf, res) = self.client.read_debug()?;

        if self.args.debug >= 2 || self.dump.is_some() {
            let data = buf.data();
            // if this data produced a frame, we have to find it. This
            // involves some assumptions, which we check
            if let (rest, Some(frame)) = parse::frame_raw(data) {
                if self.args.debug >= 3 || self.dump.is_some() {
                    let end = data.len() - rest.len();
                    let body_end = end - FRAME_END.len();
                    assert_eq!(data[body_end..end], FRAME_END);
                    let body_start = body_end - frame.len();
                    let len_start = body_start - 2; // 2 byte length
                    let start = len_start - FRAME_START.len();
                    assert_eq!(data[start..len_start], FRAME_START);

                    let raw = &data[start..end];

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
                    let deob = obfuscation::Deobfuscated::new(frame);
                    eprintln!("<<< deobfuscated:");
                    crate::hexdump::ehexdump_prefix("<<<   ", &deob.to_vec());
                }
            }
        }

        if self.args.debug >= 1 {
            match res {
                ParseResult::Ok(ref m) => {
                    eprintln!("<<< {:?}", m);
                    eprintln!();
                }
                ParseResult::ParseErr(ref inp, ref e) => {
                    eprintln!("!!! parse error: {:?}", e);
                    crate::hexdump::ehexdump_prefix("!!!   ", inp.to_vec().as_ref());
                    eprintln!();
                }
                ParseResult::CrcErr(ref inp) => {
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
    pub fn read_any(
        &mut self,
    ) -> std::io::Result<ParseResult<&[u8], Message<obfuscation::Deobfuscated<&[u8]>>>>
    where
        F: std::io::Read,
    {
        self.read()
    }

    /// Read a HostMessage.
    pub fn read_host(&mut self) -> std::io::Result<ParseResult<&[u8], HostMessage>>
    where
        F: std::io::Read,
    {
        self.read()
    }

    /// Read a RadioMessage.
    pub fn read_radio(
        &mut self,
    ) -> std::io::Result<ParseResult<&[u8], RadioMessage<obfuscation::Deobfuscated<&[u8]>>>>
    where
        F: std::io::Read,
    {
        self.read()
    }

    /// Write a message to the port.
    pub fn write<M>(&mut self, msg: &M) -> std::io::Result<()>
    where
        F: std::io::Write,
        M: MessageSerialize + std::fmt::Debug,
    {
        // also making some assumptions here about how write is implemented
        if self.args.debug >= 3 || self.dump.is_some() {
            let mut ser = serialize::SerializerWrap::new(Vec::new());
            msg.frame(self.client.out_crc(), &mut ser)?;
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
            let mut ser = serialize::SerializerWrap::new(Vec::new());
            msg.frame_body_crc(self.client.out_crc(), &mut ser)?;
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
