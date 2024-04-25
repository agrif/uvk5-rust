pub mod common;

mod pack;
mod parsedump;
mod read_eeprom;
mod simulate;
mod unpack;

trait ToolRun {
    fn run(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct ToolOptions {
    #[command(subcommand)]
    command: ToolCommand,
}

#[derive(clap::Subcommand, Debug)]
enum ToolCommand {
    Pack(pack::PackOpts),
    ParseDump(parsedump::ParseDumpOpts),
    ReadEeprom(read_eeprom::ReadEepromOpts),
    Simulate(simulate::SimulateOpts),
    Unpack(unpack::UnpackOpts),
}

impl ToolRun for ToolCommand {
    fn run(&self) -> anyhow::Result<()> {
        use ToolCommand::*;
        match self {
            Pack(o) => o.run(),
            ParseDump(o) => o.run(),
            ReadEeprom(o) => o.run(),
            Simulate(o) => o.run(),
            Unpack(o) => o.run(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    use clap::Parser;
    let opts = ToolOptions::parse();

    opts.command.run()
}
