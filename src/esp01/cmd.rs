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
        }
    }

    // pub fn process(&mut self)
}

pub struct EspCmdHandler {
    t_cmd_sent: Option<u64>,
}

impl EspCmdHandler {
    const CMD_RESP_TIMEOUT_US: u64 = 10_000_000; // Note: ConfigSta for example takes more time

    pub fn process_cmd<'cmd, EspSerialHandler: EspSerial, const RESP_SZ: usize, Timer: GetTime>(
        &mut self,
        cmd: EspCmd<'cmd>,
        serial_h: &mut EspSerialHandler,
        response_handler: &mut EspRespHandler<RESP_SZ>,
        timer: &Timer,
        check_on_is_ready: bool,
    ) -> bool {
        let t_us = timer.now_us();

        if let Some(t_sent) = self.t_cmd_sent {
            if t_us.wrapping_sub(t_sent) > Self::CMD_RESP_TIMEOUT_US {
                self.t_cmd_sent = None;
            } else if let Some(resp) = response_handler.poll() {
                match resp {
                    EspResp::Ok => {
                        return true;
                    }
                    EspResp::Ready => {
                        if check_on_is_ready {
                            return true;
                        }
                    }
                    EspResp::Error | EspResp::Fail => {
                        self.t_cmd_sent = None;
                    }
                    _ => {}
                }
            }
        } else {
            cmd.send(serial_h);
            self.t_cmd_sent.replace(t_us);
        }

        false
    }
}
