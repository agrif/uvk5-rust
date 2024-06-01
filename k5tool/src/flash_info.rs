#[derive(clap::Args, Debug)]
pub struct FlashInfoOpts {
    firmware: String,
    #[arg(long, value_enum, default_value = "auto")]
    format: crate::binformat::BinaryFormat,
}

impl crate::ToolRun for FlashInfoOpts {
    fn run(&self) -> anyhow::Result<()> {
        let (unpacked, info) = crate::binformat::read_firmware(&self.firmware, self.format, None)?;
        crate::flash_lint::check(&unpacked, &info, &[])?;
        Ok(())
    }
}
