use core::ops::Deref;

use crate::pac;

use crate::gpio::alt;
use crate::power::Device;

/// A trait for SPI peripherals.
#[allow(private_bounds)]
#[cfg(not(feature = "defmt"))]
pub trait Instance: InstanceSealed + Device {
    type Clk: core::fmt::Debug;
    type Miso: core::fmt::Debug;
    type Mosi: core::fmt::Debug;
    type Ssn: core::fmt::Debug;
}

/// A trait for SPI peripherals.
#[allow(private_bounds)]
#[cfg(feature = "defmt")]
pub trait Instance: InstanceSealed + Device {
    type Clk: core::fmt::Debug + defmt::Format;
    type Miso: core::fmt::Debug + defmt::Format;
    type Mosi: core::fmt::Debug + defmt::Format;
    type Ssn: core::fmt::Debug + defmt::Format;
}

/// A trait for SPI peripherals.
pub(super) trait InstanceSealed: Deref<Target = pac::spi0::RegisterBlock> {}

impl Instance for pac::SPI0 {
    type Clk = alt::spi0::Clk;
    type Miso = alt::spi0::Miso;
    type Mosi = alt::spi0::Mosi;
    type Ssn = alt::spi0::Ssn;
}

impl InstanceSealed for pac::SPI0 {}

impl Instance for pac::SPI1 {
    type Clk = alt::spi1::Clk;
    type Miso = alt::spi1::Miso;
    type Mosi = alt::spi1::Mosi;
    type Ssn = alt::spi1::Ssn;
}

impl InstanceSealed for pac::SPI1 {}
