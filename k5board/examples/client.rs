#![no_std]
#![no_main]

use k5board::hal;
use k5lib::protocol::messages::radio::{HelloReply, ReadEepromReply};
use k5lib::protocol::messages::HostMessage;
use k5lib::protocol::BAUD_RATE;
use panic_halt as _;

use k5board::prelude::*;

// defines VERSION, used below
k5board::version!(concat!(env!("CARGO_PKG_VERSION"), "client"));

#[cortex_m_rt::entry]
fn main() -> ! {
    // grab peripherals and initialize the clock
    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU, p.FLASH_CTRL)
        .sys_internal_24mhz()
        .freeze();

    // turn on GPIOA
    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_a = ports.port_a.enable(power.gates.gpio_a);

    // set up the uart and install it globally, then get a client
    let uart_parts = k5board::uart::Parts {
        uart: p.UART1,
        gate: power.gates.uart1,
        tx: pins_a.a7.into_mode(),
        rx: pins_a.a8.into_mode(),
    };
    let uart = k5board::uart::new(BAUD_RATE.Hz(), uart_parts).unwrap();
    let mut client = k5board::uart::install(uart).client();

    let mut session_id = 0;

    loop {
        match client.read_host().map(|v| v.ok()) {
            Ok(Some(HostMessage::Hello(msg))) => {
                session_id = msg.session_id;
                client
                    .write(&HelloReply {
                        version: VERSION.clone(),
                        has_custom_aes_key: false,
                        is_in_lock_screen: false,
                        _pad: Default::default(),
                        challenge: Default::default(),
                    })
                    .unwrap();
            }

            Ok(Some(HostMessage::ReadEeprom(msg))) => {
                if msg.session_id != session_id {
                    continue;
                }

                // dummy data
                let mut data = [0; 0x100];
                for (i, b) in data.iter_mut().enumerate() {
                    *b = ((msg.address as usize + i) & 0xff) as u8;
                }

                client
                    .write(&ReadEepromReply {
                        address: msg.address,
                        len: msg.len,
                        _pad: Default::default(),
                        data: &data[..msg.len as usize],
                    })
                    .unwrap();
            }

            // ignore errors and other messages
            _ => {}
        }
    }
}
