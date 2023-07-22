#[derive(Clone)]
pub struct EspIp {
    pub ip: [u8; 4],
    pub gw: [u8; 4],
    pub mask: [u8; 4],
}

pub enum EspIpConfig {
    Dhcp,
    Static { ip: EspIp },
}
