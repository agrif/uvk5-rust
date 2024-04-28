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
    #[arg(short, long)]
    boot: bool,

    #[arg(long)]
    initial_eeprom: Option<String>,
    #[arg(long)]
    dump_eeprom: Option<String>,
    #[arg(long, default_value_t = 0x2000)]
    eeprom_size: usize,

    #[arg(long)]
    initial_flash: Option<String>,
    #[arg(long)]
    dump_flash: Option<String>,
    #[arg(long, default_value_t = 0xf000)]
    flash_size: usize,
}

impl crate::ToolRun for SimulateOpts {
    fn run(&self) -> anyhow::Result<()> {
        let mut eeprom = if let Some(ref initial_eeprom_path) = self.initial_eeprom {
            std::fs::read(initial_eeprom_path)?
        } else {
            vec![0; self.eeprom_size]
        };

        eeprom.truncate(self.eeprom_size);
        eeprom.resize(self.eeprom_size, 0);

        let mut flash = if let Some(ref initial_flash_path) = self.initial_flash {
            std::fs::read(initial_flash_path)?
        } else {
            vec![0; self.flash_size]
        };

        flash.truncate(self.flash_size);
        flash.resize(self.flash_size, 0);

        let listener = std::net::TcpListener::bind(&self.bind)?;
        println!("Listening on {}.", self.bind);

        loop {
            let (stream, addr) = listener.accept()?;
            println!("Connected to {}.", addr);

            // use a low timeout, so we can send bootloader ready messages
            // (if we need to)
            stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;

            let client = k5lib::ClientRadio::new(stream);
            let client = self.debug.wrap_radio(client)?;
            match Simulator::new(client, self, &mut eeprom, &mut flash).simulate() {
                Err(e) => match e.downcast_ref::<std::io::Error>().map(|e| e.kind()) {
                    // an expected error, at disconnect
                    Some(std::io::ErrorKind::UnexpectedEof) => {
                        println!("Disconnected from {}.", addr);

                        if let Some(ref eeprom_path) = self.dump_eeprom {
                            std::fs::write(eeprom_path, &eeprom)?;
                        }

                        if let Some(ref flash_path) = self.dump_flash {
                            std::fs::write(flash_path, &flash)?;
                        }

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
    flash: &'a mut [u8],

    firmware_version: Option<k5lib::Version>,
    flash_in_progress: bool,
    flash_page: u16,
    flash_max_page: u16,
    flash_session_id: u32,
}

impl<'a, F> Simulator<'a, F>
where
    F: Read + Write,
{
    fn new(
        client: crate::debug::DebugClientRadio<F>,
        opts: &'a SimulateOpts,
        eeprom: &'a mut [u8],
        flash: &'a mut [u8],
    ) -> Self {
        Self {
            client,
            timestamp: 0,

            opts,
            eeprom,
            flash,

            firmware_version: None,
            flash_in_progress: false,
            flash_page: 0,
            flash_max_page: 0,
            flash_session_id: 0,
        }
    }

    fn send_boot_ready(&mut self) -> anyhow::Result<()> {
        self.client.write(&protocol::BootloaderReady {
            chip_id: [0x01234567, 0x89abcdef, 0xfedcba98, 0x76543210],
            version: k5lib::Version::from_str(&self.opts.version)?,
        })?;
        Ok(())
    }

    fn send_write_flash_error(&mut self) -> anyhow::Result<()> {
        self.client.write(&protocol::WriteFlashReply {
            session_id: 0,
            page: 0,
            error: 1,
        })?;
        Ok(())
    }

    fn simulate(&mut self) -> anyhow::Result<()> {
        loop {
            // emit a boot ready if we're in the boot_ready_loop
            if self.opts.boot && self.firmware_version.is_none() {
                self.send_boot_ready()?;
            }

            // try to parse a message
            match self.client.read_host() {
                Ok(ParseResult::Ok(msg)) => {
                    // FIXME can't write to client while borrowing. Hm.
                    let msg = msg.map(|d| d.to_vec());
                    if self.opts.boot {
                        self.handle_boot_message(msg)?;
                    } else {
                        self.handle_message(msg)?;
                    }
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

    fn handle_message(&mut self, msg: HostMessage<Vec<u8>>) -> anyhow::Result<()> {
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

            _ => {}
        }
        Ok(())
    }

    fn handle_boot_message(&mut self, msg: HostMessage<Vec<u8>>) -> anyhow::Result<()> {
        match msg {
            HostMessage::BootloaderReadyReply(m) => {
                // real bootloaders check version against their own
                // checking the first char matches, or m.version starts with *
                // we'll just... not... do that.
                self.firmware_version = Some(m.version);

                // bootloader continues to send these until we get a write
                // so, let's send at least one
                self.send_boot_ready()?;
            }

            HostMessage::WriteFlash(m) => {
                // make sure we've received a version
                if self.firmware_version.is_some() {
                    if m.page == 0 {
                        // erase flash and set up state on first page
                        self.flash_page = m.page;
                        self.flash_max_page = m.max_page;
                        self.flash_session_id = m.session_id;
                        self.flash_in_progress = true;
                        self.flash.fill(0);
                    } else {
                        // bail if not in progress
                        if !self.flash_in_progress {
                            return Ok(());
                        }

                        // check this is the same session and the next page
                        if m.page != self.flash_page + 1 || m.session_id != self.flash_session_id {
                            self.send_write_flash_error()?;
                            self.flash_in_progress = false;
                            return Ok(());
                        }
                    }

                    // copy in
                    let addr = m.page as usize * protocol::WRITE_FLASH_LEN;
                    for (i, b) in m.data.iter().enumerate() {
                        self.flash[addr + i] = *b;
                    }

                    // write reply
                    self.client.write(&protocol::WriteFlashReply {
                        session_id: self.flash_session_id,
                        page: m.page,
                        error: 0,
                    })?;

                    // update state
                    self.flash_page = m.page;

                    // is this the last page?
                    if m.page + 1 == self.flash_max_page {
                        // normally, the bootloader runs the new program now
                        self.flash_in_progress = false;
                        self.firmware_version = None;
                    }
                } else {
                    self.send_write_flash_error()?;
                }
            }

            _ => {}
        }

        Ok(())
    }
}
