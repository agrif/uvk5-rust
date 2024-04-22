#[derive(clap::Args, Debug)]
pub struct UnpackOpts {
    packed: String,
    unpacked: String,
}

impl crate::ToolRun for UnpackOpts {
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
