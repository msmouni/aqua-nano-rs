use arrayvec::ArrayVec;

pub const MAX_CLIENT_MSGS: usize = 1;
pub const MAX_CLIENT_NB: usize = 1;

pub struct ClientMessage<const MSG_SZ: usize> {
    msg_len: usize,
    msg_buff: [u8; MSG_SZ],
}

impl<const MSG_SZ: usize> ClientMessage<MSG_SZ> {
    pub fn get_buff_slice(&self) -> &[u8] {
        &self.msg_buff[..self.msg_len]
    }
}

pub struct ClientMessages<const MSG_SZ: usize, const MAX_MSG_NB: usize> {
    client_id: u8,
    messages: ArrayVec<ClientMessage<MSG_SZ>, MAX_MSG_NB>,
}

impl<const MSG_SZ: usize, const MAX_MSG_NB: usize> ClientMessages<MSG_SZ, MAX_MSG_NB> {
    pub fn new(client_id: u8) -> Self {
        Self {
            client_id,
            messages: ArrayVec::new(),
        }
    }

    pub fn add_message(&mut self, bytes: &[u8]) {
        let mut buff = [0u8; MSG_SZ];

        let msg_len = bytes.len();

        buff[..msg_len].copy_from_slice(bytes);

        if self.messages.is_full() {
            self.messages.pop_at(0);
        }

        self.messages.push(ClientMessage {
            msg_len,
            msg_buff: buff,
        })
    }

    pub fn get_next_msg(&mut self) -> Option<ClientMessage<MSG_SZ>> {
        self.messages.pop_at(0)
    }
}

#[derive(Default)]
pub struct ClientsMessages<const MSG_SZ: usize, const MAX_CL_MSG_NB: usize, const MAX_CL_NB: usize>
{
    clients_messages: ArrayVec<ClientMessages<MSG_SZ, MAX_CL_MSG_NB>, MAX_CLIENT_NB>,
}

impl<const MSG_SZ: usize, const MAX_CL_MSG_NB: usize, const MAX_CL_NB: usize>
    ClientsMessages<MSG_SZ, MAX_CL_MSG_NB, MAX_CL_NB>
{
    pub fn add_client(&mut self, client_id: u8) {
        self.clients_messages.push(ClientMessages::new(client_id))
    }

    pub fn remove_client(&mut self, client_id: u8) {
        if let Ok(rm_idx) = self
            .clients_messages
            .binary_search_by(|r| r.client_id.cmp(&client_id))
        {
            self.clients_messages.pop_at(rm_idx); // Here: panics when having two clients: the first one to connect disconnect than the other one disconnect
        }
    }

    pub fn add_client_msg(&mut self, client_id: u8, bytes: &[u8]) {
        if let Ok(client_id) = self
            .clients_messages
            .binary_search_by(|r| r.client_id.cmp(&client_id))
        {
            if let Some(client_msgs) = self.clients_messages.get_mut(client_id) {
                client_msgs.add_message(bytes)
            }
        }
    }

    pub fn get_client_next_msg(&mut self, client_id: u8) -> Option<ClientMessage<MSG_SZ>> {
        if let Ok(client_id) = self
            .clients_messages
            .binary_search_by(|r| r.client_id.cmp(&client_id))
        {
            if let Some(client_msgs) = self.clients_messages.get_mut(client_id) {
                client_msgs.get_next_msg()
            } else {
                None
            }
        } else {
            None
        }
    }
}