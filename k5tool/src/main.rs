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
        let packed = k5tool::PackedFirmware::new(std::fs::read(&self.packed)?)?;
        if !packed.check() {
            anyhow::bail!("CRC check failed, cannot unpack")
        }

        let (unpacked, version) = packed.unpack()?;
        if let Ok(s) = version.as_str() {
            println!("version: {}", s);
        } else {
            println!("version: {:?}", &version[..]);
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
        let version = k5tool::Version::from_str(&self.version)?;
        let unpacked = k5tool::UnpackedFirmware::new(std::fs::read(&self.unpacked)?);
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

        let xmodem = k5tool::protocol::CrcXModem::new();
        let dummy = k5tool::protocol::CrcConstant(0xffff);

        loop {
            if raw.len() < 3 {
                break;
            }
            let dir = raw[0];
            let len = ((raw[1] as u16) | ((raw[2] as u16) << 8)) as usize;
            let frameraw = &raw[3..3 + len];
            raw = &raw[3 + len..];

            use k5tool::protocol::Deobfuscated;
            let allparse = nom::combinator::rest::<
                Deobfuscated<&[u8]>,
                nom::error::Error<Deobfuscated<&[u8]>>,
            >;

            let frame = if dir == 0 {
                println!("radio -> computer, {} bytes", len);
                let (rest, frame) = k5tool::protocol::framed(&dummy, allparse)(frameraw)
                    .map_err(|_| anyhow::anyhow!("frame parser failed"))?;
                anyhow::ensure!(rest.len() == 0, "leftover data after parse!");
                frame
            } else {
                println!("computer -> radio, {} bytes", len);
                let (rest, frame) = k5tool::protocol::framed(&xmodem, allparse)(frameraw)
                    .map_err(|_| anyhow::anyhow!("frame parser failed"))?;
                anyhow::ensure!(rest.len() == 0, "leftover data after parse!");
                frame
            };

            use k5tool::protocol::FramedResult;
            match frame {
                FramedResult::Ok(o) => {
                    println!();
                    hexdump::hexdump(o.to_vec().as_ref());
                },
                FramedResult::ParseErr(_f, e) => println!("Parse error!!! {:?}", e),
                FramedResult::CrcErr(_f) => println!("CRC error!!!"),
                FramedResult::None => println!("Ate some input."),
            }

            println!();
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let opts = ToolOptions::parse_args_default_or_exit();
    if let Some(subcommand) = opts.command {
        subcommand.run()
    } else {
        anyhow::bail!("subcommand not provided");
    }
}
