/*!
 * A basic implementation of the `millis()` function from Arduino:
 *
 *     https://www.arduino.cc/reference/en/language/functions/time/millis/
 *
 * Uses timer TC0 and one of its interrupts to update a global millisecond
 * counter.  A walkthough of this code is available here:
 *
 *     https://blog.rahix.de/005-avr-hal-millis/
 */
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::prelude::*;
use core::cell;
use panic_halt as _;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Possible Values:
//
// ╔═══════════╦══════════════╦═══════════════════╗
// ║ PRESCALER ║ TIMER_COUNTS ║ Overflow Interval ║
// ╠═══════════╬══════════════╬═══════════════════╣
// ║        64 ║          250 ║              1 ms ║
// ║       256 ║          125 ║              2 ms ║
// ║       256 ║          250 ║              4 ms ║
// ║      1024 ║          125 ║              8 ms ║
// ║      1024 ║          250 ║             16 ms ║
// ╚═══════════╩══════════════╩═══════════════════╝
const PRESCALER: u32 = 1024;
const TIMER_COUNTS: u32 = 125;

const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16000;

static MILLIS_COUNTER: avr_device::interrupt::Mutex<cell::Cell<u32>> =
    avr_device::interrupt::Mutex::new(cell::Cell::new(0));

fn millis_init(tc0: &arduino_hal::pac::TC0) {
    // tmp ref
    // Configure the timer for the above interval (in CTC mode)
    // and enable its interrupt.
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    tc0.ocr0a.write(|w| unsafe { w.bits(TIMER_COUNTS as u8) });
    tc0.tccr0b.write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });
    tc0.timsk0.write(|w| w.ocie0a().set_bit());

    // Reset the global millisecond counter
    avr_device::interrupt::free(|cs| {
        MILLIS_COUNTER.borrow(cs).set(0);
    });
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let counter_cell = MILLIS_COUNTER.borrow(cs);
        let counter = counter_cell.get();
        counter_cell.set(counter + MILLIS_INCREMENT);
    })
}

fn millis() -> u32 {
    avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
}
//////////////////////////////////////////////////////////////////////////////////////////////////////////

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);

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

        /*led.toggle();
        arduino_hal::delay_ms(100);
        led.toggle();
        arduino_hal::delay_ms(100);
        led.toggle();
        arduino_hal::delay_ms(100);
        led.toggle();
        arduino_hal::delay_ms(800);*/
        // ufmt::uwriteln!(&mut serial, "Hello : {:?}", led_toggle_count).void_unwrap();
    }
}
