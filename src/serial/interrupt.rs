use super::{buffer::USART_RX_BUFFER, MUTEX_SERIAL_RX};

#[avr_device::interrupt(atmega328p)]
fn USART_RX() {
    avr_device::interrupt::free(|cs| {
        if let Some(serial_rx) = MUTEX_SERIAL_RX.borrow(cs).borrow_mut().as_mut() {
            if let Ok(b) = serial_rx.read_byte() {
                if let Some(rx_buffer) = USART_RX_BUFFER.borrow(cs).borrow_mut().as_mut() {
                    rx_buffer.try_push(b);
                }
            }
        }
    })
}
