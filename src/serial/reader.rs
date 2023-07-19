use super::error::SerialError;
use arduino_hal::{
    hal::port::{PD0, PD1},
    port::{
        mode::{Input, Output},
        Pin,
    },
    prelude::_embedded_hal_serial_Read,
    usart::UsartReader,
};
use avr_device::atmega328p::USART0;

pub struct SerialReader {
    usart_rx: UsartReader<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
}

impl SerialReader {
    pub fn new(usart_rx: UsartReader<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) -> Self {
        Self { usart_rx }
    }

    pub fn read_byte(&mut self) -> Result<u8, SerialError> {
        self.usart_rx.read().map_err(|_| SerialError::NoRxData)
    }
}
