pub trait GetTime {
    fn now_us(&self) -> u64;
}
