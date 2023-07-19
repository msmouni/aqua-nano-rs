use crate::drivers::{
    stepper::{AngleSpeed, RotationAngleSpeed, Stepper},
    time::sys_timer::{ImplTimer, SysTimer},
};
use arduino_hal::port::{mode::Output, Pin, PinOps};

pub struct Feeder<
    StepperPinEnable: PinOps,
    StepperPinIn1: PinOps,
    StepperPinIn2: PinOps,
    StepperPinIn3: PinOps,
    StepperPinIn4: PinOps,
> {
    enable_stepper_pin: Pin<Output, StepperPinEnable>,
    stepper_motor: Stepper<StepperPinIn1, StepperPinIn2, StepperPinIn3, StepperPinIn4>,
}

impl<
        StepperPinEnable: PinOps,
        StepperPinIn1: PinOps,
        StepperPinIn2: PinOps,
        StepperPinIn3: PinOps,
        StepperPinIn4: PinOps,
    > Feeder<StepperPinEnable, StepperPinIn1, StepperPinIn2, StepperPinIn3, StepperPinIn4>
{
    const ENABLE_STEPPER_DELAY_US: u64 = 2_000_000; //2s

    const INIT_POS_SPEED_DEG_S: f32 = 15.0; // (deg/s)
    const DELIVERY_SPEED_DEG_S: f32 = 35.0; // (deg/s)
    const VIBRATION_SPEED_DEG_S: f32 = 35.0; // (deg/s)

    const INIT_ANTI_CLK_W_ANGLE_DEG: f32 = 45.0;
    const INIT_CLK_W_ANGLE_DEG: f32 = 10.0;

    const DELIVERY_ANGLE_DEG: f32 = 22.5; // 360/16 = 22.5

    const VIBRATION_AMPL_DEG: f32 = 13.0; // (deg)

    pub fn new(
        stepper_pin_en: Pin<Output, StepperPinEnable>,
        stepper_motor: Stepper<StepperPinIn1, StepperPinIn2, StepperPinIn3, StepperPinIn4>,
    ) -> Self {
        Self {
            enable_stepper_pin: stepper_pin_en,
            stepper_motor,
        }
    }

    pub fn init_position<TimerType: ImplTimer>(&mut self, sys_timer: &SysTimer<TimerType>) {
        self.enable_stepper_pin.set_high();
        sys_timer.delay_micros(Self::ENABLE_STEPPER_DELAY_US);

        self.stepper_motor
            .rotate_by_angle(RotationAngleSpeed::AntiClockwise(AngleSpeed::new(
                Self::INIT_ANTI_CLK_W_ANGLE_DEG,
                Self::INIT_POS_SPEED_DEG_S,
            )));

        self.stepper_motor
            .rotate_by_angle(RotationAngleSpeed::Clockwise(AngleSpeed::new(
                Self::INIT_CLK_W_ANGLE_DEG,
                Self::INIT_POS_SPEED_DEG_S,
            )));

        self.enable_stepper_pin.set_low();
    }

    pub fn deliver_food<TimerType: ImplTimer>(&mut self, sys_timer: &SysTimer<TimerType>) {
        self.enable_stepper_pin.set_high();
        sys_timer.delay_micros(Self::ENABLE_STEPPER_DELAY_US);

        //Rotate
        self.stepper_motor
            .rotate_by_angle(RotationAngleSpeed::Clockwise(AngleSpeed::new(
                Self::DELIVERY_ANGLE_DEG,
                Self::DELIVERY_SPEED_DEG_S,
            )));

        // Vibration
        self.vibarte(Self::VIBRATION_AMPL_DEG, Self::VIBRATION_SPEED_DEG_S, 10);

        self.enable_stepper_pin.set_low();
    }

    fn vibarte(&mut self, amplitude: f32, speed: f32, number: usize) {
        for _i in 0..number {
            self.stepper_motor
                .rotate_by_angle(RotationAngleSpeed::Clockwise(AngleSpeed::new(
                    amplitude, speed,
                )));
            self.stepper_motor
                .rotate_by_angle(RotationAngleSpeed::AntiClockwise(AngleSpeed::new(
                    amplitude, speed,
                )));
        }
    }
}
