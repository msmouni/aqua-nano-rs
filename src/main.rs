#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(panic_info_message)]

mod app;
mod drivers;
mod esp01;
mod panic;
mod serial;
mod tools;

use app::Application;

use drivers::time::sys_timer::{CtcTimer, SysTimer};
use esp01::EspRespHandler;
use serial::SerialHandler;

#[arduino_hal::entry]
fn main() -> ! {
    // let mut app = Application::new();

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);

    let mut serial_h = SerialHandler::new(serial);

    let mut sys_timer: SysTimer<CtcTimer<16, 64, 250>> = SysTimer::new(dp.TC0);

    sys_timer.init();

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    let mut led_pin = pins.d13.into_output();

    let mut resp_h: EspRespHandler<40> = Default::default();

    loop {
        // app.update();

        led_pin.set_low();

        serial_h.write_str("AT\r\n");

        sys_timer.delay_micros(3_000_000);

        /*
        Note:
         AT
         Bytes(7):
         ERROR
         end
         (7):ERROR

         AT
         Bytes(11):
         (7):ERROR
         end
         (11):(7):ERROR

         AT
        */

        if let Some(_resp) = resp_h.poll() {
            led_pin.set_high();

            serial_h.write_str("Bytes:");
            // serial_h.write_fmt(format_args!("Bytes({}):\n", idx));
            serial_h.write_bytes(resp_h.get_resp_buff());
            serial_h.write_str("end\n");
            if let Some(rs) = resp_h.get_resp_str() {
                serial_h.write_fmt(format_args!("({}):{}\n", rs.len(), rs)); // .write_str(rs);
            }
            resp_h.clear_buff();
        }

        sys_timer.delay_micros(1_000_000);
    }
}
