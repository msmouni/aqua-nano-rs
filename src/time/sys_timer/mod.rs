mod ctc;
mod fast_pwm;

use super::time::Time;
use arduino_hal::pac::TC0;
use avr_device::interrupt::Mutex;
use core::cell::Cell;
pub use {ctc::CtcTimer, fast_pwm::FastPwmTimer};

static OVER_FLOW_COUNTER: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

// #[derive(Clone)]
// pub enum Prescaler {
//     Prescaler1 = 1,
//     Prescaler8 = 8,
//     Prescaler64 = 64,
//     Prescaler256 = 256,
//     Prescaler1024 = 1024,
// }

fn reset_time() {
    avr_device::interrupt::free(|cs| {
        OVER_FLOW_COUNTER.borrow(cs).set(0);
    });
}

pub trait ImplTimer {
    fn new(timer_counter: TC0) -> Self;

    fn init(&mut self);

    fn micros(&self) -> u64;
}

pub struct SysTimer<WhichTimer: ImplTimer> {
    sys_timer: WhichTimer,
}

impl<WhichTimer: ImplTimer> SysTimer<WhichTimer> {
    pub fn new(timer_counter: TC0) -> Self {
        Self {
            sys_timer: WhichTimer::new(timer_counter),
        }
    }

    pub fn init(&mut self) {
        self.sys_timer.init()
    }

    #[allow(dead_code)]
    pub fn micros(&self) -> u64 {
        self.sys_timer.micros()
    }

    #[allow(dead_code)]
    pub fn millis(&self) -> u64 {
        self.micros() / 1_000
    }

    #[allow(dead_code)]
    pub fn get_time(&mut self) -> Time {
        let elapsed_micros = self.micros();

        Time::new(elapsed_micros)
    }

    #[allow(dead_code)]
    pub fn delay_micros(&self, delay_us: u64) {
        let mut t = self.micros();

        let t_start = t;

        while (t - t_start) < delay_us {
            avr_device::asm::nop();
            avr_device::asm::nop();
            avr_device::asm::nop();

            t = self.micros();
        }
    }

    pub fn reset_time(&self) {
        reset_time()
    }
}
