use super::{EspSerial, EspWifiConfig};

// pub(crate) const RST_CMD: &[u8] = b"AT+RST\r\n";

pub enum EspCmd {
    Reset,
    WifiMode,
    StaJoinAp,
    ApConfig,
    StaIpConfig,
    ApIpConfig,
    EnableMultiCnx,
    DisableMultiCnx,
    CreateTcpServer,
    DeleteTcpServer,
    StartMsgSend { client_id: u8, msg_len: u8 },
}

impl EspCmd {
    pub fn send<EspSerialHandler: EspSerial>(
        &self,
        serial_h: &mut EspSerialHandler,
        config: &EspWifiConfig,
    ) -> bool {
        match self {
            EspCmd::Reset => serial_h.write_bytes(b"AT+RST\r\n"),
            EspCmd::WifiMode => {
                serial_h.write_fmt(format_args!("AT+CWMODE={}\r\n", config.get_mode() as u8))
            }
            EspCmd::StaJoinAp => {
                if let Some(ap_config) = config.get_sta_wifi_config() {
                    serial_h.write_fmt(format_args!(
                        "AT+CWJAP=\"{}\",\"{}\"\r\n",
                        ap_config.ssid, ap_config.password
                    ))
                } else {
                    false
                }
            }
            EspCmd::ApConfig => {
                if let Some(ap_config) = config.get_ap_wifi_config() {
                    serial_h.write_fmt(format_args!(
                        "AT+CWSAP=\"{}\",\"{}\",{},{},{},{}\r\n",
                        ap_config.wifi.ssid,
                        ap_config.wifi.password,
                        ap_config.chanel_id,
                        ap_config.encryption.clone() as u8,
                        ap_config.max_sta_nb,
                        ap_config.hide_ssid as u8
                    ))
                } else {
                    false
                }
            }
            EspCmd::StaIpConfig => {
                if let Some(ip_config) = config.get_sta_ip() {
                    serial_h.write_fmt(format_args!(
                        "AT+CIPSTA=\"{}.{}.{}.{}\",\"{}.{}.{}.{}\",\"{}.{}.{}.{}\"\r\n",
                        ip_config.ip[0],
                        ip_config.ip[1],
                        ip_config.ip[2],
                        ip_config.ip[3],
                        ip_config.gw[0],
                        ip_config.gw[1],
                        ip_config.gw[2],
                        ip_config.gw[3],
                        ip_config.mask[0],
                        ip_config.mask[1],
                        ip_config.mask[2],
                        ip_config.mask[3]
                    ))
                } else {
                    false
                }
            }
            EspCmd::ApIpConfig => {
                if let Some(ip_config) = config.get_ap_ip() {
                    serial_h.write_fmt(format_args!(
                        "AT+CIPAP=\"{}.{}.{}.{}\",\"{}.{}.{}.{}\",\"{}.{}.{}.{}\"\r\n",
                        ip_config.ip[0],
                        ip_config.ip[1],
                        ip_config.ip[2],
                        ip_config.ip[3],
                        ip_config.gw[0],
                        ip_config.gw[1],
                        ip_config.gw[2],
                        ip_config.gw[3],
                        ip_config.mask[0],
                        ip_config.mask[1],
                        ip_config.mask[2],
                        ip_config.mask[3]
                    ))
                } else {
                    false
                }
            }
            EspCmd::EnableMultiCnx => serial_h.write_bytes(b"AT+CIPMUX=1\r\n"),
            EspCmd::DisableMultiCnx => serial_h.write_bytes(b"AT+CIPMUX=0\r\n"),
            EspCmd::CreateTcpServer => serial_h.write_fmt(format_args!(
                "AT+CIPSERVER=1,{}\r\n",
                config.get_tcp_server_port()
            )),
            EspCmd::DeleteTcpServer => serial_h.write_fmt(format_args!(
                "AT+CIPSERVER=0,{}\r\n",
                config.get_tcp_server_port()
            )),
            EspCmd::StartMsgSend { client_id, msg_len } => {
                serial_h.write_fmt(format_args!("AT+CIPSEND={},{}\r\n", client_id, msg_len))
            }
        }
    }
}
