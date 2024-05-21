use core::ops::{Bound, DerefMut, RangeBounds};

use bitbang_hal::i2c::{Error, I2cBB};
use embedded_hal_02::digital::v2::{InputPin, OutputPin};
use embedded_hal_02::timer::{CountDown, Periodic};

// device id
pub const DEVICE_ID: u8 = 0x80;

// register addresses
pub const REG_UNKNOWN00: u8 = 0x00;
pub const REG_CHIP_ID: u8 = 0x01;
pub const REG_POWER: u8 = 0x02;
pub const REG_CHANNEL: u8 = 0x03;
pub const REG_SYSTEM1: u8 = 0x04;
pub const REG_SYSTEM2: u8 = 0x05;
pub const REG_SYSTEM3: u8 = 0x06;
pub const REG_TEST1: u8 = 0x07;
pub const REG_TEST2: u8 = 0x08;
pub const REG_BOOT: u8 = 0x09;
pub const REG_RSSI: u8 = 0x0a;
pub const REG_READ_CHANNEL: u8 = 0x0b;

// other addresses exist but are internal

// size of addressable space, in u16s
pub const REG_MAX: u8 = 0x22;

pub struct Bk1080<I2c> {
    i2c: I2c,
    registers: [u16; REG_MAX as usize],
}

impl<I2c, Scl, Sda, Clk, E> Bk1080<I2c>
where
    I2c: DerefMut<Target = I2cBB<Scl, Sda, Clk>>,
    Scl: OutputPin<Error = E>,
    Sda: InputPin<Error = E> + OutputPin<Error = E>,
    Clk: CountDown + Periodic,
{
    pub fn new(i2c: I2c) -> Result<Self, Error<E>> {
        let mut bk1080 = Self {
            i2c,
            registers: [0; REG_MAX as usize],
        };

        bk1080.update(..)?;

        Ok(bk1080)
    }

    pub fn update<R>(&mut self, range: R) -> Result<&[u16], Error<E>>
    where
        R: RangeBounds<u8>,
    {
        let start = match range.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i + 1,
            Bound::Unbounded => 0,
        } as usize;

        let end = match range.end_bound() {
            Bound::Included(i) => *i as usize + 1,
            Bound::Excluded(i) => *i as usize,
            Bound::Unbounded => self.registers.len(),
        };

        if start > end || end > self.registers.len() {
            // not a great choice but it'll do
            return Err(Error::InvalidData);
        }

        if start == end {
            return Ok(&[]);
        }

        let data = bytemuck::cast_slice_mut(&mut self.registers[start..end]);

        self.i2c.raw_i2c_start()?;
        self.i2c
            .raw_write_to_slave(&[DEVICE_ID, ((start as u8) << 1) | 1])?;
        self.i2c.raw_read_from_slave(data)?;
        self.i2c.raw_i2c_stop()?;

        for d in self.registers[start..end].iter_mut() {
            *d = u16::from_be(*d);
        }

        Ok(&self.registers[start..end])
    }

    pub fn get(&self, address: u8) -> Option<u16> {
        self.registers.get(address as usize).copied()
    }

    pub fn read(&mut self, address: u8) -> Result<u16, Error<E>> {
        Ok(self.update(address..address + 1)?[0])
    }

    pub fn write(&mut self, address: u8, data: u16) -> Result<u16, Error<E>> {
        if address as usize >= self.registers.len() {
            return Err(Error::InvalidData);
        }

        self.i2c.raw_i2c_start()?;
        self.i2c.raw_write_to_slave(&[
            DEVICE_ID,
            address << 1,
            ((data & 0xff00) >> 8) as u8,
            (data & 0x00ff) as u8,
        ])?;
        self.i2c.raw_i2c_stop()?;

        self.read(address)
    }

    pub fn enable(&mut self) -> Result<(), Error<E>> {
        const INITREGS: &[u16] = &[
            0x0008, // 0x00 (r) : unknown 0
            0x1080, // 0x01 (r) : chip id
            // 0x02 (rw): power
            // [   15] DSMUTE = 0 (enable soft mute)
            // [   14] MUTE = 0 (disable mute)
            // [   13] MONO = 0 (stereo)
            // [   12] CKSEL = 0 (external clock)
            // [   11] reserved = 0
            // [   10] SKMODE = 0 (wrap)
            // [    9] SEEKUP = 1 (seek up)
            // [    8] SEEK = 0 (disabled)
            // [    7] reserved = 0
            // [    6] DISABLE = 0
            // [ 5: 1] reserved = 0
            // [    0] ENABLE = 1
            0x0201,
            // 0x03 (rw): channel
            // [   15] TUNE = 0 (disable)
            // [14:10] reserved = 0
            // [ 9: 0] CHAN = 0
            0x0000,
            // 0x04 (rw): system 1
            // [   15] reserved = 0
            // [   14]: STCIEN = 1 (enable interrupt)
            // [   13]: DEBPS = 0 (enable de-emphasis filter)
            // [   12]: reserved = 0
            // [   11]: DE = 0 (75 us de-emphasis, USA)
            // [   10]: AGCD = 0 (AGC enabled)
            // [ 9: 8]: reserved = 0
            // [ 7: 6]: BLNDADJ = 0b11 (25-43 RSSI dBuV (-6dB) stereo blend)
            // [ 5: 4]: GPIO3 = 0b00 (high impedence)
            // [ 3: 2]: GPIO2 = 0b00 (high impedence)
            // [ 1: 0]: GPIO1 = 0b00 (high impedence)
            0x40C0,
            // 0x05 (rw): system 2
            // [15: 8]: SEEKTH = 0x0a (rssi seek threshold)
            // [ 7: 6]: BAND = 0b00 (87.5 - 108 MHz, USA/Europe)
            // [ 5: 4]: SPACE = 0b01 (100 kHz, Europe/Japan)
            // [ 3: 0]: VOLUME = 0b111 (0dB full scale)
            0x0A1F,
            // 0x06 (rw): system 3
            // [15:14]: SMUTER = 0b00 (fastest soft mute)
            // [13:12]: SMUTEA = 0b00 (16dB soft mute)
            // [11: 8]: reserved = 0
            // [ 7: 4]: SKSNR = 0b0010 (almost all stops while seeking)
            // [ 3: 0]: SKCNT = 0b1110 (almost minimum impulses for seek)
            0x002E,
            // 0x07 (rw): test 1
            // [15: 4]: FREQD = 0x2f, 148Hz "on bit"
            // [ 3: 0]: SNR = 0xf (??)
            0x02FF,
            // 0x08 (rw): test 2
            // [   15]: reserved = 0
            // [14: 0]: reserved = 0x5b11 (should be same as startup)
            0x5B11,
            // 0x09 (rw): boot
            // [15: 0]: boot = 0x0000 (should be same as startup)
            0x0000,
            // 0x0a (r) : rssi
            // [   15]: reserved = 0
            // [   14]: STC = 1 (seek/tune complete)
            // [   13]: SF/BL = 0 (seek successful)
            // [   12]: AFCRL = 0 (AFC not railed, channel valid)
            // [11:10]: reserved = 0b00
            // [    9]: STEN = 1 (stereo indicator on)
            // [    8]: ST = 0 (mono)
            // [ 7: 0]: RSSI = 0x1e (0x00 to 0xff)
            0x411E,
            // 0x0b (r) : read channel
            // [15:14]: reserved = 0b00
            // [13:10]: IMPC = 0b0000
            // [ 9: 0]: READCHAN = 0
            0x0000,
            //
            // the rest are reserved
            //
            0xCE00, // 0x0c (r) : reserved 0, always 0
            0x0000, // 0x0d (r) : reserved 1, always 0
            0x0000, // 0x0e (r) : reserved 2, always 0
            0x1000, // 0x0f (r) : reserved 3, always 0
            //
            // the rest are internal registers
            //
            0x3197, 0x0000, 0x13FF, 0x9852, 0x0000, 0x0000, 0x0008, 0x0000, 0x51E1, 0xA8BC, 0x2645,
            0x00E4, 0x1CD8, 0x3A50, 0xEAE0, 0x3000, 0x0200, 0x0000,
        ];
        for (addr, val) in INITREGS.iter().enumerate() {
            self.write(addr as u8, *val)?;
        }
        self.write(REG_POWER, 0x0201)?;
        self.write(REG_SYSTEM2, 0x0a0f)?;
        Ok(())
    }

    pub fn tune(&mut self, freq: u16) -> Result<(), Error<E>> {
        // FIXME delay? retry timeout??
        self.write(REG_CHANNEL, freq & 0x3ff)?;
        while self.read(REG_RSSI)? & (1 << 14) > 0 {}
        self.write(REG_CHANNEL, (freq & 0x3ff) | 0x8000)?;
        while self.read(REG_RSSI)? & (1 << 14) == 0 {}
        Ok(())
    }
}
