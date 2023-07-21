mod clients;
mod cmd;
mod ip;
mod resp;
mod serial;
mod state;
mod time;
mod wifi;

use self::{clients::MAX_CLIENT_NB, cmd::EspCmd};
use arrayvec::ArrayVec;
pub use ip::{EspIp, EspIpConfig};
pub use resp::{EspResp, EspRespHandler};
pub use serial::EspSerial;
use state::EspState;
pub use time::GetTime;
pub use wifi::{EspApConfig, EspWifiConfig, EspWifiMode, SsidPassword, WifiEncryption};

const CMD_RESP_TIMEOUT_US: u64 = 10_000_000; // Note: ConfigSta for example takes more time

// TODO: WifiConfig + StaIp + ApIp
pub struct EspWifiHandler<'h, const MSG_SZ: usize, EspSerialHandler: EspSerial> {
    serial_handler: EspSerialHandler,
    response_handler: EspRespHandler<MSG_SZ>,
    state: EspState<'h>,
    config: EspWifiConfig<'h>,
    sta_connected: bool,
    sta_got_ip: bool,
    connected_client: ArrayVec<u8, MAX_CLIENT_NB>,
    send_buff: [u8; MSG_SZ],
}

impl<'h, const MSG_SZ: usize, EspSerialHandler: EspSerial>
    EspWifiHandler<'h, MSG_SZ, EspSerialHandler>
{
    pub fn new(esp_serial: EspSerialHandler, config: EspWifiConfig<'h>) -> Self {
        Self {
            serial_handler: esp_serial,
            response_handler: Default::default(),
            state: EspState::Idle,
            config,
            sta_connected: false,
            sta_got_ip: false,
            connected_client: ArrayVec::new(),
            send_buff: [0u8; MSG_SZ],
        }
    }

    // TODO: Factorize
    pub fn update<Timer: GetTime>(&mut self, timer: &Timer) -> bool {
        match &mut self.state {
            EspState::Idle => {
                self.state = EspState::Reset {
                    t_cmd_sent: None,
                    cmd: EspCmd::Reset,
                };
            }
            EspState::Reset { t_cmd_sent, cmd } => {
                let t_us = timer.now_us();

                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                    } else if let Some(resp) = self.response_handler.poll() {
                        match resp {
                            EspResp::Ok => {
                                self.state = EspState::WaitReady {
                                    t_start_wait: timer.now_us(),
                                };
                                return true;
                            }
                            EspResp::Ready => {
                                // Note: When Reset the Esp-01 sends too many bytes at once, so the "OK" resp could be overwritten
                                self.state = EspState::ConfigWifiMode {
                                    t_cmd_sent: None,
                                    cmd: EspCmd::WifiMode(self.config.get_mode()),
                                };
                                return true;
                            }
                            EspResp::Error | EspResp::Fail => {
                                *t_cmd_sent = None;
                            }
                            _ => {}
                        }
                    }
                } else {
                    EspCmd::Reset.send(&mut self.serial_handler);
                    // self.serial_handler.write_bytes(RST_CMD);
                    t_cmd_sent.replace(t_us);
                }
            }
            EspState::WaitReady { t_start_wait } => {
                let t_us = timer.now_us();

                if t_us.wrapping_sub(*t_start_wait) > CMD_RESP_TIMEOUT_US {
                    self.state = EspState::Reset {
                        t_cmd_sent: None,
                        cmd: EspCmd::Reset,
                    };
                } else if let Some(resp) = self.response_handler.poll() {
                    match resp {
                        EspResp::Ready => {
                            self.state = EspState::ConfigWifiMode {
                                t_cmd_sent: None,
                                cmd: EspCmd::WifiMode(self.config.get_mode()),
                            };
                            return true;
                        }
                        EspResp::Error | EspResp::Fail => {
                            self.state = EspState::Reset {
                                t_cmd_sent: None,
                                cmd: EspCmd::Reset,
                            };
                        }
                        _ => {}
                    }
                }
            }
            EspState::ConfigWifiMode { t_cmd_sent, cmd } => {
                if Self::process_cmd(
                    &mut self.serial_handler,
                    &mut self.response_handler,
                    &mut self.sta_connected,
                    &mut self.sta_got_ip,
                    cmd,
                    t_cmd_sent,
                    timer,
                ) {
                    match &self.config {
                        EspWifiConfig::Sta {
                            ssid_password,
                            ip,
                            tcp_port,
                        } => {
                            self.sta_connected = false;
                            self.sta_got_ip = false;
                            self.state = EspState::ConfigSta {
                                t_cmd_sent: None,
                                cmd: EspCmd::StaJoinAp {
                                    ap_config: ssid_password.clone(),
                                },
                            };
                        }
                        EspWifiConfig::Ap {
                            ap_config,
                            ip,
                            tcp_port,
                        } => {
                            self.state = EspState::ConfigAP {
                                t_cmd_sent: None,
                                cmd: EspCmd::ApConfig {
                                    config: ap_config.clone(),
                                },
                            };
                        }
                        EspWifiConfig::ApSta {
                            sta_config,
                            ap_config,
                            sta_ip,
                            ap_ip,
                            tcp_port,
                        } => {
                            self.sta_connected = false;
                            self.sta_got_ip = false;
                            self.state = EspState::ConfigSta {
                                t_cmd_sent: None,
                                cmd: EspCmd::StaJoinAp {
                                    ap_config: sta_config.clone(),
                                },
                            };
                        }
                    }

                    return true;
                }
            }
            EspState::ConfigSta { t_cmd_sent, cmd } => {
                if Self::process_cmd(
                    &mut self.serial_handler,
                    &mut self.response_handler,
                    &mut self.sta_connected,
                    &mut self.sta_got_ip,
                    cmd,
                    t_cmd_sent,
                    timer,
                ) {
                    if self.sta_connected {
                        match &self.config {
                            EspWifiConfig::Sta {
                                ssid_password,
                                ip,
                                tcp_port,
                            } => match ip {
                                EspIpConfig::Dhcp => {
                                    if self.sta_got_ip {
                                        self.state = EspState::EnablingMultiConx {
                                            t_cmd_sent: None,
                                            cmd: EspCmd::EnableMultiCnx,
                                        };
                                    } else {
                                        self.state = EspState::WaitStaGotIp {
                                            t_start_wait: timer.now_us(),
                                        }
                                    }
                                }
                                EspIpConfig::Static { ip } => {
                                    self.state = EspState::StaIp {
                                        t_cmd_sent: None,
                                        cmd: EspCmd::StaIpConfig {
                                            ip_config: ip.clone(),
                                        },
                                    }
                                }
                            },
                            EspWifiConfig::Ap { .. } => {}
                            EspWifiConfig::ApSta {
                                sta_config,
                                sta_ip,
                                ap_config,
                                ap_ip,
                                tcp_port,
                            } => match sta_ip {
                                EspIpConfig::Dhcp => {
                                    if self.sta_got_ip {
                                        self.state = EspState::ConfigAP {
                                            t_cmd_sent: None,
                                            cmd: EspCmd::ApConfig {
                                                config: ap_config.clone(),
                                            },
                                        };
                                    } else {
                                        self.state = EspState::WaitStaGotIp {
                                            t_start_wait: timer.now_us(),
                                        }
                                    }
                                }
                                EspIpConfig::Static { .. } => {
                                    self.state = EspState::ConfigAP {
                                        t_cmd_sent: None,
                                        cmd: EspCmd::ApConfig {
                                            config: ap_config.clone(),
                                        },
                                    };
                                }
                            },
                        }
                    } else {
                        self.state = EspState::WaitStaConnected {
                            t_start_wait: timer.now_us(),
                        };
                    }

                    return true;
                }
            }
            EspState::WaitStaConnected { t_start_wait } => {
                let t_us = timer.now_us();

                if t_us.wrapping_sub(*t_start_wait) > CMD_RESP_TIMEOUT_US {
                    match &self.config {
                        EspWifiConfig::Sta {
                            ssid_password,
                            ip,
                            tcp_port,
                        } => {
                            self.state = EspState::ConfigSta {
                                t_cmd_sent: None,
                                cmd: EspCmd::StaJoinAp {
                                    ap_config: ssid_password.clone(),
                                },
                            }
                        }
                        EspWifiConfig::Ap { .. } => {}
                        EspWifiConfig::ApSta {
                            sta_config,
                            sta_ip,
                            ap_config,
                            ap_ip,
                            tcp_port,
                        } => {
                            self.state = EspState::ConfigSta {
                                t_cmd_sent: None,
                                cmd: EspCmd::StaJoinAp {
                                    ap_config: sta_config.clone(),
                                },
                            }
                        }
                    }
                } else if let Some(resp) = self.response_handler.poll() {
                    match resp {
                        EspResp::StaConnected => {
                            self.sta_connected = true;

                            self.state = EspState::WaitStaGotIp {
                                t_start_wait: timer.now_us(),
                            };

                            return true;
                        }
                        EspResp::StaGotIp => {
                            self.sta_got_ip = true;
                        }
                        EspResp::Error | EspResp::Fail => {
                            // TODO: RESET ...
                            self.state = EspState::Reset {
                                t_cmd_sent: None,
                                cmd: EspCmd::Reset,
                            };
                        }
                        _ => {}
                    }
                }
            }
            EspState::WaitStaGotIp { t_start_wait } => {
                let t_us = timer.now_us();

                if t_us.wrapping_sub(*t_start_wait) > CMD_RESP_TIMEOUT_US {
                    match &self.config {
                        EspWifiConfig::Sta {
                            ssid_password,
                            ip,
                            tcp_port,
                        } => {
                            self.state = EspState::ConfigSta {
                                t_cmd_sent: None,
                                cmd: EspCmd::StaJoinAp {
                                    ap_config: ssid_password.clone(),
                                },
                            }
                        }
                        EspWifiConfig::Ap { .. } => {}
                        EspWifiConfig::ApSta {
                            sta_config,
                            sta_ip,
                            ap_config,
                            ap_ip,
                            tcp_port,
                        } => {
                            self.state = EspState::ConfigSta {
                                t_cmd_sent: None,
                                cmd: EspCmd::StaJoinAp {
                                    ap_config: sta_config.clone(),
                                },
                            }
                        }
                    }
                } else if let Some(resp) = self.response_handler.poll() {
                    match resp {
                        EspResp::StaGotIp => {
                            self.sta_got_ip = true;
                            match &self.config {
                                EspWifiConfig::Sta {
                                    ssid_password,
                                    ip,
                                    tcp_port,
                                } => {
                                    self.state = EspState::EnablingMultiConx {
                                        t_cmd_sent: None,
                                        cmd: EspCmd::EnableMultiCnx,
                                    };
                                }
                                EspWifiConfig::Ap { .. } => {}
                                EspWifiConfig::ApSta {
                                    sta_config,
                                    sta_ip,
                                    ap_config,
                                    ap_ip,
                                    tcp_port,
                                } => {
                                    self.state = EspState::ConfigAP {
                                        t_cmd_sent: None,
                                        cmd: EspCmd::ApConfig {
                                            config: ap_config.clone(),
                                        },
                                    };
                                }
                            }
                        }
                        EspResp::Error | EspResp::Fail => {
                            // TODO: RESET ...
                            self.state = EspState::Reset {
                                t_cmd_sent: None,
                                cmd: EspCmd::Reset,
                            };
                        }
                        _ => {}
                    }
                }
            }
            EspState::ConfigAP { t_cmd_sent, cmd } => {
                if Self::process_cmd(
                    &mut self.serial_handler,
                    &mut self.response_handler,
                    &mut self.sta_connected,
                    &mut self.sta_got_ip,
                    cmd,
                    t_cmd_sent,
                    timer,
                ) {
                    match &self.config {
                        EspWifiConfig::Sta {
                            ssid_password,
                            ip,
                            tcp_port,
                        } => {}
                        EspWifiConfig::Ap {
                            ap_config,
                            ip,
                            tcp_port,
                        } => match ip {
                            EspIpConfig::Dhcp => {
                                self.state = EspState::EnablingMultiConx {
                                    t_cmd_sent: None,
                                    cmd: EspCmd::EnableMultiCnx,
                                }
                            }
                            EspIpConfig::Static { ip } => {
                                self.state = EspState::ApIp {
                                    t_cmd_sent: None,
                                    cmd: EspCmd::ApIpConfig {
                                        ip_config: ip.clone(),
                                    },
                                };
                            }
                        },
                        EspWifiConfig::ApSta {
                            sta_config,
                            ap_config,
                            sta_ip,
                            ap_ip,
                            tcp_port,
                        } => match sta_ip {
                            EspIpConfig::Dhcp => match ap_ip {
                                EspIpConfig::Dhcp => {
                                    self.state = EspState::EnablingMultiConx {
                                        t_cmd_sent: None,
                                        cmd: EspCmd::EnableMultiCnx,
                                    }
                                }
                                EspIpConfig::Static { ip } => {
                                    self.state = EspState::ApIp {
                                        t_cmd_sent: None,
                                        cmd: EspCmd::ApIpConfig {
                                            ip_config: ip.clone(),
                                        },
                                    }
                                }
                            },
                            EspIpConfig::Static { ip } => {
                                self.state = EspState::StaIp {
                                    t_cmd_sent: None,
                                    cmd: EspCmd::StaIpConfig {
                                        ip_config: ip.clone(),
                                    },
                                }
                            }
                        },
                    }

                    return true;
                }
            }
            EspState::StaIp { t_cmd_sent, cmd } => {
                if Self::process_cmd(
                    &mut self.serial_handler,
                    &mut self.response_handler,
                    &mut self.sta_connected,
                    &mut self.sta_got_ip,
                    cmd,
                    t_cmd_sent,
                    timer,
                ) {
                    match &self.config {
                        EspWifiConfig::Sta {
                            ssid_password,
                            ip,
                            tcp_port,
                        } => {
                            self.state = EspState::EnablingMultiConx {
                                t_cmd_sent: None,
                                cmd: EspCmd::EnableMultiCnx,
                            };
                        }
                        EspWifiConfig::Ap { .. } => {}
                        EspWifiConfig::ApSta {
                            sta_config,
                            sta_ip,
                            ap_config,
                            ap_ip,
                            tcp_port,
                        } => match ap_ip {
                            EspIpConfig::Dhcp => {
                                self.state = EspState::EnablingMultiConx {
                                    t_cmd_sent: None,
                                    cmd: EspCmd::EnableMultiCnx,
                                }
                            }
                            EspIpConfig::Static { ip } => {
                                self.state = EspState::ApIp {
                                    t_cmd_sent: None,
                                    cmd: EspCmd::ApIpConfig {
                                        ip_config: ip.clone(),
                                    },
                                }
                            }
                        },
                    }
                    return true;
                }
            }
            EspState::ApIp { t_cmd_sent, cmd } => {
                if Self::process_cmd(
                    &mut self.serial_handler,
                    &mut self.response_handler,
                    &mut self.sta_connected,
                    &mut self.sta_got_ip,
                    cmd,
                    t_cmd_sent,
                    timer,
                ) {
                    match &self.config {
                        EspWifiConfig::Sta { .. } => {}
                        EspWifiConfig::Ap { .. } | EspWifiConfig::ApSta { .. } => {
                            self.state = EspState::EnablingMultiConx {
                                t_cmd_sent: None,
                                cmd: EspCmd::EnableMultiCnx,
                            }
                        }
                    }
                    return true;
                }
            }
            EspState::EnablingMultiConx { t_cmd_sent, cmd } => {
                if Self::process_cmd(
                    &mut self.serial_handler,
                    &mut self.response_handler,
                    &mut self.sta_connected,
                    &mut self.sta_got_ip,
                    cmd,
                    t_cmd_sent,
                    timer,
                ) {
                    match &self.config {
                        EspWifiConfig::Sta {
                            ssid_password,
                            ip,
                            tcp_port,
                        } => {
                            self.state = EspState::StartingTcpIpServer {
                                t_cmd_sent: None,
                                cmd: EspCmd::CreateTcpServer {
                                    server_port: *tcp_port,
                                },
                            }
                        }
                        EspWifiConfig::Ap {
                            ap_config,
                            ip,
                            tcp_port,
                        } => {
                            self.state = EspState::StartingTcpIpServer {
                                t_cmd_sent: None,
                                cmd: EspCmd::CreateTcpServer {
                                    server_port: *tcp_port,
                                },
                            }
                        }
                        EspWifiConfig::ApSta {
                            sta_config,
                            sta_ip,
                            ap_config,
                            ap_ip,
                            tcp_port,
                        } => {
                            self.state = EspState::StartingTcpIpServer {
                                t_cmd_sent: None,
                                cmd: EspCmd::CreateTcpServer {
                                    server_port: *tcp_port,
                                },
                            }
                        }
                    }
                    return true;
                }
            }
            EspState::StartingTcpIpServer { t_cmd_sent, cmd } => {
                if Self::process_cmd(
                    &mut self.serial_handler,
                    &mut self.response_handler,
                    &mut self.sta_connected,
                    &mut self.sta_got_ip,
                    cmd,
                    t_cmd_sent,
                    timer,
                ) {
                    self.state = EspState::Ready;
                    return true;
                }
            }
            EspState::Ready => {
                if let Some(resp) = self.response_handler.poll() {
                    match resp {
                        EspResp::ClientConnected(client_id) => {
                            self.connected_client.push(client_id);
                            return true;
                        }
                        EspResp::ClientDisconnected(client_id) => {
                            if let Ok(idx_rm) = self
                                .connected_client
                                .binary_search_by(|clt_id| clt_id.cmp(&client_id))
                            {
                                self.connected_client.pop_at(idx_rm);
                            }
                            return true;
                        }
                        EspResp::ClientMsg(client_id) => {
                            if let Some(client_msg) =
                                self.response_handler.get_client_next_msg(client_id)
                            {
                                // hadle msg
                                // TMP: Echo
                                self.send_buff_to_client(client_id, client_msg.get_buff_slice());
                            }
                            return true;
                        }
                        // Handle later:
                        // EspResp::StaDisconnected => todo!(),
                        // EspResp::Error => todo!(),
                        // EspResp::Fail => todo!(),
                        _ => {}
                    }
                }
            }
            EspState::SendingMsg {
                t_cmd_sent,
                cmd,
                ready_to_send,
            } => {
                if *ready_to_send {
                    let t_us = timer.now_us();

                    if let Some(t_sent) = t_cmd_sent {
                        if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                            *t_cmd_sent = None;
                        } else if let Some(resp) = self.response_handler.poll() {
                            match resp {
                                EspResp::MsgSent => {
                                    self.state = EspState::Ready;
                                    return true;
                                }
                                EspResp::Error | EspResp::Fail => {
                                    *ready_to_send = false;
                                }
                                _ => {}
                            }
                        }
                    } else {
                        self.serial_handler.write_bytes(&self.send_buff);
                        t_cmd_sent.replace(t_us);
                    }
                } else if Self::process_cmd(
                    &mut self.serial_handler,
                    &mut self.response_handler,
                    &mut self.sta_connected,
                    &mut self.sta_got_ip,
                    cmd,
                    t_cmd_sent,
                    timer,
                ) {
                    *ready_to_send = true;
                    return true;
                }
            }
        }

        false
    }

    // To adjust later: seperate correctly ...
    fn process_cmd<'cmd, Timer: GetTime>(
        serial_handler: &mut EspSerialHandler,
        response_handler: &mut EspRespHandler<MSG_SZ>,
        sta_connected: &mut bool,
        sta_got_ip: &mut bool,
        cmd: &EspCmd<'cmd>,
        t_cmd_sent: &mut Option<u64>,
        timer: &Timer,
    ) -> bool {
        let t_us = timer.now_us();

        if let Some(t_sent) = t_cmd_sent {
            if t_us.wrapping_sub(*t_sent) > CMD_RESP_TIMEOUT_US {
                *t_cmd_sent = None;
            } else if let Some(resp) = response_handler.poll() {
                match resp {
                    EspResp::Ok => {
                        *t_cmd_sent = None;
                        return true;
                    }
                    EspResp::Error | EspResp::Fail => {
                        *t_cmd_sent = None;
                    }
                    EspResp::StaConnected => {
                        *sta_connected = true;
                    }
                    EspResp::StaGotIp => {
                        *sta_got_ip = true;
                    }
                    EspResp::StaDisconnected => {
                        *sta_connected = false;
                        *sta_got_ip = false;
                    }
                    _ => {}
                }
            }
        } else {
            cmd.send(serial_handler);
            t_cmd_sent.replace(t_us);
        }

        false
    }
    /*
    fn send(&mut self){
        /*
        AT+CIPSEND=1,4

        OK
        >

        busy s...

        Recv 4 bytes

        SEND OK
        AT+CIPSEND=1,4

        OK
        >

        busy s...

        Recv 4 bytes

        SEND OK

         */
    }
     */

    pub fn send_buff_to_client(&mut self, client_id: u8, buff: &[u8]) {
        let buff_len = buff.len();

        self.send_buff[..buff_len].copy_from_slice(buff);

        self.state = EspState::SendingMsg {
            t_cmd_sent: None,
            cmd: EspCmd::StartMsgSend {
                client_id,
                msg_len: buff_len as u8,
            },
            ready_to_send: false,
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, EspState::Ready)
    }
}
