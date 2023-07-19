use crate::tools::buffer::BufferU8;
use avr_device::interrupt::Mutex;
use core::cell::RefCell;

pub(crate) const RX_BUFFER_SIZE: usize = 40;

pub(crate) type RxBufferType = Mutex<RefCell<Option<BufferU8<RX_BUFFER_SIZE>>>>;

pub(crate) static USART_RX_BUFFER: RxBufferType =
    Mutex::new(RefCell::new(Some(BufferU8::new_const())));

pub struct UsartRxBuffer(&'static RxBufferType);

impl Default for UsartRxBuffer {
    fn default() -> Self {
        Self(&USART_RX_BUFFER)
    }
}

impl UsartRxBuffer {
    pub fn try_get_byte(&self) -> Option<u8> {
        avr_device::interrupt::free(|cs| {
            self.0.borrow(cs).borrow_mut().as_mut().unwrap().get_first()
        })
    }
}

#[derive(Default)]
pub(crate) struct UartBuffers {
    rx: UsartRxBuffer,
}

impl UartBuffers {
    pub fn try_get_rx_byte(&self) -> Option<u8> {
        self.rx.try_get_byte()
    }
}
