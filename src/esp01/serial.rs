pub trait EspSerial {
    fn write_bytes(&mut self, bytes: &[u8]) -> bool;
    fn write_str(&mut self, s: &str) -> bool;
    fn write_fmt(&mut self, args: core::fmt::Arguments) -> bool;
}
