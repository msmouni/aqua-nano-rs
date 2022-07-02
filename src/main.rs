#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod timer;

use panic_halt as _;
use timer::{millis, millis_init};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // let mut serial = arduino_hal::default_serial!(dp, pins, 9600);

    // Digital pin 13 is also connected to an onboard LED marked "L"
    // let mut led = pins.d13.into_output();
    let mut led = pins.d13.into_output();
    led.set_low();

    millis_init(&dp.TC0);

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    let mut t = millis();

    let d_toggle = 100; //millis_s
    let mut t_toggle = t;
    let mut led_toggle_count = 0;

    let d_led_off = 800;
    let mut t_led_off = t;
    let mut is_led_off = false;

    let mut pin_en = pins.d8.into_output();
    pin_en.set_high();
    let mut is_pin_en_high = true;
    let enable_time_ms = 7 * 60 * 60 * 1_000; // 7h

    let day_ms = 24 * 60 * 60 * 1_000 - 1_000; // 24h (-1s to restart)

    let mut start_loop = true;
    let mut t_start = millis();

    loop {
        t = millis();
        if start_loop {
            t_toggle = t;
            led_toggle_count = 0;

            t_led_off = t;
            is_led_off = false;

            pin_en.set_high();
            is_pin_en_high = true;
            t_start = t;

            start_loop = false;
        } else {
            if led_toggle_count == 4 {
                t_led_off = t;
                led_toggle_count = 0;
                is_led_off = true;
            }

            if (t.wrapping_sub(t_led_off) > d_led_off) && is_led_off {
                is_led_off = false;
            }

            if (t.wrapping_sub(t_toggle) > d_toggle) && !is_led_off {
                led.toggle();
                led_toggle_count += 1;
                t_toggle = t;
            }

            if (t.wrapping_sub(t_start) >= enable_time_ms) && is_pin_en_high {
                pin_en.set_low();
                is_pin_en_high = false;
            }

            if t.wrapping_sub(t_start) >= day_ms {
                millis_init(&dp.TC0);

                arduino_hal::delay_ms(1_000);

                start_loop = true;
            }
        }

        // ufmt::uwriteln!(&mut serial, "Hello : {:?}", led_toggle_count).void_unwrap();
    }
}
