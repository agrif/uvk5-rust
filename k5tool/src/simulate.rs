use std::io::{Read, Write};

use k5lib::protocol;
use k5lib::protocol::{HostMessage, ParseResult};

#[derive(clap::Args, Debug)]
pub struct SimulateOpts {
    #[arg(default_value = "localhost:8855")]
    bind: String,
    #[command(flatten)]
    debug: crate::debug::DebugClientArgs,
    #[arg(long, default_value = "k5sim")]
    version: String,
    #[arg(long)]
    initial_eeprom: Option<String>,
    #[arg(long, default_value_t = 0x2000)]
    empty_eeprom_size: usize,
}

impl crate::ToolRun for SimulateOpts {
    fn run(&self) -> anyhow::Result<()> {
        let mut eeprom = if let Some(ref initial_eeprom_path) = self.initial_eeprom {
            std::fs::read(initial_eeprom_path)?
        } else {
            // FIXME magic eeprom size
            vec![0; self.empty_eeprom_size]
        };

        let listener = std::net::TcpListener::bind(&self.bind)?;
        eprintln!("Listening on {}.", self.bind);

        loop {
            let (stream, addr) = listener.accept()?;
            eprintln!("Connected to {}.", addr);

            // use a low timeout, so we can send bootloader ready messages
            // (if we need to)
            stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;

            let client = k5lib::ClientRadio::new(stream);
            let client = self.debug.wrap(client);
            match Simulator::new(client, self, &mut eeprom).simulate() {
                Err(e) => match e.downcast_ref::<std::io::Error>().map(|e| e.kind()) {
                    // an expected error, at disconnect
                    Some(std::io::ErrorKind::UnexpectedEof) => {
                        eprintln!("Disconnected from {}.", addr);
                        continue;
                    }
                    // any other error is unexpected
                    _ => anyhow::bail!(e),
                },
                // statically impossible, but ! not stable
                _ => {}
            }
        }
    }
}

struct Simulator<'a, F> {
    client: crate::debug::DebugClientRadio<F>,
    timestamp: u32,

    opts: &'a SimulateOpts,
    eeprom: &'a mut [u8],
}

impl<'a, F> Simulator<'a, F>
where
    F: Read + Write,
{
    fn new(
        client: crate::debug::DebugClientRadio<F>,
        opts: &'a SimulateOpts,
        eeprom: &'a mut [u8],
    ) -> Self {
        Self {
            client,
            timestamp: 0,
            opts,
            eeprom,
        }
    }

    fn simulate(&mut self) -> anyhow::Result<()>
    where
        F: Read + Write,
    {
        loop {
            // try to parse a message
            match self.client.read_host() {
                Ok(ParseResult::Ok(msg)) => {
                    self.handle_message(msg)?;
                    continue;
                }
                Err(e) => {
                    if let std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock = e.kind()
                    {
                        // try again if timed out
                        continue;
                    } else {
                        // any other error means stop the loop
                        anyhow::bail!(e);
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_message(&mut self, msg: HostMessage) -> anyhow::Result<()> {
        match msg {
            HostMessage::Hello(m) => {
                self.timestamp = m.timestamp;
                self.client.write(&protocol::HelloReply {
                    version: k5lib::Version::from_str(&self.opts.version)?,
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
