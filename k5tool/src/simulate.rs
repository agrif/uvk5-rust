use std::io::{Read, Write};

use k5lib::protocol;
use k5lib::protocol::{HostMessage, ParseResult};

#[derive(clap::Args, Debug)]
pub struct SimulateOpts {
    #[command(flatten)]
    port: crate::common::SerialPortArgs,
    #[command(flatten)]
    debug: crate::debug::DebugClientArgs,
    #[arg(long)]
    initial_eeprom: Option<String>,
    #[arg(long, default_value_t = 0x2000)]
    empty_eeprom_size: usize,
}

impl crate::ToolRun for SimulateOpts {
    fn run(&self) -> anyhow::Result<()> {
        let eeprom = if let Some(ref initial_eeprom_path) = self.initial_eeprom {
            std::fs::read(initial_eeprom_path)?
        } else {
            // FIXME magic eeprom size
            vec![0; self.empty_eeprom_size]
        };

        let client = k5lib::ClientRadio::new(self.port.open()?);
        let client = self.debug.wrap(client);
        Simulator::new(client, eeprom).simulate()
    }
}

struct Simulator<F> {
    client: crate::debug::DebugClientRadio<F>,
    timestamp: u32,

    eeprom: Vec<u8>,
}

impl<F> Simulator<F>
where
    F: Read + Write,
{
    fn new(client: crate::debug::DebugClientRadio<F>, eeprom: Vec<u8>) -> Self {
        Self {
            client,
            timestamp: 0,
            eeprom,
        }
    }

    fn simulate(&mut self) -> anyhow::Result<()>
    where
        F: Read + Write,
    {
        loop {
            // try to parse a message
            let res = self.client.read_host()?;
            if let ParseResult::Ok(msg) = res {
                self.handle_message(msg)?;
            }
        }
    }

    fn handle_message(&mut self, msg: HostMessage) -> anyhow::Result<()> {
        match msg {
            HostMessage::Hello(m) => {
                self.timestamp = m.timestamp;
                self.client.write(&protocol::HelloReply {
                    version: k5lib::Version::from_str("k5sim")?,
                    has_custom_aes_key: false,
                    is_in_lock_screen: false,
                    padding: [0; 2],
                    challenge: [0; 4],
                })?;
            }
            HostMessage::ReadEeprom(m) => {
                if m.timestamp == self.timestamp {
                    // sleep a bit, eeprom reads are slow
                    std::thread::sleep(std::time::Duration::from_millis(100));

                    let mut start = m.address as usize;
                    let mut end = start + m.len as usize;
                    if start > self.eeprom.len() {
                        start = self.eeprom.len();
                    }
                    if end > self.eeprom.len() {
                        end = self.eeprom.len();
                    }

                    let data = &self.eeprom[start..end].to_owned();
                    self.client.write(&protocol::ReadEepromReply {
                        address: m.address,
                        len: data.len() as u8,
                        padding: 0,
                        data: &data[..],
                    })?;
                }
            }
        }
        Ok(())
    }
}
