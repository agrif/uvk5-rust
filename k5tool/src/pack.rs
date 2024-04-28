#[derive(clap::Args, Debug)]
pub struct PackOpts {
    firmware: String,
    packed: String,

    #[arg(long)]
    version: Option<String>,
    // raw is a tempting default, but running it with no arguments on an elf
    // file will silently produce garbage. if the user wants raw, they can say
    #[arg(long, value_enum, default_value = "elf")]
    format: crate::binformat::BinaryFormat,
}

impl crate::ToolRun for PackOpts {
    fn run(&self) -> anyhow::Result<()> {
        let version = if let Some(ref v) = self.version {
            Some(k5lib::Version::from_str(v)?)
        } else {
            None
        };

        let (unpacked, version) =
            crate::binformat::read_firmware(&self.firmware, self.format, version)?;

        let version = version.ok_or(anyhow::anyhow!(
            "image has no version, use --version to provide one"
        ))?;
        let packed = unpacked.pack(version);

        std::fs::write(&self.packed, &packed[..])?;
        Ok(())
    }
}
