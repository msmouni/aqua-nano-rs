use super::{ip::EspIpConfig, EspIp};

#[repr(u8)]
#[derive(Clone)]
pub enum EspWifiMode {
    Sta = 1,
    SoftAp = 2,
    SoftApAndSta = 3,
}

#[derive(Clone)]
pub struct SsidPassword<'cfg> {
    pub ssid: &'cfg str,
    pub password: &'cfg str, // string, range: 8 ~ 64 bytes ASCII
}

#[repr(u8)]
#[derive(Clone)]
pub enum WifiEncryption {
    Open = 0,
    WpaPsk = 2,
    Wpa2Psk = 3,
    /// Both WPA PSK and WPA2 PSK encryption modes.
    WpaWpa2Psk = 4,
}

//TMP: Clone
#[derive(Clone)]
pub struct EspApConfig<'cfg> {
    pub wifi: SsidPassword<'cfg>,
    pub chanel_id: u8,
    pub encryption: WifiEncryption,
    pub max_sta_nb: u8, // maximum count of stations that allowed to connect to ESP8266 soft-AP: range: [1, 4]
    pub hide_ssid: bool,
}

pub enum EspWifiConfig<'cfg> {
    Sta {
        ssid_password: SsidPassword<'cfg>,
        ip: EspIpConfig,
        tcp_port: u16,
    },
    Ap {
        ap_config: EspApConfig<'cfg>,
        ip: EspIpConfig,
        tcp_port: u16,
    },
    ApSta {
        sta_config: SsidPassword<'cfg>,
        sta_ip: EspIpConfig,
        ap_config: EspApConfig<'cfg>,
        ap_ip: EspIpConfig,
        tcp_port: u16,
    },
}

impl<'cfg> EspWifiConfig<'cfg> {
    pub fn get_mode(&self) -> EspWifiMode {
        match self {
            EspWifiConfig::Sta { .. } => EspWifiMode::Sta,
            EspWifiConfig::Ap { .. } => EspWifiMode::SoftAp,
            EspWifiConfig::ApSta { .. } => EspWifiMode::SoftApAndSta,
        }
    }

    pub fn get_sta_wifi_config(&self) -> Option<&SsidPassword> {
        match self {
            EspWifiConfig::Sta { ssid_password, .. } => Some(ssid_password),
            EspWifiConfig::Ap { .. } => None,
            EspWifiConfig::ApSta { sta_config, .. } => Some(sta_config),
        }
    }

    pub fn get_ap_wifi_config(&self) -> Option<&EspApConfig> {
        match self {
            EspWifiConfig::Sta { .. } => None,
            EspWifiConfig::Ap { ap_config, .. } => Some(ap_config),
            EspWifiConfig::ApSta { ap_config, .. } => Some(ap_config),
        }
    }

    pub fn get_sta_ip(&self) -> Option<&EspIp> {
        match self {
            EspWifiConfig::Sta { ip, .. } => {
                if let EspIpConfig::Static { ip } = ip {
                    Some(ip)
                } else {
                    None
                }
            }
            EspWifiConfig::Ap { .. } => None,
            EspWifiConfig::ApSta { sta_ip, .. } => {
                if let EspIpConfig::Static { ip } = sta_ip {
                    Some(ip)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_ap_ip(&self) -> Option<&EspIp> {
        match self {
            EspWifiConfig::Sta { .. } => None,
            EspWifiConfig::Ap { ip, .. } => {
                if let EspIpConfig::Static { ip } = ip {
                    Some(ip)
                } else {
                    None
                }
            }
            EspWifiConfig::ApSta { ap_ip, .. } => {
                if let EspIpConfig::Static { ip } = ap_ip {
                    Some(ip)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_tcp_server_port(&self) -> u16 {
        match self {
            EspWifiConfig::Sta {
                ssid_password,
                ip,
                tcp_port,
            } => *tcp_port,
            EspWifiConfig::Ap {
                ap_config,
                ip,
                tcp_port,
            } => *tcp_port,
            EspWifiConfig::ApSta {
                sta_config,
                sta_ip,
                ap_config,
                ap_ip,
                tcp_port,
            } => *tcp_port,
        }
    }
}
