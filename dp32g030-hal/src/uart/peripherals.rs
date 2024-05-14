use core::ops::Deref;

use crate::pac;

use crate::gpio::alt;
use crate::power::Device;

/// A trait for UARTs.
#[allow(private_bounds)]
#[cfg(not(feature = "defmt"))]
pub trait Instance: InstanceSealed + Device {
    type Rts: core::fmt::Debug;
    type Cts: core::fmt::Debug;
    type Rx: core::fmt::Debug;
    type Tx: core::fmt::Debug;
}

/// A trait for UARTs.
#[allow(private_bounds)]
#[cfg(feature = "defmt")]
pub trait Instance: InstanceSealed + Device {
    type Rts: core::fmt::Debug + defmt::Format;
    type Cts: core::fmt::Debug + defmt::Format;
    type Rx: core::fmt::Debug + defmt::Format;
    type Tx: core::fmt::Debug + defmt::Format;
}

/// A trait for UARTs.
pub(super) trait InstanceSealed: Deref<Target = pac::uart0::RegisterBlock> {
    /// Steal the UART peripheral.
    ///
    /// # Safety
    /// Every existing clone of this UART peripheral must be
    /// used for exclusive purposes, such as Rx and Tx sides.
    unsafe fn steal(&self) -> Self;
}

impl Instance for pac::UART0 {
    type Rts = alt::uart0::Rts;
    type Cts = alt::uart0::Cts;
    type Rx = alt::uart0::Rx;
    type Tx = alt::uart0::Tx;
}

impl InstanceSealed for pac::UART0 {
    #[inline(always)]
    unsafe fn steal(&self) -> Self {
        pac::UART0::steal()
    }
}

impl Instance for pac::UART1 {
    type Rts = alt::uart1::Rts;
    type Cts = alt::uart1::Cts;
    type Rx = alt::uart1::Rx;
    type Tx = alt::uart1::Tx;
}

impl InstanceSealed for pac::UART1 {
    #[inline(always)]
    unsafe fn steal(&self) -> Self {
        pac::UART1::steal()
    }
}

impl Instance for pac::UART2 {
    type Rts = alt::uart2::Rts;
    type Cts = alt::uart2::Cts;
    type Rx = alt::uart2::Rx;
    type Tx = alt::uart2::Tx;
}

impl InstanceSealed for pac::UART2 {
    #[inline(always)]
    unsafe fn steal(&self) -> Self {
        pac::UART2::steal()
    }
}
