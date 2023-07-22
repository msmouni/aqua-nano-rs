use super::{
    ip::EspIp,
    wifi::{EspApConfig, EspWifiMode, SsidPassword},
    EspResp, EspRespHandler, EspSerial, GetTime,
};

// pub(crate) const RST_CMD: &[u8] = b"AT+RST\r\n";

pub enum EspCmd<'cmd> {
    Reset,
    WifiMode(EspWifiMode),
    StaJoinAp { ap_config: SsidPassword<'cmd> },
    ApConfig { config: EspApConfig<'cmd> }, // to test: { config: &'cmd EspApConfig<'cmd> },
    StaIpConfig { ip_config: EspIp<'cmd> },
    ApIpConfig { ip_config: EspIp<'cmd> },
    EnableMultiCnx,
    DisableMultiCnx,
    CreateTcpServer { server_port: u16 },
    DeleteTcpServer { server_port: u16 },
    StartMsgSend { client_id: u8, msg_len: u8 },
}

impl<'cmd> EspCmd<'cmd> {
    pub fn send<EspSerialHandler: EspSerial>(&self, serial_h: &mut EspSerialHandler) -> bool {
        match self {
            EspCmd::Reset => serial_h.write_bytes(b"AT+RST\r\n"),
            EspCmd::WifiMode(wifimode) => {
                serial_h.write_fmt(format_args!("AT+CWMODE={}\r\n", wifimode.clone() as u8))
            }
            EspCmd::StaJoinAp { ap_config } => serial_h.write_fmt(format_args!(
                "AT+CWJAP=\"{}\",\"{}\"\r\n",
                ap_config.ssid, ap_config.password
            )),
            EspCmd::ApConfig { config } => serial_h.write_fmt(format_args!(
                "AT+CWSAP=\"{}\",\"{}\",{},{},{},{}\r\n",
                config.wifi.ssid,
                config.wifi.password,
                config.chanel_id,
                config.encryption.clone() as u8,
                config.max_sta_nb,
                config.hide_ssid as u8
            )),
            EspCmd::StaIpConfig { ip_config } => serial_h.write_fmt(format_args!(
                "AT+CIPSTA=\"{}\",\"{}\",\"{}\"\r\n",
                ip_config.ip, ip_config.gw, ip_config.mask
            )),
            EspCmd::ApIpConfig { ip_config } => serial_h.write_fmt(format_args!(
                "AT+CIPAP=\"{}\",\"{}\",\"{}\"\r\n",
                ip_config.ip, ip_config.gw, ip_config.mask
            )),
            EspCmd::EnableMultiCnx => serial_h.write_bytes(b"AT+CIPMUX=1\r\n"),
            EspCmd::DisableMultiCnx => serial_h.write_bytes(b"AT+CIPMUX=0\r\n"),
            EspCmd::CreateTcpServer { server_port } => {
                serial_h.write_fmt(format_args!("AT+CIPSERVER=1,{}\r\n", server_port))
            }
            EspCmd::DeleteTcpServer { server_port } => {
                serial_h.write_fmt(format_args!("AT+CIPSERVER=0,{}\r\n", server_port))
            }
            EspCmd::StartMsgSend { client_id, msg_len } => {
                serial_h.write_fmt(format_args!("AT+CIPSEND={},{}\r\n", client_id, msg_len))
            }
        }
    }

    // pub fn process(&mut self)
}
