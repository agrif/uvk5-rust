#[derive(clap::Args, Debug)]
pub struct UnpackOpts {
    firmware: String,
    unpacked: String,

    #[arg(long, value_enum, default_value = "auto")]
    format: crate::binformat::BinaryFormat,
}

impl crate::ToolRun for UnpackOpts {
    fn run(&self) -> anyhow::Result<()> {
        let (unpacked, version) =
            crate::binformat::read_firmware(&self.firmware, self.format, None)?;

        if let Some(version) = version {
            if let Ok(s) = version.as_str() {
                println!("version: {}", s);
            } else {
                println!("version: {:?}", version.as_bytes());
            }
        }

        std::fs::write(&self.unpacked, &unpacked[..])?;
        Ok(())
    }
}
