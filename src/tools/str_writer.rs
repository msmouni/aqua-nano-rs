use core::fmt;

pub struct StrWriter<const BUFF_SIZE: usize> {
    buffer: [u8; BUFF_SIZE],
    used: usize,
}

impl<const BUFF_SIZE: usize> Default for StrWriter<BUFF_SIZE> {
    fn default() -> Self {
        StrWriter {
            buffer: [0; BUFF_SIZE],
            used: 0,
        }
    }
}

impl<const BUFF_SIZE: usize> StrWriter<BUFF_SIZE> {
    pub fn new() -> Self {
        StrWriter {
            buffer: [0; BUFF_SIZE],
            used: 0,
        }
    }

    fn as_str(&self) -> Option<&str> {
        if self.used <= self.buffer.len() {
            // only successful concats of str - must be a valid str.
            use core::str::from_utf8;
            from_utf8(&self.buffer[..self.used]).ok()
        } else {
            None
        }
    }

    pub fn get_str<'a>(&'a mut self, args: fmt::Arguments) -> Result<&'a str, fmt::Error> {
        self.used = 0;
        fmt::write(self, args)?;
        self.as_str().ok_or(fmt::Error)
    }
}

impl<const BUFF_SIZE: usize> fmt::Write for StrWriter<BUFF_SIZE> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.used > BUFF_SIZE {
            return Err(fmt::Error);
        }
        let remaining_buf = &mut self.buffer[self.used..];
        let raw_s = s.as_bytes();

        let raw_s_len = raw_s.len();

        if remaining_buf.len() < raw_s_len {
            Err(fmt::Error)
        } else {
            remaining_buf[..raw_s_len].copy_from_slice(raw_s);
            self.used += raw_s_len;

            Ok(())
        }
    }
}
