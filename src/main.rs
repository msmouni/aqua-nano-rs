#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod stepper;

use arduino_hal::hal::port::{PB0, PB1, PB2, PD7};
use panic_halt as _;
use stepper::{AngleSpeed, RotationAngleSpeed, Stepper};
mod time;

use panic_halt as _;
use time::{
    sys_timer::{CtcTimer, SysTimer},
    timer::Timer,
};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // let mut serial = arduino_hal::default_serial!(dp, pins, 9600);

    // Digital pin 13 is also connected to an onboard LED marked "L"
    let mut led = pins.d13.into_output();
    led.set_low();

    // Stepper motor
    let mut stepper_motor = Stepper::new(
        pins.d7.into_output(),
        pins.d8.into_output(),
        pins.d9.into_output(),
        pins.d10.into_output(),
        stepper::StepType::Step8,
    );
    let mut enable_stepper_pin = pins.d6.into_output();

    let mut sys_timer: SysTimer<CtcTimer<16, 64, 250>> = SysTimer::new(dp.TC0);

    sys_timer.init();

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    let mut led_toggle_count = 0;
    let mut led_toggle_timer = Timer::new(100_000);
    let mut led_off_timer = Timer::new(800_000);

    let mut light_pin = pins.d5.into_output();
    light_pin.set_high();
    let mut light_timer = Timer::new(7 * 60 * 60 * 1_000 * 1_000); // 7h

    let mut day_timer = Timer::new(24 * 60 * 60 * 1_000 * 1_000); // 24h

    // init Stepper motor
    enable_stepper_pin.set_high();
    sys_timer.delay_micros(2_000_000); //2s
    stepper_motor.rotate_by_angle(RotationAngleSpeed::AntiClockwise(AngleSpeed::new(
        45.0, 15.0,
    )));
    stepper_motor.rotate_by_angle(RotationAngleSpeed::Clockwise(AngleSpeed::new(10.0, 15.0)));

    loop {
        let t = sys_timer.micros();
        if !day_timer.has_started() {
            day_timer.start(t);

            enable_stepper_pin.set_high();
            sys_timer.delay_micros(2_000_000); //2s

            light_pin.set_high();
            led.set_high();

            //Rotate
            stepper_motor
                .rotate_by_angle(RotationAngleSpeed::Clockwise(AngleSpeed::new(22.5, 35.0)));

            // Vibration
            vibarte(&mut stepper_motor, 10.0, 35.0, 10);

            enable_stepper_pin.set_low();

            led_toggle_count = 0;
            led_off_timer.start(t);

            light_timer.start(t);
        } else {
            if led_toggle_count == 4 {
                led_toggle_count = 0;
                led_toggle_timer.stop();
                led_off_timer.start(t);
            }

            if let Ok(has_expired) = led_off_timer.has_expired(t) {
                if has_expired {
                    led_off_timer.stop();
                    led_toggle_timer.start(t);
                }
            }

            if let Ok(has_expired) = led_toggle_timer.has_expired(t) {
                if has_expired {
                    led.toggle();
                    led_toggle_count += 1;
                    led_toggle_timer.start(t);
                }
            }

            if let Ok(has_expired) = light_timer.has_expired(t) {
                if has_expired {
                    light_pin.set_low();
                    light_timer.stop();
                }
            }

            if let Ok(has_expired) = day_timer.has_expired(t) {
                if has_expired {
                    day_timer.stop();

                    sys_timer.reset_time();
                }
            }
        }

        // ufmt::uwriteln!(&mut serial, "Hello : {:?}", led_toggle_count).unwrap();
    }
}

fn vibarte(
    stepper_motor: &mut Stepper<PD7, PB0, PB1, PB2>,
    amplitude: f32,
    speed: f32,
    number: usize,
) {
    for _i in 0..number {
        stepper_motor.rotate_by_angle(RotationAngleSpeed::Clockwise(AngleSpeed::new(
            amplitude, speed,
        )));
        stepper_motor.rotate_by_angle(RotationAngleSpeed::AntiClockwise(AngleSpeed::new(
            amplitude, speed,
        )));
    }
}
