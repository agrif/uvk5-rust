use std::io::{Read, Write};

use k5lib::protocol::{Hello, HelloReply, ReadEeprom, ReadEepromReply, HELLO_SESSION_ID};

const CHUNK_SIZE: u8 = 0x80;

#[derive(clap::Args, Debug)]
pub struct ReadEepromOpts {
    #[command(flatten)]
    port: crate::common::SerialPortArgs,
    #[command(flatten)]
    debug: crate::debug::DebugClientArgs,
    #[arg(short, long)]
    output: Option<String>,
    #[arg(long)]
    raw: bool,
    #[arg(long, default_value_t = crate::common::EEPROM_MAX)]
    eeprom_size: usize,
}

impl crate::ToolRun for ReadEepromOpts {
    fn run(&self) -> anyhow::Result<()> {
        self.send_hello(self.port.open()?)
    }
}

impl ReadEepromOpts {
    fn send_hello<F>(&self, port: F) -> anyhow::Result<()>
    where
        F: Read + Write,
    {
        let session_id = HELLO_SESSION_ID;
        let mut client = self.debug.wrap_host(k5lib::ClientHost::<F>::new(port))?;

        client.write(&Hello { session_id })?;
        let m = loop {
            if let Some(m) = client.read::<HelloReply>()?.ok() {
                break m;
            }
        };

        if let Ok(ver) = m.version.as_str() {
            eprintln!("Connected to version: {}", ver);
        } else {
            eprintln!("Connected to version: {:x?}", m.version.as_bytes());
        }

        if let Some(ref path) = self.output {
            let output = std::io::BufWriter::new(std::fs::File::create(path)?);
            self.read_eeprom(client, output)?;
        } else {
            let mut output = std::io::BufWriter::new(Vec::new());
            self.read_eeprom(client, &mut output)?;
            let data = output.into_inner()?;
            if self.raw {
                std::io::stdout().write_all(&data)?;
            } else {
                crate::hexdump::hexdump(&data);
            }
        }

        Ok(())
    }

    fn read_eeprom<F, W>(
        &self,
        mut client: crate::debug::DebugClientHost<F>,
        mut output: W,
    ) -> anyhow::Result<()>
    where
        F: Read + Write,
        W: Write,
    {
        let bar = crate::common::download_bar(self.eeprom_size as u64);
        bar.set_position(0);

        let session_id = HELLO_SESSION_ID;
        let mut address = 0;
        loop {
            client.write(&ReadEeprom {
                address,
                len: CHUNK_SIZE,
                _pad: Default::default(),
                session_id,
            })?;
            let m = loop {
                if let Some(m) = client.read::<ReadEepromReply<_>>()?.ok() {
                    break m;
                }
            };

            if address != m.address {
                anyhow::bail!("Reply had different address!");
            }

            output.write_all(m.data)?;

            address += m.len as u16;
            bar.set_position(address as u64);
            if m.len < CHUNK_SIZE || address as usize >= self.eeprom_size {
                break;
            }
        }

        bar.finish();

        Ok(())
    }
}
