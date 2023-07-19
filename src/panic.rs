use crate::tools::str_writer::StrWriter;
use arduino_hal::delay_ms;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts globally
    unsafe { avr_device::interrupt::disable() };

    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);

    let mut led_pin = pins.d13.into_output();

    led_pin.set_high();

    delay_ms(1_000);

    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);

    let mut str_w = StrWriter::<100>::default();

    loop {
        if let Some(args) = info.message() {
            if let Ok(msg) = str_w.get_str(*args) {
                if let Some(loc) = info.location() {
                    ufmt::uwriteln!(
                        &mut serial,
                        "Panic: {} at file: {} | line: {} \n",
                        msg,
                        loc.file(),
                        loc.line()
                    )
                    .ok();
                } else {
                    ufmt::uwriteln!(&mut serial, "Panic: {}\n", msg).ok();
                }
            } else {
                ufmt::uwriteln!(&mut serial, "Panic\n").ok();
            }
        } else {
            ufmt::uwriteln!(&mut serial, "Panic\n").ok();
        }

        delay_ms(1_000);
    }
}
