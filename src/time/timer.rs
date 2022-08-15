enum TimerState {
    Stopped,
    Started { t_start_us: u64 },
    Expired,
}

pub enum TimerError {
    NotStarted,
}

pub struct Timer {
    timeout_us: u64,
    state: TimerState,
}

impl Timer {
    pub fn new(timeout_us: u64) -> Self {
        Self {
            timeout_us,
            state: TimerState::Stopped,
        }
    }

    pub fn start(&mut self, t_start_us: u64) {
        self.state = TimerState::Started { t_start_us }
    }

    pub fn stop(&mut self) {
        self.state = TimerState::Stopped
    }

    pub fn update(&mut self, micros_us: u64) {
        match self.state {
            TimerState::Stopped | TimerState::Expired => {}
            TimerState::Started { t_start_us } => {
                if (micros_us - t_start_us) >= self.timeout_us {
                    self.state = TimerState::Expired
                }
            }
        }
    }

    pub fn has_expired(&mut self, micros_us: u64) -> Result<bool, TimerError> {
        if let TimerState::Stopped = self.state {
            Err(TimerError::NotStarted)
        } else {
            self.update(micros_us);

            Ok(matches!(self.state, TimerState::Expired))
        }
    }

    pub fn has_started(&self) -> bool {
        matches!(self.state, TimerState::Started { t_start_us: _ })
    }
}
