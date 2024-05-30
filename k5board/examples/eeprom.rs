#![no_std]
#![no_main]

use panic_halt as _;

use k5board::hal;
use k5board::hal::time::Hertz;
use k5board::prelude::*;

k5board::version!(concat!(env!("CARGO_PKG_VERSION"), "eeprom"));

#[cortex_m_rt::entry]
fn main() -> ! {
    // grab peripherals and initialize the clock
    let p = hal::pac::Peripherals::take().unwrap();
    let power = hal::power::new(p.SYSCON, p.PMU);
    let clocks = power.clocks.sys_internal_24mhz().freeze();

    // turn on GPIOA and GPIOC
    let ports = hal::gpio::new(p.PORTCON, p.GPIOA, p.GPIOB, p.GPIOC);
    let pins_a = ports.port_a.enable(power.gates.gpio_a);

    // set up the uart and install it globally so we can print data out
    let uart_parts = k5board::uart::Parts {
        uart: p.UART1,
        gate: power.gates.uart1,
        tx: pins_a.a7.into_mode(),
        rx: pins_a.a8.into_mode(),
    };
    let uart = k5board::uart::new(&clocks, 38_400.Hz(), uart_parts).unwrap();
    k5board::uart::install(uart);

    // bit-bang i2c needs a timer at twice the desired frequency
    // we'll use TIMER_BASE0 at 200kHz
    let mut i2c_timer = hal::timer::new(p.TIMER_BASE0, power.gates.timer_base0)
        .frequency::<{ Hertz::kHz(200).to_Hz() }>(&clocks)
        .unwrap()
        .split(&clocks)
        .low
        .timing();
    i2c_timer.start_native().unwrap();

    // set up the i2c bus the eeprom lives on
    let i2c_parts = k5board::shared_i2c::Parts {
        clk: i2c_timer,
        scl: pins_a.a10.into_mode().into(),
        sda: pins_a.a11.into_mode().into(),
    };
    let i2c = k5board::shared_i2c::new(i2c_parts);

    // set up the eeprom
    let mut eeprom = k5board::eeprom::new(i2c.acquire());

    // loop through over and over writing a hex dump to uart

    let mut addr = 0;
    let mut buffer = [0; 0x10];
    loop {
        // don't go off the end of the eeprom
        let len = (k5board::eeprom::SIZE - addr).min(buffer.len());

        // read the data
        eeprom.read(addr, &mut buffer[..len]).unwrap();

        // write the data, in hexdump form

        print!("0x{:04x}: ", addr);
        for (i, b) in buffer.iter().enumerate() {
            if i == 8 {
                print!(" ");
            }
            print!(" {:02x}", b);
        }

        print!("  |");
        for b in buffer.iter() {
            if (0x20..0x7f).contains(b) {
                print!("{}", *b as char);
            } else {
                print!(".");
            }
        }

        println!("|");

        // increment address, looping back at the end
        addr += len;
        if addr >= k5board::eeprom::SIZE {
            addr = 0;
            println!();
            cortex_m::asm::delay(clocks.sys_clk().to_Hz());
        }
    }
}
