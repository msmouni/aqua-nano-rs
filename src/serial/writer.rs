use super::{buffer::RX_BUFFER_SIZE, error::SerialError};
use crate::tools::str_writer::StrWriter;
use arduino_hal::{
    hal::port::{PD0, PD1},
    port::{
        mode::{Input, Output},
        Pin,
    },
    prelude::_embedded_hal_serial_Write,
    usart::UsartWriter,
};
use avr_device::atmega328p::USART0;
use ufmt::uWrite;

pub struct SerialWriter {
    usart_tx: UsartWriter<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    str_writer: StrWriter<RX_BUFFER_SIZE>,
}

impl SerialWriter {
    pub(crate) fn new(usart_tx: UsartWriter<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) -> Self {
        Self {
            usart_tx,
            str_writer: Default::default(),
        }
    }
    pub(crate) fn write_byte(&mut self, byte: u8) -> nb::Result<(), SerialError> {
        self.usart_tx
            .write(byte)
            .map_err(|err| err.map(|_| SerialError::WriteBuffer))
    }

    pub(crate) fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), SerialError> {
        for byte in bytes.iter() {
            nb::block!(self.write_byte(*byte))?;
        }
        Ok(())
    }

    pub(crate) fn write_str(&mut self, str_w: &str) -> Result<(), SerialError> {
        self.usart_tx
            .write_str(str_w)
            .map_err(|_| SerialError::WriteBuffer)
    }

    pub(crate) fn write_fmt(&mut self, args: core::fmt::Arguments) -> Result<(), SerialError> {
        if let Ok(s) = self.str_writer.get_str(args) {
            self.usart_tx
                .write_str(s)
                .map_err(|_| SerialError::WriteBuffer)
        } else {
            Err(SerialError::WriteFmt)
        }
    }
}
