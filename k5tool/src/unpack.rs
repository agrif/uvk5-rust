#[derive(clap::Args, Debug)]
pub struct UnpackOpts {
    firmware: String,
    unpacked: String,

    #[arg(long, value_enum, default_value = "auto")]
    format: crate::binformat::BinaryFormat,
}

impl crate::ToolRun for UnpackOpts {
    fn run(&self) -> anyhow::Result<()> {
        let (unpacked, info) = crate::binformat::read_firmware(&self.firmware, self.format, None)?;

        info.report();
        std::fs::write(&self.unpacked, &unpacked[..])?;
        Ok(())
    }
}
