use k5lib::protocol::crc;
use k5lib::protocol::obfuscation;
use k5lib::protocol::{
    HostMessage, Message, MessageParse, MessageSerialize, ParseResult, RadioMessage,
};

#[derive(clap::Args, Debug, Clone)]
pub struct DebugClientArgs {
    #[arg(short, long, action=clap::ArgAction::Count)]
    debug: u8,
}

pub struct DebugClient<F, B, InC, OutC> {
    client: k5lib::Client<F, B, InC, OutC>,
    args: DebugClientArgs,
}

pub type DebugClientHost<F, B = k5lib::ArrayBuffer> =
    DebugClient<F, B, crc::CrcConstantIgnore, crc::CrcXModem>;
pub type DebugClientRadio<F, B = k5lib::ArrayBuffer> =
    DebugClient<F, B, crc::CrcXModem, crc::CrcConstantIgnore>;

impl DebugClientArgs {
    pub fn wrap<F, B, InC, OutC>(
        &self,
        client: k5lib::Client<F, B, InC, OutC>,
    ) -> DebugClient<F, B, InC, OutC> {
        DebugClient {
            client,
            args: self.clone(),
        }
    }
}

impl<F, B, InC, OutC> DebugClient<F, B, InC, OutC>
where
    B: k5lib::ClientBuffer,
    InC: crc::CrcStyle,
    OutC: crc::CrcStyle,
{
    pub fn read<'a, M>(&'a mut self) -> std::io::Result<ParseResult<B::Slice<'a>, M>>
    where
        M: MessageParse<obfuscation::Deobfuscated<B::Slice<'a>>> + std::fmt::Debug,
        F: std::io::Read,
        B::Slice<'a>: std::fmt::Debug,
    {
        let res = self.client.read()?;
        if self.args.debug >= 1 {
            match res {
                ParseResult::Ok(ref m) => eprintln!("<<< {:?}", m),
                ParseResult::ParseErr(ref inp, ref e) => {
                    eprintln!("!!! parse error: {:?}", e);
                    crate::common::e_hexdump(inp.to_vec().as_ref());
                }
                ParseResult::CrcErr(ref inp) => {
                    eprintln!("!!! crc error");
                    crate::common::e_hexdump(inp.to_vec().as_ref());
                }
                ParseResult::None => {}
            }
        }
        Ok(res)
    }

    /// Read a Message.
    pub fn read_any(
        &mut self,
    ) -> std::io::Result<ParseResult<B::Slice<'_>, Message<obfuscation::Deobfuscated<B::Slice<'_>>>>>
    where
        F: std::io::Read,
        for<'a> B::Slice<'a>: std::fmt::Debug,
    {
        self.read()
    }

    /// Read a HostMessage.
    pub fn read_host(&mut self) -> std::io::Result<ParseResult<B::Slice<'_>, HostMessage>>
    where
        F: std::io::Read,
        for<'a> B::Slice<'a>: std::fmt::Debug,
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
        for<'a> B::Slice<'a>: std::fmt::Debug,
    {
        self.read()
    }

    /// Write a message to the port.
    pub fn write<M>(&mut self, msg: &M) -> std::io::Result<()>
    where
        F: std::io::Write,
        M: MessageSerialize + std::fmt::Debug,
    {
        if self.args.debug >= 1 {
            eprintln!(">>> {:?}", msg);
        }
        self.client.write(msg)
    }
}
