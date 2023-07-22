use super::{clients::ClientMessage, cmd::EspCmd, EspResp, EspRespHandler, EspSerial};

pub enum EspRespType {
    Received(EspResp),
    Timeout,
}

enum AtComState<'a> {
    Idle,
    WaitForResp {
        t_start_wait: u64,
    },
    ProcessingCmd {
        t_cmd_sent: Option<u64>,
        cmd: EspCmd<'a>,
    },
    WrittingMsg {
        t_msg_sent: u64,
    },
}

pub struct EspAtCom<'a, const MSG_SZ: usize, EspSerialHandler: EspSerial> {
    serial_handler: EspSerialHandler,
    response_handler: EspRespHandler<MSG_SZ>,
    state: AtComState<'a>,
}

impl<'a, const MSG_SZ: usize, EspSerialHandler: EspSerial> EspAtCom<'a, MSG_SZ, EspSerialHandler> {
    const CMD_RESP_TIMEOUT_US: u64 = 10_000_000; // Note: ConfigSta for example takes more time

    pub fn new(serial_handler: EspSerialHandler) -> Self {
        Self {
            serial_handler,
            response_handler: EspRespHandler::default(),
            state: AtComState::Idle,
        }
    }

    pub fn process_cmd(&mut self, cmd: EspCmd<'a>) {
        self.state = AtComState::ProcessingCmd {
            t_cmd_sent: None,
            cmd,
        }
    }

    pub fn wait_for_resp(&mut self, t_us: u64) {
        self.state = AtComState::WaitForResp { t_start_wait: t_us }
    }

    pub fn write_msg(&mut self, msg_buff: &[u8], t_us: u64) {
        if self.serial_handler.write_bytes(msg_buff) {
            self.state = AtComState::WrittingMsg { t_msg_sent: t_us }
        }
    }

    pub fn retry_cmd(&mut self) {
        match &mut self.state {
            AtComState::ProcessingCmd { t_cmd_sent, cmd } => {
                t_cmd_sent.take();
            }
            _ => {}
        }
    }

    pub fn update(&mut self, t_us: u64) -> Option<EspRespType> {
        match &mut self.state {
            AtComState::Idle => None,
            AtComState::WaitForResp { t_start_wait } => {
                if t_us.wrapping_sub(*t_start_wait) > Self::CMD_RESP_TIMEOUT_US {
                    Some(EspRespType::Timeout)
                } else {
                    None
                }
            }
            AtComState::ProcessingCmd { t_cmd_sent, cmd } => {
                if let Some(t_sent) = t_cmd_sent {
                    if t_us.wrapping_sub(*t_sent) > Self::CMD_RESP_TIMEOUT_US {
                        *t_cmd_sent = None;
                        None
                    } else if let Some(resp) = self.response_handler.poll() {
                        Some(EspRespType::Received(resp))
                    } else {
                        None
                    }
                } else {
                    cmd.send(&mut self.serial_handler);
                    // self.serial_handler.write_fmt(cmd_args.clone());

                    t_cmd_sent.replace(t_us);

                    None
                }
            }
            AtComState::WrittingMsg { t_msg_sent } => {
                if t_us.wrapping_sub(*t_msg_sent) > Self::CMD_RESP_TIMEOUT_US {
                    Some(EspRespType::Timeout)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_client_next_msg(&mut self, client_id: u8) -> Option<ClientMessage<MSG_SZ>> {
        self.response_handler.get_client_next_msg(client_id)
    }
}
