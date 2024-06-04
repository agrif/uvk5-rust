use std::io::{Read, Write};

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

pub struct Console<'a, F> {
    client: &'a mut crate::debug::DebugClientHost<F>,
}

impl<'a, F> Console<'a, F>
where
    F: Read + Write,
{
    pub fn new(client: &'a mut crate::debug::DebugClientHost<F>) -> Self {
        Self { client }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let (msg, extra) = self.client.read_and_get_extra::<Message<&[u8]>>()?;
            if !extra.is_empty() {
                if let Ok(s) = std::str::from_utf8(extra) {
                    print!("{}", s);
                } else {
                    println!();
                    crate::hexdump::hexdump(extra);
                    println!();
                }
            }

            if !matches!(msg, ParseResult::None) {
                println!();
                println!("{:?}", msg);
                println!();
            }
        }
    }
}
