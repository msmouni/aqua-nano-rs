mod clients;
mod cmd;
mod com;
mod ip;
mod resp;
mod serial;
mod state;
mod time;
mod wifi;

// TMP pub
pub use self::{
    clients::MAX_CLIENT_NB,
    com::EspAtCom,
    state::{EspStateHandler, StateEvent},
};
use arrayvec::ArrayVec;
pub use ip::{EspIp, EspIpConfig};
pub use resp::{EspResp, EspRespHandler};
pub use serial::EspSerial;
pub use time::GetTime;
pub use wifi::{EspApConfig, EspWifiConfig, EspWifiMode, SsidPassword, WifiEncryption};

// 424
pub struct EspWifiHandler<'h, const MSG_SZ: usize, EspSerialHandler: EspSerial> {
    at_com_handler: EspAtCom<MSG_SZ, EspSerialHandler>, // 192
    state_handler: EspStateHandler,                     // size = 184 : Config:176
    config: EspWifiConfig<'h>,                          // 176
    connected_client: ArrayVec<u8, MAX_CLIENT_NB>,      // 8
    send_buff: [u8; MSG_SZ],                            // 40
}

impl<'h, const MSG_SZ: usize, EspSerialHandler: EspSerial>
    EspWifiHandler<'h, MSG_SZ, EspSerialHandler>
{
    pub fn new(serial_handler: EspSerialHandler, config: EspWifiConfig<'h>) -> Self {
        let state_handler = EspStateHandler::new();

        let mut at_com_handler = EspAtCom::new(serial_handler);

        // Cmd handler: set state cmd (Reset: the first time, we don't have a resp)
        if let Some(cmd) = state_handler.get_cmd(&config) {
            at_com_handler.process_cmd(cmd)
        }

        Self {
            at_com_handler,
            state_handler,
            config,
            connected_client: ArrayVec::new(),
            send_buff: [0u8; MSG_SZ],
        }
    }

    pub fn update<Timer: GetTime>(&mut self, timer: &Timer) -> bool {
        if let Some(resp) = self.at_com_handler.update(timer.now_us(), &self.config) {
            match self.state_handler.update(&resp, &self.config) {
                StateEvent::None => {}
                StateEvent::CmdRequest => {
                    if let Some(cmd) = self.state_handler.get_cmd(&self.config) {
                        self.at_com_handler.process_cmd(cmd)
                    }
                }
                StateEvent::CmdFailed => self.at_com_handler.retry_cmd(),
                StateEvent::WaitForResp => self.at_com_handler.wait_for_resp(timer.now_us()),
                StateEvent::ClientConnected { client_id } => {
                    self.connected_client.push(client_id);
                }
                StateEvent::ClientMsg { client_id } => {
                    // if let Some(client_msg) = self.at_com_handler.get_client_next_msg(client_id) {
                    //     // hadle msg
                    //     // TMP: Echo
                    //     self.send_buff_to_client(client_id, client_msg.get_buff_slice());
                    // }
                }
                StateEvent::ClientDisconnected { client_id } => {
                    if let Ok(idx_rm) = self
                        .connected_client
                        .binary_search_by(|clt_id| clt_id.cmp(&client_id))
                    {
                        self.connected_client.pop_at(idx_rm);
                    }
                }
                StateEvent::ReadyTosendMsg => self
                    .at_com_handler
                    .write_msg(&self.send_buff, timer.now_us()),
                StateEvent::MsgSent => {}
            }

            // Tmp
            true
        } else {
            false
        }
    }

    pub fn send_buff_to_client(&mut self, client_id: u8, buff: &[u8]) {
        let buff_len = buff.len();

        self.send_buff[..buff_len].copy_from_slice(buff);

        self.at_com_handler.process_cmd(
            self.state_handler
                .start_msg_send_cmd(client_id, buff_len as u8),
        )
    }

    pub fn is_ready(&self) -> bool {
        self.state_handler.is_ready()
    }
}
