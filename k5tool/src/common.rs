#[derive(clap::Args, Debug, Clone)]
pub struct SerialPortArgs {
    #[arg(default_value_t = default_serial_port())]
    port: String,
    #[arg(short, long, default_value_t = k5lib::protocol::BAUD_RATE)]
    baud: u32,
    #[arg(long)]
    tcp: bool,
    #[arg(long, default_value_t = 5)]
    timeout: u64,
}

#[derive(Debug)]
pub enum SerialPort {
    Serial(std::io::BufWriter<Box<dyn serialport::SerialPort>>),
    Tcp(std::io::BufWriter<std::net::TcpStream>),
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
            Self::Serial(port) => port.get_mut().read(buf),
            Self::Tcp(port) => port.get_mut().read(buf),
        }
    }
}

impl std::io::Write for SerialPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Serial(port) => port.write(buf),
            Self::Tcp(port) => port.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Serial(port) => port.flush(),
            Self::Tcp(port) => port.flush(),
        }
    }
}

impl SerialPortArgs {
    pub fn open(&self) -> anyhow::Result<SerialPort> {
        let timeout = std::time::Duration::from_secs(self.timeout);
        if self.tcp {
            let port = std::net::TcpStream::connect(&self.port)?;
            port.set_read_timeout(Some(timeout))?;
            port.set_write_timeout(Some(timeout))?;
            Ok(SerialPort::Tcp(std::io::BufWriter::new(port)))
        } else {
            let mut port = serialport::new(&self.port, self.baud).open()?;
            port.set_timeout(timeout)?;
            Ok(SerialPort::Serial(std::io::BufWriter::new(port)))
        }
    }
}

pub fn read_le_u32(data: &[u8]) -> Option<u32> {
    if data.len() < 4 {
        None
    } else {
        Some(
            (data[0] as u32)
                | ((data[1] as u32) << 8)
                | ((data[2] as u32) << 16)
                | ((data[3] as u32) << 24),
        )
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
