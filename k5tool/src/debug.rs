use k5lib::protocol::crc;
use k5lib::protocol::obfuscation;
use k5lib::protocol::{
    parse, serialize, HostMessage, Message, MessageParse, MessageSerialize, ParseResult,
    RadioMessage, FRAME_END, FRAME_START,
};
use k5lib::ClientBuffer;

#[derive(clap::Args, Debug, Clone)]
pub struct DebugClientArgs {
    #[arg(short, long, action=clap::ArgAction::Count)]
    debug: u8,
}

pub struct DebugClient<F, InC, OutC> {
    client: k5lib::Client<F, k5lib::ArrayBuffer, InC, OutC>,
    args: DebugClientArgs,
}

pub type DebugClientHost<F> = DebugClient<F, crc::CrcConstantIgnore, crc::CrcXModem>;
pub type DebugClientRadio<F> = DebugClient<F, crc::CrcXModem, crc::CrcConstantIgnore>;

impl DebugClientArgs {
    pub fn wrap<F, InC, OutC>(
        &self,
        client: k5lib::Client<F, k5lib::ArrayBuffer, InC, OutC>,
    ) -> DebugClient<F, InC, OutC> {
        DebugClient {
            client,
            args: self.clone(),
        }
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
        match res {
            ParseResult::Ok(ref m) => {
                if self.args.debug >= 2 {
                    let data = buf.data();
                    // we know this data produced a frame, we just have to
                    // find it. This involves some assumptions, which we check
                    if let (rest, Some(frame)) = parse::frame_raw(data) {
                        if self.args.debug >= 3 {
                            let end = data.len() - rest.len();
                            let body_end = end - FRAME_END.len();
                            assert_eq!(data[body_end..end], FRAME_END);
                            let body_start = body_end - frame.len();
                            let len_start = body_start - 2; // 2 byte length
                            let start = len_start - FRAME_START.len();
                            assert_eq!(data[start..len_start], FRAME_START);

                            let raw = &data[start..end];

                            eprintln!("<<< raw frame:");
                            crate::common::e_hexdump("<<<   ", raw);
                        }

                        let deob = obfuscation::Deobfuscated::new(frame);
                        eprintln!("<<< deobfuscated:");
                        crate::common::e_hexdump("<<<   ", &deob.to_vec());
                    }
                }
                if self.args.debug >= 1 {
                    eprintln!("<<< {:?}", m);
                }
            }
            ParseResult::ParseErr(ref inp, ref e) => {
                if self.args.debug >= 1 {
                    eprintln!("!!! parse error: {:?}", e);
                    crate::common::e_hexdump("!!!   ", inp.to_vec().as_ref());
                }
            }
            ParseResult::CrcErr(ref inp) => {
                if self.args.debug >= 1 {
                    eprintln!("!!! crc error:");
                    crate::common::e_hexdump("!!!   ", inp.to_vec().as_ref());
                }
            }
            ParseResult::None => {}
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
        if self.args.debug >= 3 {
            let mut ser = serialize::SerializerWrap::new(Vec::new());
            msg.frame(self.client.out_crc(), &mut ser)?;
            eprintln!(">>> raw frame:");
            crate::common::e_hexdump(">>>   ", &ser.done());
        }
        if self.args.debug >= 2 {
            let mut ser = serialize::SerializerWrap::new(Vec::new());
            msg.frame_body_crc(self.client.out_crc(), &mut ser)?;
            eprintln!(">>> deobfuscated:");
            crate::common::e_hexdump(">>>   ", &ser.done());
        }
        if self.args.debug >= 1 {
            eprintln!(">>> {:?}", msg);
        }
        self.client.write(msg)
    }
}
