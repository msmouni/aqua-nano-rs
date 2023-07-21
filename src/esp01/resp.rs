use super::clients::{ClientMessage, ClientsMessages, MAX_CLIENT_MSGS, MAX_CLIENT_NB};
use crate::{serial::UsartRxBuffer, tools::buffer::BufferU8};

#[derive(Debug, Clone, Copy)]
pub enum EspResp {
    Ready,
    Ok,
    StaConnected,
    StaDisconnected,
    StaGotIp,
    ClientConnected(u8),
    ClientDisconnected(u8),
    ClientMsg(u8),
    MsgSent,
    Error,
    Fail,
}

#[derive(Default)]
pub struct EspRespHandler<const RESP_SZ: usize> {
    resp_buff: BufferU8<RESP_SZ>,
    usart_rx_buff: UsartRxBuffer,
    clients_msgs: ClientsMessages<RESP_SZ, MAX_CLIENT_MSGS, MAX_CLIENT_NB>,
}

impl<const RESP_SZ: usize> EspRespHandler<RESP_SZ> {
    const READY: &'static [u8] = b"ready\r\n";
    const OK: &'static [u8] = b"OK\r\n";
    const ERROR: &'static [u8] = b"ERROR\r\n";
    const FAIL: &'static [u8] = b"FAIL\r\n";
    const STA_CONNECTED: &'static [u8] = b"WIFI CONNECTED\r\n";
    const STA_DISCONNECTED: &'static [u8] = b"WIFI DISCONNECT\r\n";
    const STA_GOT_IP: &'static [u8] = b"WIFI GOT IP\r\n";
    const CONNECTED_CLIENT: &'static [u8] = b",CONNECT\r\n";
    const DISCONNECTED_CLIENT: &'static [u8] = b",CLOSED\r\n";
    const CLIENT_MSG: &'static [u8] = b"\r\n+IPD,";
    const MSG_SENT: &'static [u8] = b"SEND OK\r\n";

    pub fn get_resp_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.resp_buff.get_buff()).ok()
    }

    pub fn get_resp_buff(&self) -> &[u8] {
        self.resp_buff.get_buff()
    }

    pub fn poll(&mut self) -> Option<EspResp> {
        while let Some(b) = self.usart_rx_buff.try_get_byte() {
            self.resp_buff.try_push(b);
            if let Some(resp) = self.check_resp() {
                self.resp_buff.clear();
                return Some(resp);
            }
        }
        None
    }

    // TMP for debug
    pub fn clear_buff(&mut self) {
        self.resp_buff.clear();
    }

    // TODO: Seperate check: At each stage we know the reponse that we're expecting ...
    fn check_resp(&mut self) -> Option<EspResp> {
        let resp_buff = self.resp_buff.get_buff();
        if resp_buff.ends_with(Self::READY) {
            Some(EspResp::Ready)
        } else if resp_buff.ends_with(Self::MSG_SENT) {
            // before OK
            Some(EspResp::MsgSent)
        } else if resp_buff.ends_with(Self::OK) {
            Some(EspResp::Ok)
        } else if resp_buff.ends_with(Self::ERROR) {
            Some(EspResp::Error)
        } else if resp_buff.ends_with(Self::FAIL) {
            Some(EspResp::Fail)
        } else if resp_buff.ends_with(Self::CONNECTED_CLIENT) {
            /*
            0,CONNECT
            1,CONNECT
            */
            if let Some(clt_id) = resp_buff.split(|b| *b == b',').next() {
                if let Some(client_id) = get_u8_from_slice(clt_id) {
                    self.clients_msgs.add_client(client_id);
                    Some(EspResp::ClientConnected(client_id))
                } else {
                    None
                }
            } else {
                None
            }
        } else if resp_buff.ends_with(Self::DISCONNECTED_CLIENT) {
            /*
            0,CLOSED
            1,CLOSED
            */
            if let Some(clt_id) = resp_buff.split(|b| *b == b',').next() {
                if let Some(client_id) = get_u8_from_slice(clt_id) {
                    self.clients_msgs.remove_client(client_id);
                    Some(EspResp::ClientDisconnected(client_id))
                } else {
                    None
                }
            } else {
                None
            }
        } else if resp_buff.starts_with(Self::CLIENT_MSG) {
            /*
            +IPD,0,19:Hello from client 1
            +IPD,1,19:Hello from client 2
            */

            // Some(EspResp::ClientMsg(0))

            // TODO: Handle Too Long Messages
            let mut rsp_splt = resp_buff.split(|b| *b == b',');
            if let Some(ipd) = rsp_splt.next() {
                if let Some(clt_id) = rsp_splt.next() {
                    if let Some(client_msg_len) = rsp_splt.next() {
                        let mut msg_len_splt = client_msg_len.split(|b| *b == b':');

                        if let Some(msg_len) = msg_len_splt.next() {
                            if let Some(msg_data) = msg_len_splt.next() {
                                if let Some(msg_len_u8) = get_u8_from_slice(msg_len) {
                                    let msg_len = msg_len_u8 as usize;
                                    if msg_data.len() == msg_len {
                                        if let Some(client_id) = get_u8_from_slice(clt_id) {
                                            self.clients_msgs
                                                .add_client_msg(client_id, &msg_data[..msg_len]);
                                            return Some(EspResp::ClientMsg(client_id));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            None
        } else if resp_buff.ends_with(Self::STA_CONNECTED) {
            Some(EspResp::StaConnected)
        } else if resp_buff.ends_with(Self::STA_DISCONNECTED) {
            Some(EspResp::StaDisconnected)
        } else if resp_buff.ends_with(Self::STA_GOT_IP) {
            Some(EspResp::StaGotIp)
        } else {
            None
        }
    }

    pub fn get_client_next_msg(&mut self, client_id: u8) -> Option<ClientMessage<RESP_SZ>> {
        self.clients_msgs.get_client_next_msg(client_id)
    }
}

fn get_u8_from_slice(slice: &[u8]) -> Option<u8> {
    if let Ok(slice_str) = core::str::from_utf8(slice) {
        if let Ok(slice_u8) = u8::from_str_radix(slice_str, 10) {
            Some(slice_u8)
        } else {
            None
        }
    } else {
        None
    }
}
