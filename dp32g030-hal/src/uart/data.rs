/// A trait for UART data types.
pub(super) trait UartDataSealed {
    const NINEBIT: bool;
}

/// A trait for UART data types.
#[allow(private_bounds)]
pub trait UartData: UartDataSealed + Copy + From<u8> {}

/// u8 for eight-bit UARTS.
impl UartData for u8 {}

impl UartDataSealed for u8 {
    const NINEBIT: bool = false;
}

/// u16 for nine-bit UARTS.
impl UartData for u16 {}

impl UartDataSealed for u16 {
    const NINEBIT: bool = true;
}
