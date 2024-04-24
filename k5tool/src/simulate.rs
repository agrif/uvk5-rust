use std::io::{Read, Write};

use k5lib::protocol;
use k5lib::protocol::{HostMessage, MessageSerialize, ParseResult};

#[derive(clap::Args, Debug)]
pub struct SimulateOpts {
    port: String,
    #[arg(short, long, default_value_t = protocol::BAUD_RATE)]
    baud: u32,
    #[arg(long)]
    plain_file: bool,
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

        if self.plain_file {
            let port = std::fs::File::options()
                .read(true)
                .write(true)
                .open(&self.port)?;

            Simulator::new(port, eeprom).simulate()
        } else {
            let port = serialport::new(&self.port, protocol::BAUD_RATE).open()?;
            Simulator::new(port, eeprom).simulate()
        }
    }
}

struct Simulator<F> {
    port: F,
    in_crc: protocol::crc::CrcXModem,
    out_crc: protocol::crc::CrcConstantIgnore,
    timestamp: u32,

    eeprom: Vec<u8>,
}

impl<F> Simulator<F>
where
    F: Read + Write,
{
    fn new(port: F, eeprom: Vec<u8>) -> Self {
        Self {
            port,
            in_crc: protocol::crc::CrcXModem::new(),
            out_crc: protocol::crc::CrcConstantIgnore(0xffff),
            timestamp: 0,
            eeprom,
        }
    }

    fn simulate(&mut self) -> anyhow::Result<()>
    where
        F: Read + Write,
    {
        let mut buffer = [0u8; protocol::MAX_FRAME_SIZE];
        let mut avail = 0;
        loop {
            let amt = self.port.read(&mut buffer[avail..])?;
            avail += amt;

            // try to parse a message
            let (rest, res) = protocol::parse(&self.in_crc, &buffer[..avail]);
            match res {
                ParseResult::Ok(msg) => self.handle_message(msg)?,
                ParseResult::ParseErr(inp, e) => {
                    println!("!!! parse error: {:?}", e);
                    hexdump::hexdump(inp.to_vec().as_ref());
                }
                ParseResult::CrcErr(inp) => {
                    println!("!!! crc error");
                    hexdump::hexdump(inp.to_vec().as_ref());
                }
                ParseResult::None => {}
            }

            let leftover = rest.len();
            let new_start = avail - leftover;
            buffer.copy_within(new_start..avail, 0);
            avail = leftover;

            if avail == buffer.len() {
                // buffer overflow waiting for complete frame, dump it
                avail = 0;
            }
        }
    }

    fn handle_message(&mut self, msg: HostMessage) -> anyhow::Result<()> {
        println!("<<< {:?}", msg);
        match msg {
            HostMessage::Hello(m) => {
                self.timestamp = m.timestamp;
                self.reply(protocol::HelloReply {
                    version: k5lib::Version::from_str("k5sim")?,
                    has_custom_aes_key: false,
                    is_in_lock_screen: false,
                    padding: [0; 2],
                    challenge: [0; 4],
                })?;
            }
            HostMessage::ReadEeprom(m) => {
                if m.timestamp == self.timestamp {
                    let mut start = m.address as usize;
                    let mut end = start + m.len as usize;
                    if start > self.eeprom.len() {
                        start = self.eeprom.len();
                    }
                    if end > self.eeprom.len() {
                        end = self.eeprom.len();
                    }

                    let data = &self.eeprom[start..end].to_owned();
                    self.reply(protocol::ReadEepromReply {
                        address: m.address,
                        len: data.len() as u8,
                        padding: 0,
                        data: &data[..],
                    })?;
                } else {
                    println!("!!! bad timestamp, ignoring")
                }
            }
        }
        Ok(())
    }

    fn reply<M>(&mut self, msg: M) -> anyhow::Result<()>
    where
        M: MessageSerialize + std::fmt::Debug,
    {
        println!(">>> {:?}", msg);
        protocol::serialize(&self.out_crc, &mut self.port, &msg)?;
        Ok(())
    }
}
