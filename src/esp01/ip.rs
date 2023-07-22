#[derive(Clone)]
pub struct EspIp<'ip> {
    pub ip: &'ip str,
    pub gw: &'ip str,
    pub mask: &'ip str,
}

pub enum EspIpConfig<'ip> {
    Dhcp,
    Static { ip: EspIp<'ip> },
}
