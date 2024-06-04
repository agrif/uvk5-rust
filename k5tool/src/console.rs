use std::io::{Read, Write};

use rustyline::{error::ReadlineError, DefaultEditor, ExternalPrinter};

use k5lib::protocol::messages::custom::DebugInput;
use k5lib::protocol::messages::Message;
use k5lib::protocol::ParseResult;

#[derive(clap::Args, Debug)]
pub struct ConsoleOpts {
    #[command(flatten)]
    port: crate::common::SerialPortArgs,
    #[command(flatten)]
    debug: crate::debug::DebugClientArgs,
}

impl crate::ToolRun for ConsoleOpts {
    fn run(&self) -> anyhow::Result<()> {
        let port = self.port.open()?;
        let mut client = self.debug.wrap_host(k5lib::ClientHost::new_std(port))?;
        let mut console = Console::new(&mut client);
        console.run()
    }
}

pub trait ConsoleTryClone: Sized {
    fn try_clone(&self) -> Option<Self>;
}

impl ConsoleTryClone for Box<dyn serialport::SerialPort> {
    fn try_clone(&self) -> Option<Self> {
        serialport::SerialPort::try_clone(self.as_ref()).ok()
    }
}

impl ConsoleTryClone for std::net::TcpStream {
    fn try_clone(&self) -> Option<Self> {
        std::net::TcpStream::try_clone(self).ok()
    }
}

impl ConsoleTryClone for crate::common::SerialPort {
    fn try_clone(&self) -> Option<Self> {
        match self {
            Self::Serial(port) => ConsoleTryClone::try_clone(port.get_ref())
                .map(std::io::BufWriter::new)
                .map(Self::Serial),

            Self::Tcp(port) => ConsoleTryClone::try_clone(port.get_ref())
                .map(std::io::BufWriter::new)
                .map(Self::Tcp),
        }
    }
}

pub struct Console<'a, F> {
    client: &'a mut crate::debug::DebugClientHost<F>,
}

impl<'a, F> Console<'a, F>
where
    F: Read + Write + Send + ConsoleTryClone,
{
    pub fn new(client: &'a mut crate::debug::DebugClientHost<F>) -> Self {
        Self { client }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut rl = DefaultEditor::new()?;
        let mut printer = rl.create_external_printer()?;

        std::thread::scope(|s| {
            let writer = self
                .client
                .port()
                .try_clone()
                .ok_or_else(|| anyhow::anyhow!("could not get writer from port"))?;
            // we lose dump and debug facilities here, oh well
            let mut write_client = k5lib::ClientHostStd::<_, k5lib::ArrayBuffer>::new_std(writer);

            let reader = s.spawn(move || -> anyhow::Result<()> {
                loop {
                    match self.client.read_and_get_extra::<Message<&[u8]>>() {
                        Ok((msg, extra)) => {
                            if !extra.is_empty() {
                                if let Ok(s) = std::str::from_utf8(extra) {
                                    // FIXME linebuffer this, print adds \n
                                    printer.print(s.to_owned())?;
                                } else {
                                    let dump = "\n".to_owned()
                                        + &crate::hexdump::hexdump_format(extra)
                                        + "\n";
                                    printer.print(dump)?;
                                }
                            }

                            if !matches!(msg, ParseResult::None) {
                                let debug = "\n".to_owned() + &format!("{:?}", msg) + "\n";
                                printer.print(debug)?;
                            }
                        }

                        Err(e) => {
                            let timed_out = if let k5lib::ClientError::Io(ref io) = e {
                                matches!(
                                    io.kind(),
                                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock,
                                )
                            } else {
                                false
                            };

                            if timed_out {
                                // time-outs are ok
                                continue;
                            } else {
                                // other errors are not
                                anyhow::bail!(e);
                            }
                        }
                    }
                }
            });

            loop {
                if reader.is_finished() {
                    println!("read thread over.");
                    reader
                        .join()
                        .map_err(|_| anyhow::anyhow!("reader thread failed to start"))??;
                    panic!("reader thread finished but no error found");
                }

                match rl.readline("> ") {
                    Ok(line) => {
                        rl.add_history_entry(&line)?;
                        write_client.write(&DebugInput {
                            line: line.as_bytes(),
                        })?;
                    }

                    Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                        std::process::exit(0);
                    }

                    Err(e) => Err(e)?,
                };
            }
        })
    }
}
