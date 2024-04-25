use k5lib::protocol::crc;
use k5lib::protocol::obfuscation;
use k5lib::protocol::{
    HostMessage, Message, MessageParse, MessageSerialize, ParseResult, RadioMessage,
};

#[derive(clap::Args, Debug, Clone)]
pub struct SerialPortArgs {
    port: String,
    #[arg(short, long, default_value_t = k5lib::protocol::BAUD_RATE)]
    baud: u32,
    #[arg(long)]
    plain_file: bool,
}

#[derive(Debug)]
pub enum SerialPort {
    Serial(Box<dyn serialport::SerialPort>),
    File(std::fs::File),
}

impl std::io::Read for SerialPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Serial(port) => port.read(buf),
            Self::File(port) => port.read(buf),
        }
    }
}

impl std::io::Write for SerialPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Serial(port) => port.write(buf),
            Self::File(port) => port.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Serial(port) => port.flush(),
            Self::File(port) => port.flush(),
        }
    }
}

impl SerialPortArgs {
    pub fn open(&self) -> anyhow::Result<SerialPort> {
        if self.plain_file {
            let port = std::fs::File::options()
                .read(true)
                .write(true)
                .open(&self.port)?;

            Ok(SerialPort::File(port))
        } else {
            let mut port = serialport::new(&self.port, self.baud).open()?;
            port.set_timeout(std::time::Duration::from_secs(1))?;
            Ok(SerialPort::Serial(port))
        }
    }
}

pub fn download_bar(size: u64) -> indicatif::ProgressBar {
    let bar = indicatif::ProgressBar::new(size);
    bar.set_style(
        indicatif::ProgressStyle::with_template(
            "({spinner}) [{wide_bar}] ({percent:>3}%, {bytes_per_sec:>12})",
        )
        .unwrap()
        .progress_chars("=> ")
        .tick_strings(&["<<<  ", "<<  <", "<  <<", "  <<<", " <<< ", "-----"]),
    );
    bar
}

pub fn e_hexdump(bytes: &[u8]) {
    for s in hexdump::hexdump_iter(bytes) {
        eprintln!("{}", s);
    }
}

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
                    e_hexdump(inp.to_vec().as_ref());
                }
                ParseResult::CrcErr(ref inp) => {
                    eprintln!("!!! crc error");
                    e_hexdump(inp.to_vec().as_ref());
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
