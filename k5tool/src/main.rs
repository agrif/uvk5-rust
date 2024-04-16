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
}

impl ToolRun for ToolCommand {
    fn run(&self) -> anyhow::Result<()> {
        use ToolCommand::*;
        match self {
            Unpack(o) => o.run(),
            Pack(o) => o.run(),
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

fn main() -> anyhow::Result<()> {
    let opts = ToolOptions::parse_args_default_or_exit();
    if let Some(subcommand) = opts.command {
        subcommand.run()
    } else {
        anyhow::bail!("subcommand not provided");
    }
}
