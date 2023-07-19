#![allow(dead_code)]

#[derive(Debug, Clone, Default, Eq, PartialEq, PartialOrd)]
pub struct SysTime {
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    milli_second: u32,
    micro_second: u32,
}
impl SysTime {
    fn add(&mut self, other: &Self) {
        self.day += other.day;
        self.hour += other.hour;
        self.minute += other.minute;
        self.second += other.second;
        self.milli_second += other.milli_second;
        self.micro_second += other.micro_second;
    }
    fn sub(&self, other: &Self) -> Self {
        Self {
            day: self.day - other.day,
            hour: self.hour - other.hour,
            minute: self.minute - other.minute,
            second: self.second - other.second,
            milli_second: self.milli_second - other.milli_second,
            micro_second: self.micro_second - other.micro_second,
        }
    }
    pub fn new(elapsed_micros: u64) -> Self {
        let mut time = Self::default();
        time.compute(elapsed_micros);

        time
    }
    pub fn compute(&mut self, elapsed_micros: u64) {
        let elapsed_millis = elapsed_micros / 1_000;

        let micro_second = elapsed_micros - (elapsed_millis * 1_000);

        let elapsed_sec = elapsed_millis / 1_000;

        let milli_second = elapsed_millis - (elapsed_sec * 1_000);

        let elapsed_min = elapsed_sec / 60;

        let second = elapsed_sec - (elapsed_min * 60);

        let elapsed_hours = elapsed_min / 60;

        let minute = elapsed_min - (elapsed_hours * 60);

        let day = elapsed_hours / 24;

        let hour = elapsed_hours - (day * 24);

        self.add(&Self {
            day: day as u32,
            hour: hour as u32,
            minute: minute as u32,
            second: second as u32,
            milli_second: milli_second as u32,
            micro_second: micro_second as u32,
        })
    }

    pub fn elapsed_since(&self, old_time: &Self) -> Option<Self> {
        if old_time < self {
            Some(self.sub(old_time))
        } else {
            None
        }
    }
}
