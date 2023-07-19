mod buffer;
mod error;
mod interrupt;
mod reader;
mod writer;

use arduino_hal::{
    hal::{
        port::{PD0, PD1},
        usart::Event,
    },
    port::{
        mode::{Input, Output},
        Pin,
    },
    Usart,
};
use avr_device::{atmega328p::USART0, interrupt::Mutex};
use buffer::UartBuffers;
pub(crate) use buffer::UsartRxBuffer;
use core::cell::RefCell;
use reader::SerialReader;
use writer::SerialWriter;

pub(super) static MUTEX_SERIAL_RX: Mutex<RefCell<Option<SerialReader>>> =
    Mutex::new(RefCell::new(None));

pub struct SerialHandler {
    serial_rx: &'static Mutex<RefCell<Option<SerialReader>>>,
    serial_tx: SerialWriter,
    buffers: UartBuffers,
}

impl SerialHandler {
    pub fn new(mut serial: Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) -> Self {
        serial.listen(Event::RxComplete);

        let (rx, tx) = serial.split();

        avr_device::interrupt::free(|cs| {
            MUTEX_SERIAL_RX
                .borrow(cs)
                .borrow_mut()
                .replace(SerialReader::new(rx));
        });

        Self {
            serial_rx: &MUTEX_SERIAL_RX,
            serial_tx: SerialWriter::new(tx),

            buffers: UartBuffers::default(),
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> bool {
        self.serial_tx.write_bytes(bytes).is_ok()
    }

    pub fn write_str(&mut self, s: &str) -> bool {
        self.serial_tx.write_str(s).is_ok()
    }

    pub(crate) fn write_fmt(&mut self, args: core::fmt::Arguments) -> bool {
        self.serial_tx.write_fmt(args).is_ok()
    }
}
