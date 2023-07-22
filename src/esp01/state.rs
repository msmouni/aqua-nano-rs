use core::fmt::Arguments;

use super::{
    cmd::EspCmd, com::EspRespType, EspIpConfig, EspResp, EspRespHandler, EspSerial, EspWifiConfig,
    GetTime,
};

pub enum EspState {
    Reset,
    WaitReady,
    ConfigWifiMode,
    ConfigSta,
    WaitStaConnected,
    ConfigAP,
    StaIp,
    WaitStaGotIp,
    ApIp,
    EnablingMultiConx,
    StartingTcpIpServer,
    SendingMsg { client_id: u8, buff_len: u8 },
    Ready,
}

// To rename maybe
pub enum StateEvent {
    None,
    CmdRequest,
    CmdFailed,
    WaitForResp,
    ClientConnected { client_id: u8 },
    ClientMsg { client_id: u8 },
    ClientDisconnected { client_id: u8 },
    ReadyTosendMsg,
    MsgSent,
}

// size = 184 : Config:176
pub struct EspStateHandler {
    state: EspState, // 3
    // config: EspWifiConfig<'h>, // 176
    sta_connected: bool,
    sta_got_ip: bool,
}

impl EspStateHandler {
    pub fn new() -> Self {
        Self {
            state: EspState::Reset,
            // config,
            sta_connected: false,
            sta_got_ip: false,
        }
    }
    pub fn update(&mut self, resp: &EspRespType, config: &EspWifiConfig) -> StateEvent {
        //-> StateEvent {
        match &self.state {
            EspState::Reset => {
                match resp {
                    EspRespType::Received(EspResp::Ok) => {
                        self.state = EspState::WaitReady;
                        StateEvent::WaitForResp
                    }
                    EspRespType::Received(EspResp::Ready) => {
                        // Note: When Reset the Esp-01 sends too many bytes at once, so the "OK" resp could be overwritten
                        self.state = EspState::ConfigWifiMode;

                        StateEvent::CmdRequest
                    }
                    EspRespType::Received(EspResp::Error)
                    | EspRespType::Received(EspResp::Fail) => StateEvent::CmdFailed,
                    _ => StateEvent::None,
                }
            }
            EspState::WaitReady => match resp {
                EspRespType::Received(EspResp::Ready) => {
                    self.state = EspState::ConfigWifiMode;
                    return StateEvent::CmdRequest;
                }
                EspRespType::Received(EspResp::Error)
                | EspRespType::Received(EspResp::Fail)
                | EspRespType::Timeout => {
                    self.state = EspState::Reset;
                    StateEvent::CmdFailed
                }
                _ => StateEvent::None,
            },
            EspState::ConfigWifiMode
            | EspState::ConfigSta
            | EspState::WaitStaConnected
            | EspState::ConfigAP
            | EspState::StaIp
            | EspState::WaitStaGotIp
            | EspState::ApIp => match &resp {
                EspRespType::Received(EspResp::Ok) => match config {
                    EspWifiConfig::Sta {
                        ssid_password,
                        ip,
                        tcp_port,
                    } => match &self.state {
                        EspState::ConfigWifiMode => {
                            self.state = EspState::ConfigSta;
                            self.sta_connected = false;
                            self.sta_got_ip = false;
                            StateEvent::CmdRequest
                        }
                        EspState::ConfigSta => {
                            if self.sta_connected {
                                match ip {
                                    EspIpConfig::Dhcp => {
                                        if self.sta_got_ip {
                                            self.state = EspState::EnablingMultiConx;
                                            StateEvent::CmdRequest
                                        } else {
                                            self.state = EspState::WaitStaGotIp;
                                            StateEvent::WaitForResp
                                        }
                                    }
                                    EspIpConfig::Static { ip } => {
                                        self.state = EspState::StaIp;
                                        StateEvent::CmdRequest
                                    }
                                }
                            } else {
                                self.state = EspState::WaitStaConnected;
                                StateEvent::WaitForResp
                            }
                        }
                        EspState::StaIp => {
                            self.state = EspState::EnablingMultiConx;
                            StateEvent::CmdRequest
                        }
                        _ => StateEvent::None,
                    },
                    EspWifiConfig::Ap {
                        ap_config,
                        ip,
                        tcp_port,
                    } => match &self.state {
                        EspState::ConfigWifiMode => {
                            self.state = EspState::ConfigAP;
                            StateEvent::CmdRequest
                        }
                        EspState::ConfigAP => match ip {
                            EspIpConfig::Dhcp => {
                                self.state = EspState::EnablingMultiConx;
                                StateEvent::CmdRequest
                            }
                            EspIpConfig::Static { ip } => {
                                self.state = EspState::ApIp;
                                StateEvent::CmdRequest
                            }
                        },
                        EspState::ApIp => {
                            self.state = EspState::EnablingMultiConx;
                            StateEvent::CmdRequest
                        }
                        _ => StateEvent::None,
                    },
                    EspWifiConfig::ApSta {
                        sta_config,
                        sta_ip,
                        ap_config,
                        ap_ip,
                        tcp_port,
                    } => match &self.state {
                        EspState::ConfigWifiMode => {
                            self.state = EspState::ConfigSta;
                            self.sta_connected = false;
                            self.sta_got_ip = false;
                            StateEvent::CmdRequest
                        }
                        EspState::ConfigSta => {
                            // To factorize
                            if self.sta_connected {
                                match sta_ip {
                                    EspIpConfig::Dhcp => {
                                        if self.sta_got_ip {
                                            self.state = EspState::EnablingMultiConx;
                                            StateEvent::CmdRequest
                                        } else {
                                            self.state = EspState::WaitStaGotIp;
                                            StateEvent::WaitForResp
                                        }
                                    }
                                    EspIpConfig::Static { ip } => {
                                        self.state = EspState::StaIp;
                                        StateEvent::CmdRequest
                                    }
                                }
                            } else {
                                self.state = EspState::WaitStaConnected;
                                StateEvent::WaitForResp
                            }
                        }
                        EspState::WaitStaConnected => todo!(),
                        EspState::ConfigAP => todo!(),
                        EspState::StaIp => todo!(),
                        EspState::WaitStaGotIp => todo!(),
                        EspState::ApIp => todo!(),
                        _ => StateEvent::None,
                    },
                },
                EspRespType::Timeout => match &self.state {
                    EspState::WaitStaConnected | EspState::WaitStaGotIp => {
                        self.state = EspState::ConfigSta;
                        StateEvent::CmdFailed
                    }
                    _ => StateEvent::None,
                },
                EspRespType::Received(EspResp::StaConnected) => {
                    self.sta_connected = true;

                    if matches!(self.state, EspState::WaitStaConnected) {
                        match config {
                            EspWifiConfig::Sta {
                                ssid_password,
                                ip,
                                tcp_port,
                            } => match ip {
                                EspIpConfig::Dhcp => {
                                    if self.sta_got_ip {
                                        self.state = EspState::EnablingMultiConx;
                                        StateEvent::CmdRequest
                                    } else {
                                        self.state = EspState::WaitStaGotIp;
                                        StateEvent::WaitForResp
                                    }
                                }
                                EspIpConfig::Static { ip } => {
                                    self.state = EspState::StaIp;
                                    StateEvent::CmdRequest
                                }
                            },
                            EspWifiConfig::ApSta {
                                sta_config,
                                sta_ip,
                                ap_config,
                                ap_ip,
                                tcp_port,
                            } => match sta_ip {
                                EspIpConfig::Dhcp => {
                                    if self.sta_got_ip {
                                        self.state = EspState::ConfigAP;
                                        StateEvent::CmdRequest
                                    } else {
                                        self.state = EspState::WaitStaGotIp;
                                        StateEvent::WaitForResp
                                    }
                                }
                                EspIpConfig::Static { ip } => {
                                    self.state = EspState::StaIp;
                                    StateEvent::CmdRequest
                                }
                            },
                            EspWifiConfig::Ap {
                                ap_config,
                                ip,
                                tcp_port,
                            } => StateEvent::None,
                        }
                    } else {
                        StateEvent::None
                    }
                }
                EspRespType::Received(EspResp::StaDisconnected) => {
                    self.sta_connected = false;
                    StateEvent::None
                }
                EspRespType::Received(EspResp::StaGotIp) => {
                    self.sta_got_ip = true;
                    if matches!(self.state, EspState::WaitStaGotIp) {
                        self.state = EspState::EnablingMultiConx;
                        StateEvent::CmdRequest
                    } else {
                        StateEvent::None
                    }
                }
                EspRespType::Received(EspResp::Error) | EspRespType::Received(EspResp::Fail) => {
                    StateEvent::CmdFailed
                }
                _ => StateEvent::None,
            },
            EspState::EnablingMultiConx => match resp {
                EspRespType::Received(EspResp::Ok) => {
                    self.state = EspState::StartingTcpIpServer;
                    StateEvent::CmdRequest
                }
                EspRespType::Received(EspResp::Error) | EspRespType::Received(EspResp::Fail) => {
                    StateEvent::CmdFailed
                }
                _ => StateEvent::None,
            },
            EspState::StartingTcpIpServer => match resp {
                EspRespType::Received(EspResp::Ok) => {
                    self.state = EspState::Ready;
                    StateEvent::None
                }
                EspRespType::Received(EspResp::Error) | EspRespType::Received(EspResp::Fail) => {
                    StateEvent::CmdFailed
                }
                _ => StateEvent::None,
            },
            EspState::Ready => {
                match resp {
                    EspRespType::Received(EspResp::ClientConnected(client_id)) => {
                        StateEvent::ClientConnected {
                            client_id: *client_id,
                        }
                    }
                    EspRespType::Received(EspResp::ClientDisconnected(client_id)) => {
                        StateEvent::ClientDisconnected {
                            client_id: *client_id,
                        }
                    }
                    EspRespType::Received(EspResp::ClientMsg(client_id)) => StateEvent::ClientMsg {
                        client_id: *client_id,
                    },
                    EspRespType::Received(EspResp::StaDisconnected)
                    | EspRespType::Received(EspResp::Error)
                    | EspRespType::Received(EspResp::Fail) => {
                        // To handle better
                        self.state = EspState::Reset;
                        StateEvent::CmdRequest
                    }
                    _ => StateEvent::None,
                }
            }
            EspState::SendingMsg { .. } => match resp {
                EspRespType::Received(EspResp::Ok) => StateEvent::ReadyTosendMsg,
                EspRespType::Received(EspResp::MsgSent) => {
                    self.state = EspState::Ready;
                    StateEvent::MsgSent
                }
                _ => StateEvent::None,
            }, ////////////////////////////////////////////////////////
        }

        // false
    }

    pub fn get_cmd(&self, config: &EspWifiConfig) -> Option<EspCmd> {
        match &self.state {
            EspState::Reset => Some(EspCmd::Reset),
            EspState::WaitReady => None,
            EspState::ConfigWifiMode => Some(EspCmd::WifiMode),
            EspState::ConfigSta => match config {
                EspWifiConfig::Sta { .. } | EspWifiConfig::ApSta { .. } => Some(EspCmd::StaJoinAp),
                EspWifiConfig::Ap { .. } => None,
            },
            EspState::WaitStaConnected => None,
            EspState::ConfigAP => match config {
                EspWifiConfig::Sta { .. } => None,
                EspWifiConfig::Ap { .. } | EspWifiConfig::ApSta { .. } => Some(EspCmd::ApConfig),
            },
            EspState::StaIp => match config {
                EspWifiConfig::Sta { ip, .. } => match ip {
                    EspIpConfig::Dhcp => None,
                    EspIpConfig::Static { ip } => Some(EspCmd::StaIpConfig),
                },
                EspWifiConfig::Ap { .. } => None,
                EspWifiConfig::ApSta { sta_ip, .. } => match sta_ip {
                    EspIpConfig::Dhcp => None,
                    EspIpConfig::Static { ip } => Some(EspCmd::StaIpConfig),
                },
            },
            EspState::WaitStaGotIp => None,
            EspState::ApIp => match config {
                EspWifiConfig::Sta { .. } => None,
                EspWifiConfig::Ap { ip, .. } => match ip {
                    EspIpConfig::Dhcp => None,
                    EspIpConfig::Static { ip } => Some(EspCmd::ApIpConfig),
                },
                EspWifiConfig::ApSta { ap_ip, .. } => match ap_ip {
                    EspIpConfig::Dhcp => None,
                    EspIpConfig::Static { ip } => Some(EspCmd::ApIpConfig),
                },
            },
            EspState::EnablingMultiConx => Some(EspCmd::EnableMultiCnx),
            EspState::StartingTcpIpServer => Some(EspCmd::CreateTcpServer),
            EspState::SendingMsg {
                client_id,
                buff_len,
            } => None,
            EspState::Ready => None,
        }
    }

    pub fn start_msg_send_cmd(&mut self, client_id: u8, buff_len: u8) -> EspCmd {
        self.state = EspState::SendingMsg {
            client_id,
            buff_len,
        };
        EspCmd::StartMsgSend {
            client_id,
            msg_len: buff_len,
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, EspState::Ready)
    }
}
