use arrayvec::ArrayVec;

#[derive(Default)]
pub struct BufferU8<const N: usize> {
    buffer: ArrayVec<u8, N>,
}

impl<const N: usize> BufferU8<N> {
    pub const fn new_const() -> Self {
        Self {
            buffer: ArrayVec::new_const(),
        }
    }
    pub fn try_push(&mut self, byte: u8) -> bool {
        let buffer_full = self.buffer.is_full();

        if buffer_full {
            self.get_first();
        }

        self.buffer.push(byte);

        !buffer_full
    }
    pub fn get_first(&mut self) -> Option<u8> {
        self.buffer.pop_at(0)
    }
    pub fn get_buff(&self) -> &[u8] {
        &self.buffer.as_slice()
    }
    pub fn clear(&mut self) {
        self.buffer.clear()
    }
}
