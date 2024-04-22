#[derive(clap::Args, Debug)]
pub struct PackOpts {
    version: String,
    unpacked: String,
    packed: String,
}

impl crate::ToolRun for PackOpts {
    fn run(&self) -> anyhow::Result<()> {
        let version = k5lib::Version::from_str(&self.version)?;
        let unpacked = k5lib::pack::UnpackedFirmware::new(std::fs::read(&self.unpacked)?);
        let packed = unpacked.pack(version);

        std::fs::write(&self.packed, &packed[..])?;
        Ok(())
    }
}
