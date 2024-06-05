use std::io::{Read, Write};

use rustyline::{error::ReadlineError, DefaultEditor, ExternalPrinter};

use k5lib::protocol::messages::custom::DebugInput;
use k5lib::protocol::messages::{Message, RadioMessage};
use k5lib::protocol::ParseResult;

#[derive(clap::Args, Debug)]
pub struct ConsoleOpts {
    #[command(flatten)]
    port: crate::common::SerialPortArgs,
    #[command(flatten)]
    debug: crate::debug::DebugClientArgs,

    #[arg(short, long)]
    elf: Option<String>,
}

impl crate::ToolRun for ConsoleOpts {
    fn run(&self) -> anyhow::Result<()> {
        let port = self.port.open()?;
        let mut client = self.debug.wrap_host(k5lib::ClientHost::new_std(port))?;
        let mut console = Console::new(&mut client, self.elf.as_ref().map(|s| s.as_str()));
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

struct DefmtDecode {
    proc: std::process::Child,
    stdin: std::process::ChildStdin,
    read_thread: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
}

impl DefmtDecode {
    fn new<P>(elf: &str, mut printer: P) -> anyhow::Result<Self>
    where
        P: ExternalPrinter + Send + 'static,
    {
        let mut proc = std::process::Command::new("defmt-print")
            .args(["-e", elf, "stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let stdout = proc.stdout.take().unwrap();
        let read_thread = std::thread::spawn(move || {
            let mut buffered = std::io::BufReader::new(stdout);
            let mut line = String::new();
            loop {
                use std::io::BufRead;
                line.clear();
                if buffered.read_line(&mut line)? == 0 {
                    // eof
                    break;
                }
                printer.print(line.clone())?;
            }

            Ok(())
        });

        Ok(Self {
            stdin: proc.stdin.take().unwrap(),
            proc,
            read_thread: Some(read_thread),
        })
    }

    fn decode(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.stdin.write_all(data)?;
        Ok(())
    }
}

impl Drop for DefmtDecode {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
        if let Some(read_thread) = self.read_thread.take() {
            let _ = read_thread.join();
        }
    }
}

pub struct Console<'a, F> {
    client: &'a mut crate::debug::DebugClientHost<F>,
    elf: Option<&'a str>,
}

impl<'a, F> Console<'a, F>
where
    F: Read + Write + Send + ConsoleTryClone,
{
    pub fn new(client: &'a mut crate::debug::DebugClientHost<F>, elf: Option<&'a str>) -> Self {
        Self { client, elf }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut rl = DefaultEditor::new()?;

        let printer = rl.create_external_printer()?;
        let mut defmt = self
            .elf
            .and_then(|elf| match DefmtDecode::new(elf, printer) {
                Ok(decode) => Some(decode),
                Err(_) => {
                    eprintln!("Could not start `defmt-print`.");
                    eprintln!("`defmt` messages will be undecoded.");
                    None
                }
            });

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

                            match msg {
                                ParseResult::Ok(
                                    _,
                                    ref m @ Message::Radio(RadioMessage::DebugOutput(ref output)),
                                ) => {
                                    if output.defmt {
                                        if let Some(ref mut defmt) = defmt {
                                            defmt.decode(output.data)?;
                                        } else {
                                            printer.print(format!("{:?}\n", m))?;
                                        }
                                    } else {
                                        match std::str::from_utf8(output.data) {
                                            Ok(s) => printer.print(s.to_owned())?,
                                            Err(_) => {
                                                let dump = "\n".to_owned()
                                                    + &crate::hexdump::hexdump_format(output.data)
                                                    + "\n";
                                                printer.print(dump)?;
                                            }
                                        }
                                    }
                                }
                                ParseResult::Ok(_, ref m) => {
                                    printer.print(format!("{:?}\n", m))?;
                                }
                                ParseResult::ParseErr(_, _, ref e) => {
                                    printer.print(format!("!!! parse error: {:?}\n", e))?;
                                }
                                ParseResult::CrcErr(_, _) => {
                                    printer.print(format!("!!! crc error\n"))?;
                                }
                                ParseResult::None => {}
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
                                eprintln!("{}", e);
                                std::process::exit(0);
                            }
                        }
                    }
                }
            });

            loop {
                if reader.is_finished() {
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
