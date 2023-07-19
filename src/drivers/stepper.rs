#![allow(dead_code)]
use arduino_hal::port::{mode::Output, Pin, PinOps};
#[allow(unused_imports)]
use micromath::F32Ext;

const STEP_8_ANGLE: f32 = 0.08789; // 4096 steps = 360°
const STEP_4_ANGLE: f32 = 0.175781; // 2048 steps = 360°

#[derive(Debug, Clone)]
enum Steps {
    A,
    B,
    C,
    D,
    AB,
    BC,
    CD,
    DA,
}

pub enum StepType {
    Step8, // 4096 steps = 360°
    Step4, // 2048 steps = 360°
}

enum StepSeq {
    Step8([Steps; 8]),
    Step4([Steps; 4]),
}

impl StepSeq {
    fn get_seq_len(&self) -> usize {
        match self {
            StepSeq::Step8(vec_seq) => vec_seq.len(),
            StepSeq::Step4(vec_seq) => vec_seq.len(),
        }
    }

    fn get_number_steps(&self, angle_deg: f32) -> usize {
        match self {
            StepSeq::Step8(_) => (angle_deg.abs() / STEP_8_ANGLE) as usize,
            StepSeq::Step4(_) => (angle_deg.abs() / STEP_4_ANGLE) as usize,
        }
    }

    fn get_step_delay_us(&self, speed_deg_s: f32) -> u32 {
        match self {
            StepSeq::Step8(_) => ((STEP_8_ANGLE / speed_deg_s.abs()) * 1_000_000.0) as u32,
            StepSeq::Step4(_) => ((STEP_4_ANGLE / speed_deg_s.abs()) * 1_000_000.0) as u32,
        }
    }
}

pub enum RotationDirection {
    Clockwise,
    AntiClockwise,
}

struct StepSeqIterator {
    seq_vec: StepSeq,
    index: usize,
}

impl StepSeqIterator {
    fn new(step_type: StepType) -> Self {
        let seq_vec = match step_type {
            StepType::Step8 => StepSeq::Step8([
                Steps::A,
                Steps::AB,
                Steps::B,
                Steps::BC,
                Steps::C,
                Steps::CD,
                Steps::D,
                Steps::DA,
            ]),
            StepType::Step4 => StepSeq::Step4([Steps::AB, Steps::BC, Steps::CD, Steps::DA]),
        };

        Self { seq_vec, index: 0 }
    }

    fn step(&mut self, direction: &RotationDirection) -> Steps {
        match direction {
            RotationDirection::Clockwise => {
                self.index += 1;

                if self.index == self.seq_vec.get_seq_len() {
                    self.index = 0;
                }
            }
            RotationDirection::AntiClockwise => {
                if self.index == 0 {
                    self.index = self.seq_vec.get_seq_len() - 1;
                } else {
                    self.index -= 1;
                }
            }
        }

        match &self.seq_vec {
            StepSeq::Step8(vec_seq) => vec_seq[self.index].clone(),
            StepSeq::Step4(vec_seq) => vec_seq[self.index].clone(),
        }
    }

    fn get_number_steps(&self, angle: f32) -> usize {
        self.seq_vec.get_number_steps(angle)
    }
    fn get_step_delay_us(&self, speed_deg_s: f32) -> u32 {
        self.seq_vec.get_step_delay_us(speed_deg_s)
    }
}

/// Rotation angle (deg) and speed (deg/s)
pub struct AngleSpeed {
    angle: f32,
    speed: f32,
}
impl AngleSpeed {
    pub fn new(angle: f32, speed: f32) -> Self {
        Self { angle, speed }
    }
}

pub enum RotationAngleSpeed {
    Clockwise(AngleSpeed),
    AntiClockwise(AngleSpeed),
}

pub struct Stepper<PinIn1: PinOps, PinIn2: PinOps, PinIn3: PinOps, PinIn4: PinOps> {
    in_1: Pin<Output, PinIn1>,
    in_2: Pin<Output, PinIn2>,
    in_3: Pin<Output, PinIn3>,
    in_4: Pin<Output, PinIn4>,
    steps_seq: StepSeqIterator,
}

impl<PinIn1: PinOps, PinIn2: PinOps, PinIn3: PinOps, PinIn4: PinOps>
    Stepper<PinIn1, PinIn2, PinIn3, PinIn4>
{
    pub fn new(
        in_1: Pin<Output, PinIn1>,
        in_2: Pin<Output, PinIn2>,
        in_3: Pin<Output, PinIn3>,
        in_4: Pin<Output, PinIn4>,
        step_type: StepType,
    ) -> Self {
        let steps_seq = StepSeqIterator::new(step_type);

        Self {
            in_1,
            in_2,
            in_3,
            in_4,
            steps_seq,
        }
    }

    pub fn step(&mut self, direction: &RotationDirection) {
        match self.steps_seq.step(direction) {
            Steps::A => {
                self.in_1.set_high();
                self.in_2.set_low();
                self.in_3.set_low();
                self.in_4.set_low();
            }
            Steps::B => {
                self.in_1.set_low();
                self.in_2.set_high();
                self.in_3.set_low();
                self.in_4.set_low();
            }
            Steps::C => {
                self.in_1.set_low();
                self.in_2.set_low();
                self.in_3.set_high();
                self.in_4.set_low();
            }
            Steps::D => {
                self.in_1.set_low();
                self.in_2.set_low();
                self.in_3.set_low();
                self.in_4.set_high();
            }
            Steps::AB => {
                self.in_1.set_high();
                self.in_2.set_high();
                self.in_3.set_low();
                self.in_4.set_low();
            }
            Steps::BC => {
                self.in_1.set_low();
                self.in_2.set_high();
                self.in_3.set_high();
                self.in_4.set_low();
            }
            Steps::CD => {
                self.in_1.set_low();
                self.in_2.set_low();
                self.in_3.set_high();
                self.in_4.set_high();
            }
            Steps::DA => {
                self.in_1.set_high();
                self.in_2.set_low();
                self.in_3.set_low();
                self.in_4.set_high();
            }
        }
    }

    pub fn rotate_by_angle(&mut self, angle_speed: RotationAngleSpeed) {
        let (number_steps, step_delay_us, direction) = match angle_speed {
            RotationAngleSpeed::Clockwise(angle_speed) => (
                self.steps_seq.get_number_steps(angle_speed.angle),
                self.steps_seq.get_step_delay_us(angle_speed.speed),
                RotationDirection::Clockwise,
            ),
            RotationAngleSpeed::AntiClockwise(angle_speed) => (
                self.steps_seq.get_number_steps(angle_speed.angle),
                self.steps_seq.get_step_delay_us(angle_speed.speed),
                RotationDirection::AntiClockwise,
            ),
        };

        for _ in 0..number_steps {
            self.step(&direction);

            // change delay later: timer -> non blocking
            arduino_hal::delay_us(step_delay_us);
        }
    }
}
