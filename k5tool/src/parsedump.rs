#[derive(clap::Args, Debug)]
pub struct ParseDumpOpts {
    dump: String,
    #[command(flatten)]
    debug: crate::debug::DebugClientArgs,
}

impl crate::ToolRun for ParseDumpOpts {
    fn run(&self) -> anyhow::Result<()> {
        // command only makes sense *at all* for debug >= 1
        let mut debug = self.debug.clone();
        debug.debug = debug.debug.max(1);

        let rawdata = std::fs::read(&self.dump)?;
        let mut raw = &rawdata[..];

        loop {
            if raw.len() < 3 {
                break;
            }
            let dir = raw[0];
            let len = ((raw[1] as u16) | ((raw[2] as u16) << 8)) as usize;
            let frameraw = &raw[3..3 + len];
            raw = &raw[3 + len..];

            if dir == 0 {
                // radio -> computer, so act like host
                eprintln!("*** from radio");
                let mut host = debug.wrap(k5lib::ClientHost::new(frameraw));
                host.read_radio()?;
            } else {
                // computer -> radio, so act like radio
                eprintln!("*** from host");
                let mut radio = debug.wrap(k5lib::ClientRadio::new(frameraw));
                radio.read_host()?;
            }
        }
        Ok(())
    }
}
