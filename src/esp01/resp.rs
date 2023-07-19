use crate::{serial::UsartRxBuffer, tools::buffer::BufferU8};

#[derive(Debug, Clone, Copy)]
pub enum EspResp {
    Ready,
    Ok,
    StaConnected,
    StaGotIp,
    ClientConnected,
    ClientDisconnected,
    ClientMsg,
    Error,
    Fail,
}

#[derive(Default)]
pub struct EspRespHandler<const RESP_SZ: usize> {
    resp_buff: BufferU8<RESP_SZ>,
    usart_rx_buff: UsartRxBuffer,
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
    const CLIENT_MSG: &'static [u8] = b"+IPD,";

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
            // self.resp_buff[..buff.len()].copy_from_slice(buff);
            // self.resp_buff.clear();
            Some(EspResp::Ready)
        } else if resp_buff.ends_with(Self::OK) {
            // self.resp_buff.clear();
            Some(EspResp::Ok)
        } else if resp_buff.ends_with(Self::ERROR) {
            // self.resp_buff.clear();
            Some(EspResp::Error)
        } else if resp_buff.ends_with(Self::FAIL) {
            // self.resp_buff.clear();
            Some(EspResp::Fail)
        } else if resp_buff.ends_with(Self::CONNECTED_CLIENT) {
            // self.resp_buff.clear();
            Some(EspResp::ClientConnected)
        } else if resp_buff.ends_with(Self::DISCONNECTED_CLIENT) {
            // self.resp_buff.clear();
            Some(EspResp::ClientDisconnected)
        } else if resp_buff.starts_with(Self::CLIENT_MSG) {
            // self.resp_buff.clear();
            Some(EspResp::ClientMsg)
        } else if resp_buff.ends_with(Self::STA_CONNECTED) {
            // self.resp_buff.clear();
            Some(EspResp::StaConnected)
        } else if resp_buff.ends_with(Self::STA_GOT_IP) {
            // self.resp_buff.clear();
            Some(EspResp::StaGotIp)
        } else {
            None
        }
    }
}
