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
    Step8,
    Step4,
}

enum StepSeq {
    Step8([Steps; 8]),
    Step4([Steps; 4]),
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
}

pub struct Stepper<PinIn1, PinIn2, PinIn3, PinIn4> {
    in_1: PinIn1,
    in_2: PinIn2,
    in_3: PinIn3,
    in_4: PinIn4,
    steps_seq: StepSeqIterator,
}
