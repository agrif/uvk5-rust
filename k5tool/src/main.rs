use gumdrop::Options;

trait ToolRun {
    fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Options, Debug)]
struct ToolOptions {
    #[options(help = "print help message")]
    help: bool,

    #[options(command, required)]
    command: Option<ToolCommand>,
}

#[derive(Options, Debug)]
enum ToolCommand {
    Unpack(UnpackOpts),
    Pack(PackOpts),
    ParseDump(ParseDumpOpts),
}

impl ToolRun for ToolCommand {
    fn run(&self) -> anyhow::Result<()> {
        use ToolCommand::*;
        match self {
            Unpack(o) => o.run(),
            Pack(o) => o.run(),
            ParseDump(o) => o.run(),
        }
    }
}

#[derive(Options, Debug)]
struct UnpackOpts {
    #[options(free, required)]
    packed: String,

    #[options(free, required)]
    unpacked: String,
}

impl ToolRun for UnpackOpts {
    fn run(&self) -> anyhow::Result<()> {
        let packed = k5lib::PackedFirmware::new(std::fs::read(&self.packed)?)?;
        if !packed.check() {
            anyhow::bail!("CRC check failed, cannot unpack")
        }

        let (unpacked, version) = packed.unpack()?;
        if let Ok(s) = version.as_str() {
            println!("version: {}", s);
        } else {
            println!("version: {:?}", version.as_bytes());
        }

        std::fs::write(&self.unpacked, &unpacked[..])?;
        Ok(())
    }
}

#[derive(Options, Debug)]
struct PackOpts {
    #[options(free, required)]
    version: String,

    #[options(free, required)]
    unpacked: String,

    #[options(free, required)]
    packed: String,
}

impl ToolRun for PackOpts {
    fn run(&self) -> anyhow::Result<()> {
        let version = k5lib::Version::from_str(&self.version)?;
        let unpacked = k5lib::UnpackedFirmware::new(std::fs::read(&self.unpacked)?);
        let packed = unpacked.pack(version)?;

        std::fs::write(&self.packed, &packed[..])?;
        Ok(())
    }
}

#[derive(Options, Debug)]
struct ParseDumpOpts {
    #[options(free, required)]
    dump: String,
}

impl ToolRun for ParseDumpOpts {
    fn run(&self) -> anyhow::Result<()> {
        let rawdata = std::fs::read(&self.dump)?;
        let mut raw = &rawdata[..];

        let xmodem = k5lib::protocol::CrcXModem::new();
        let dummy = k5lib::protocol::CrcConstant(0xffff);

        loop {
            if raw.len() < 3 {
                break;
            }
            let dir = raw[0];
            let len = ((raw[1] as u16) | ((raw[2] as u16) << 8)) as usize;
            let frameraw = &raw[3..3 + len];
            raw = &raw[3 + len..];

            let crc = if dir == 0 {
                println!("radio -> computer, {} bytes", len);
                k5lib::protocol::CrcEither::Left(&dummy)
            } else {
                println!("computer -> radio, {} bytes", len);
                k5lib::protocol::CrcEither::Right(&xmodem)
            };

            println!();

            match parse_frame(crc, frameraw) {
                Ok(o) => {
                    println!("{:?}", o);
                    println!();
                }
                Err(e) => {
                    println!("Unparsed frame:");
                    hexdump::hexdump(frameraw);
                    println!();
                    anyhow::bail!(e);
                }
            }
        }
        Ok(())
    }
}

fn parse_frame<C>(crc: C, data: &[u8]) -> anyhow::Result<k5lib::protocol::Message>
where
    C: k5lib::protocol::CrcStyle,
{
    let (rest, frame) = k5lib::protocol::framed(crc, nom::combinator::rest)(data)
        .map_err(|_| anyhow::anyhow!("Frame parser failed."))?;
    anyhow::ensure!(rest.len() == 0, "Frame parser left leftover data.");

    use k5lib::protocol::FramedResult;
    match frame {
        FramedResult::Ok(framebody) => match parse_message(framebody.clone()) {
            Ok(o) => Ok(o),
            Err(e) => {
                println!("Deobfuscated frame:");
                hexdump::hexdump(framebody.to_vec().as_ref());
                println!();
                Err(e)
            }
        },
        FramedResult::ParseErr(_f, e) => anyhow::bail!("Frame parse error: {:?}", e.code),
        FramedResult::CrcErr(f) => {
            println!("Deobfuscated frame + CRC:");
            hexdump::hexdump(f.to_vec().as_ref());
            println!();
            anyhow::bail!("CRC error.");
        }
        FramedResult::None => anyhow::bail!("Frame parser found no frames."),
    }
}

fn parse_message(
    data: k5lib::protocol::Deobfuscated<&[u8]>,
) -> anyhow::Result<k5lib::protocol::Message> {
    let (rest, (typ, body)) = k5lib::protocol::message(|t| {
        nom::combinator::map(nom::combinator::rest, move |r| (t, r))
    })(data)
    .map_err(|_| anyhow::anyhow!("Message parser falied."))?;
    anyhow::ensure!(rest.len() == 0, "Message parser left leftover data.");

    println!("Message type: {:x?}", typ);

    match parse_message_body(typ, body.clone()) {
        Ok(o) => Ok(o),
        Err(e) => {
            println!("Unparsed message body:");
            hexdump::hexdump(body.to_vec().as_ref());
            println!();
            anyhow::bail!(e);
        }
    }
}

fn parse_message_body(
    typ: u16,
    body: k5lib::protocol::Deobfuscated<&[u8]>,
) -> anyhow::Result<k5lib::protocol::Message> {
    use k5lib::protocol::MessageParse;
    use nom::Parser;

    let (rest, msg) = k5lib::protocol::Message::parse_body(typ)
        .parse(body)
        .map_err(|_| anyhow::anyhow!("Message body parser falied."))?;
    anyhow::ensure!(rest.len() == 0, "Message body parser left leftover data.");
    Ok(msg)
}

fn main() -> anyhow::Result<()> {
    let opts = ToolOptions::parse_args_default_or_exit();
    if let Some(subcommand) = opts.command {
        subcommand.run()
    } else {
        anyhow::bail!("subcommand not provided");
    }
}
