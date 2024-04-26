pub mod common;
pub mod debug;

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
    ListPorts(ListPortsOpts),
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
            ListPorts(o) => o.run(),
            Pack(o) => o.run(),
            ParseDump(o) => o.run(),
            ReadEeprom(o) => o.run(),
            Simulate(o) => o.run(),
            Unpack(o) => o.run(),
        }
    }
}

#[derive(clap::Args, Debug)]
pub struct ListPortsOpts;

impl crate::ToolRun for ListPortsOpts {
    fn run(&self) -> anyhow::Result<()> {
        for port in serialport::available_ports()? {
            if port.port_name == common::default_serial_port() {
                eprintln!("* {}", port.port_name);
            } else {
                eprintln!("  {}", port.port_name);
            }
            if let serialport::SerialPortType::UsbPort(usb) = port.port_type {
                eprintln!("    - USB {:x}:{:x}", usb.vid, usb.pid);
                if let Some(serial_number) = usb.serial_number {
                    eprintln!("    - S/N: {}", serial_number);
                }
                if let Some(manufacturer) = usb.manufacturer {
                    eprintln!("    - {}", manufacturer);
                }
                if let Some(product) = usb.product {
                    eprintln!("    - {}", product);
                }
            }
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    use clap::Parser;
    let opts = ToolOptions::parse();

    opts.command.run()
}
