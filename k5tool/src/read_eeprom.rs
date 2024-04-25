use std::io::{Read, Write};

use k5lib::protocol::{Hello, HelloReply, ParseResult, ReadEeprom, ReadEepromReply};

const TIMESTAMP: u32 = 0x6457396a;
const CHUNK_SIZE: u8 = 0x80;

// default, can be overridden
const EEPROM_SIZE: u64 = 0x2000;

#[derive(clap::Args, Debug)]
pub struct ReadEepromOpts {
    #[command(flatten)]
    port: crate::common::SerialPortArgs,
    #[arg(short, long)]
    output: Option<String>,
    #[arg(long)]
    raw: bool,
    #[arg(long, default_value_t = EEPROM_SIZE)]
    eeprom_size: u64,
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
        let timestamp = TIMESTAMP;
        let mut client = k5lib::ClientHost::<F>::new(port);

        client.write(&Hello { timestamp })?;
        let m = loop {
            if let ParseResult::Ok(m) = client.read::<HelloReply>()? {
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
                hexdump::hexdump(&data);
            }
        }

        eprintln!("Done.");

        Ok(())
    }

    fn read_eeprom<F, W>(
        &self,
        mut client: k5lib::ClientHost<F>,
        mut output: W,
    ) -> anyhow::Result<()>
    where
        F: Read + Write,
        W: Write,
    {
        let bar = crate::common::download_bar(self.eeprom_size);
        bar.set_position(0);

        let timestamp = TIMESTAMP;
        let mut address = 0;
        loop {
            client.write(&ReadEeprom {
                address,
                len: CHUNK_SIZE,
                padding: 0,
                timestamp,
            })?;
            let m = loop {
                if let ParseResult::Ok(m) = client.read::<ReadEepromReply<_>>()? {
                    break m;
                }
            };

            if address != m.address {
                anyhow::bail!("Reply had different address!");
            }

            for b in m.data.iter() {
                output.write_all(&[b])?;
            }

            address += m.len as u16;
            bar.set_position(address as u64);
            if m.len < CHUNK_SIZE || address as u64 >= self.eeprom_size {
                break;
            }
        }

        Ok(())
    }
}
