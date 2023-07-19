mod alive;
mod feeder;
mod light;

use crate::drivers::{
    stepper::{StepType, Stepper},
    time::{
        sys_timer::{CtcTimer, SysTimer},
        timer::Timer,
    },
};
use alive::AliveBeat;
use arduino_hal::{
    hal::port::{PB0, PB1, PB2, PD0, PD1, PD5, PD6, PD7},
    port::{
        mode::{AnyInput, Input, Output},
        Pin,
    },
    Usart,
};
use avr_device::atmega328p::USART0;
use core::fmt::Arguments;
use feeder::Feeder;
use light::Light;

pub struct Application {
    sys_timer: SysTimer<CtcTimer<16, 64, 250>>,
    alive: AliveBeat,
    day_timer: Timer,
    light: Light<PD5>,
    feeder: Feeder<PD6, PD7, PB0, PB1, PB2>,
    #[allow(dead_code)]
    serial: Usart<USART0, Pin<Input<AnyInput>, PD0>, Pin<Output, PD1>>,
}

impl Application {
    const DAY_US: u64 = 24 * 60 * 60 * 1_000 * 1_000; // 24h

    pub fn new() -> Self {
        let dp = arduino_hal::Peripherals::take().unwrap();
        let pins = arduino_hal::pins!(dp);

        let serial = arduino_hal::default_serial!(dp, pins, 9600);

        // Digital pin 13 is also connected to an onboard LED marked "L"
        let alive = AliveBeat::new(pins.d13.into_output());

        let mut sys_timer: SysTimer<CtcTimer<16, 64, 250>> = SysTimer::new(dp.TC0);

        sys_timer.init();

        // Enable interrupts globally
        unsafe { avr_device::interrupt::enable() };

        let mut feeder = Feeder::new(
            pins.d6.into_output(),
            Stepper::new(
                pins.d7.into_output(),
                pins.d8.into_output(),
                pins.d9.into_output(),
                pins.d10.into_output(),
                StepType::Step8,
            ),
        );

        feeder.init_position(&sys_timer);

        Self {
            sys_timer,
            alive,
            day_timer: Timer::new(Self::DAY_US),
            light: Light::new(pins.d5.into_output()),
            feeder,
            serial,
        }
    }

    pub fn update(&mut self) {
        let t_us = self.sys_timer.micros();
        if !self.day_timer.has_started() {
            self.day_timer.start(t_us);

            // Light
            self.light.init(t_us);

            self.feeder.deliver_food(&self.sys_timer);

            self.alive.reset(t_us);
        } else {
            self.alive.update(&mut self.sys_timer);

            self.light.update(t_us);

            if let Ok(has_expired) = self.day_timer.has_expired(t_us) {
                if has_expired {
                    self.day_timer.stop();

                    self.sys_timer.reset_time();
                }
            }
        }

        // self.usb_debug(format_args!("Hello : {:?}", 1));
    }

    #[allow(dead_code)]
    /// Example: self.usb_debug(format_args!("T_{}", 1));
    pub fn usb_debug(&mut self, args: Arguments) {
        if let Some(s) = args.as_str() {
            ufmt::uwriteln!(&mut self.serial, "{}", s).unwrap();
        }
    }
}
