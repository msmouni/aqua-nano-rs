use crate::drivers::time::{
    sys_timer::{ImplTimer, SysTimer},
    timer::Timer,
};
use arduino_hal::{
    hal::port::PB5,
    port::{mode::Output, Pin},
};

pub struct AliveBeat {
    led: Pin<Output, PB5>, // Digital pin 13 is also connected to an onboard LED marked "L"
    led_toggle_timer: Timer,
    led_off_timer: Timer,
    led_toggle_count: u32,
}

impl AliveBeat {
    const LED_TOGGLE_TIMEOUT_US: u64 = 100_000;
    const LED_OFF_TIMEOUT_US: u64 = 800_000;

    const LED_TOGGLE_MAX_COUNT: u32 = 4;

    // Digital pin 13 is also connected to an onboard LED marked "L"
    pub fn new(mut led_pin: Pin<Output, PB5>) -> Self {
        led_pin.set_low();

        Self {
            led: led_pin,
            led_toggle_timer: Timer::new(Self::LED_TOGGLE_TIMEOUT_US),
            led_off_timer: Timer::new(Self::LED_OFF_TIMEOUT_US),
            led_toggle_count: 0,
        }
    }

    pub fn reset(&mut self, t_us: u64) {
        self.led.set_high();

        self.led_toggle_count = 0;
        self.led_off_timer.start(t_us);
    }

    pub fn update<WhichTimer: ImplTimer>(&mut self, sys_timer: &mut SysTimer<WhichTimer>) {
        let t = sys_timer.micros();

        if self.led_toggle_count == Self::LED_TOGGLE_MAX_COUNT {
            self.led_toggle_count = 0;
            self.led_toggle_timer.stop();
            self.led_off_timer.start(t);
        }

        if let Ok(has_expired) = self.led_off_timer.has_expired(t) {
            if has_expired {
                self.led_off_timer.stop();
                self.led_toggle_timer.start(t);
            }
        }

        if let Ok(has_expired) = self.led_toggle_timer.has_expired(t) {
            if has_expired {
                self.led.toggle();
                self.led_toggle_count += 1;
                self.led_toggle_timer.start(t);
            }
        }
    }
}
