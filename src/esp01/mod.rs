mod cmd;
mod resp;
mod serial;
mod state;
mod time;

pub use resp::{EspResp, EspRespHandler};
pub use serial::EspSerial;
use state::EspState;
pub use time::GetTime;

use self::cmd::RST_CMD;

const CMD_RESP_TIMEOUT_US: u64 = 10_000_000; // Note: ConfigSta for example takes more time

// TODO: WifiConfig + StaIp + ApIp
pub struct EspWifiHandler<const RESP_SZ: usize, EspSerialHandler: EspSerial> {
    serial_handler: EspSerialHandler,
    response_handler: EspRespHandler<RESP_SZ>,
    state: EspState,
}

impl<const RESP_SZ: usize, EspSerialHandler: EspSerial> EspWifiHandler<RESP_SZ, EspSerialHandler> {
    pub fn new(esp_serial: EspSerialHandler) -> Self {
        Self {
            serial_handler: esp_serial,
            response_handler: Default::default(),
            state: EspState::Idle,
        }
    }

    // TODO: Factorize
    pub fn update<Timer: GetTime>(&mut self, timer: &Timer) -> bool {
        match &mut self.state {
            EspState::Idle => {
                self.state = EspState::Reset { t_cmd_sent: None };
            }
            EspState::Reset { t_cmd_sent } => {
                let t_us = timer.now_us();

                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                    } else {
                        if let Some(resp) = self.response_handler.poll() {
                            match resp {
                                EspResp::Ok => {
                                    self.state = EspState::WaitReady {
                                        t_start_wait: timer.now_us(),
                                    };
                                    return true;
                                }
                                EspResp::Ready => {
                                    // Note: When Reset the Esp-01 sends too many bytes at once, so the "OK" resp could be overwritten
                                    self.state = EspState::ConfigWifiMode { t_cmd_sent: None };
                                    return true;
                                }
                                EspResp::Error | EspResp::Fail => {
                                    *t_cmd_sent = None;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    self.serial_handler.write_bytes(RST_CMD);
                    t_cmd_sent.replace(t_us);
                }
            }
            EspState::WaitReady { t_start_wait } => {
                let t_us = timer.now_us();

                if t_us.wrapping_sub(*t_start_wait) > CMD_RESP_TIMEOUT_US {
                    self.state = EspState::Reset { t_cmd_sent: None };
                } else {
                    if let Some(resp) = self.response_handler.poll() {
                        match resp {
                            EspResp::Ready => {
                                self.state = EspState::ConfigWifiMode { t_cmd_sent: None };
                                return true;
                            }
                            EspResp::Error | EspResp::Fail => {
                                self.state = EspState::Reset { t_cmd_sent: None };
                            }
                            _ => {}
                        }
                    }
                }
            }
            EspState::ConfigWifiMode { t_cmd_sent } => {
                let t_us = timer.now_us();

                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                    } else {
                        if let Some(resp) = self.response_handler.poll() {
                            match resp {
                                EspResp::Ok => {
                                    self.state = EspState::ConfigSta { t_cmd_sent: None };
                                    return true;
                                }
                                EspResp::Error | EspResp::Fail => {
                                    *t_cmd_sent = None;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    self.serial_handler.write_bytes(b"AT+CWMODE=3\r\n"); // 1 : station mode | 2 : softAP mode | 3 : softAP + station mode
                    t_cmd_sent.replace(t_us);
                }
            }
            EspState::ConfigSta { t_cmd_sent } => {
                let t_us = timer.now_us();

                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                    } else {
                        if let Some(resp) = self.response_handler.poll() {
                            match resp {
                                EspResp::Ok => {
                                    self.state = EspState::ConfigAP { t_cmd_sent: None };
                                    return true;
                                }
                                EspResp::Error | EspResp::Fail => {
                                    *t_cmd_sent = None;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    self.serial_handler.write_fmt(format_args!(
                        "AT+CWJAP=\"{}\",\"{}\"\r\n",
                        "SSID", "Password"
                    ));

                    // TODO: store :WIFI DISCONNECT -> WIFI CONNECTED -> WIFI GOT IP (in case we're using DHCP)
                    // TODO: Go next after N attempts
                    t_cmd_sent.replace(t_us);
                }
            }
            EspState::ConfigAP { t_cmd_sent } => {
                let t_us = timer.now_us();

                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                    } else {
                        if let Some(resp) = self.response_handler.poll() {
                            match resp {
                                EspResp::Ok => {
                                    // TODO: If DHCP go to EnablingMultiConx
                                    self.state = EspState::StaIp { t_cmd_sent: None };
                                    return true;
                                }
                                EspResp::Error | EspResp::Fail => {
                                    *t_cmd_sent = None;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    self.serial_handler.write_fmt(format_args!(
                        "AT+CWSAP=\"{}\",\"{}\",{},{}\r\n",
                        "my_ssid", "my_password", 4, 3
                    )); // self.ssid, self.password, self.channel, self.mode as u8, // To complete later with config

                    t_cmd_sent.replace(t_us);
                }
            }
            EspState::StaIp { t_cmd_sent } => {
                let t_us = timer.now_us();

                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                    } else {
                        if let Some(resp) = self.response_handler.poll() {
                            match resp {
                                EspResp::Ok => {
                                    // TODO: If DHCP go to EnablingMultiConx
                                    self.state = EspState::ApIp { t_cmd_sent: None };
                                    return true;
                                }
                                EspResp::Error | EspResp::Fail => {
                                    *t_cmd_sent = None;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    self.serial_handler.write_fmt(format_args!(
                        "AT+CIPSTA=\"{}\",\"{}\",\"{}\"\r\n",
                        "xxx.xxx.xxx.xxx", "xxx.xxx.xxx.xxx", "255.255.255.0"
                    )); // IP, GW, MASK

                    t_cmd_sent.replace(t_us);
                }
            }
            EspState::ApIp { t_cmd_sent } => {
                let t_us = timer.now_us();

                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                    } else {
                        if let Some(resp) = self.response_handler.poll() {
                            match resp {
                                EspResp::Ok => {
                                    self.state = EspState::EnablingMultiConx { t_cmd_sent: None };
                                    return true;
                                }
                                EspResp::Error | EspResp::Fail => {
                                    *t_cmd_sent = None;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    self.serial_handler.write_fmt(format_args!(
                        "AT+CIPAP=\"{}\",\"{}\",\"{}\"\r\n",
                        "xxx.xxx.xxx.xxx", "xxx.xxx.xxx.xxx", "255.255.255.0"
                    )); // IP, GW, MASK

                    t_cmd_sent.replace(t_us);
                }
            }
            EspState::EnablingMultiConx { t_cmd_sent } => {
                let t_us = timer.now_us();

                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                    } else {
                        if let Some(resp) = self.response_handler.poll() {
                            match resp {
                                EspResp::Ok => {
                                    self.state = EspState::StartingTcpIpServer { t_cmd_sent: None };
                                    return true;
                                }
                                EspResp::Error | EspResp::Fail => {
                                    *t_cmd_sent = None;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    self.serial_handler.write_bytes(b"AT+CIPMUX=1\r\n");

                    t_cmd_sent.replace(t_us);
                }
            }
            EspState::StartingTcpIpServer { t_cmd_sent } => {
                let t_us = timer.now_us();

                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                    } else {
                        if let Some(resp) = self.response_handler.poll() {
                            match resp {
                                EspResp::Ok => {
                                    self.state = EspState::Ready;
                                    return true;
                                }
                                EspResp::Error | EspResp::Fail => {
                                    *t_cmd_sent = None;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    self.serial_handler
                        .write_fmt(format_args!("AT+CIPSERVER=1,{}\r\n", 2_000)); // Port

                    t_cmd_sent.replace(t_us);
                }
            }
            EspState::Ready => {}
        }

        false
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, EspState::Ready)
    }
}
