use crate::drivers::time::timer::Timer;
use arduino_hal::port::{mode::Output, Pin, PinOps};

pub struct Light<LightPin: PinOps> {
    timer: Timer,
    pin: Pin<Output, LightPin>,
}

impl<LightPin: PinOps> Light<LightPin> {
    const LIGHT_ON_TIMEOUT_US: u64 = 7 * 60 * 60 * 1_000 * 1_000; // 7h

    pub fn new(light_pin: Pin<Output, LightPin>) -> Self {
        Self {
            timer: Timer::new(Self::LIGHT_ON_TIMEOUT_US),
            pin: light_pin,
        }
    }

    pub fn init(&mut self, t_us: u64) {
        self.pin.set_high();
        self.timer.start(t_us);
    }

    pub fn update(&mut self, t_us: u64) {
        if let Ok(has_expired) = self.timer.has_expired(t_us) {
            if has_expired {
                self.pin.set_low();
                self.timer.stop();
            }
        }
    }
}
