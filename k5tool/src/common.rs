#[derive(clap::Args, Debug, Clone)]
pub struct SerialPortArgs {
    #[arg(default_value_t = default_serial_port())]
    port: String,
    #[arg(short, long, default_value_t = k5lib::protocol::BAUD_RATE)]
    baud: u32,
    #[arg(long)]
    plain_file: bool,
}

#[derive(Debug)]
pub enum SerialPort {
    Serial(Box<dyn serialport::SerialPort>),
    File(std::fs::File),
}

pub fn default_serial_port() -> String {
    if let Ok(infos) = serialport::available_ports() {
        for info in infos {
            #[cfg(target_os = "macos")]
            if info.port_name.ends_with(".Bluetooth-Incoming-Port") {
                // these ports are almost always *not* what we want
                continue;
            }

            #[cfg(target_os = "macos")]
            if info.port_name.starts_with("/dev/tty.") {
                // macos ports with tty. have flow control we don't use
                // use cu. ports instead!
                continue;
            }

            return info.port_name.clone();
        }
    }

    // not great, but reasonable fallback
    "/dev/ttyUSB0".to_owned()
}

impl std::io::Read for SerialPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Serial(port) => port.read(buf),
            Self::File(port) => port.read(buf),
        }
    }
}

impl std::io::Write for SerialPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Serial(port) => port.write(buf),
            Self::File(port) => port.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Serial(port) => port.flush(),
            Self::File(port) => port.flush(),
        }
    }
}

impl SerialPortArgs {
    pub fn open(&self) -> anyhow::Result<SerialPort> {
        if self.plain_file {
            let port = std::fs::File::options()
                .read(true)
                .write(true)
                .open(&self.port)?;

            Ok(SerialPort::File(port))
        } else {
            let mut port = serialport::new(&self.port, self.baud).open()?;
            port.set_timeout(std::time::Duration::from_secs(1))?;
            Ok(SerialPort::Serial(port))
        }
    }
}

pub fn download_bar(size: u64) -> indicatif::ProgressBar {
    let bar = indicatif::ProgressBar::new(size);
    bar.set_style(
        indicatif::ProgressStyle::with_template(
            "({spinner}) [{wide_bar}] ({percent:>3}%, {bytes_per_sec:>12})",
        )
        .unwrap()
        .progress_chars("=> ")
        .tick_strings(&["<<<  ", "<<  <", "<  <<", "  <<<", " <<< ", "-----"]),
    );
    bar
}

pub fn e_hexdump(prefix: &str, bytes: &[u8]) {
    for s in hexdump::hexdump_iter(bytes) {
        if prefix.len() > 0 {
            eprintln!("{} {}", prefix, s);
        } else {
            eprintln!("{}", s);
        }
    }
}
