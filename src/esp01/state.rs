pub enum EspState {
    Idle,
    Reset { t_cmd_sent: Option<u64> },
    WaitReady { t_start_wait: u64 },
    ConfigWifiMode { t_cmd_sent: Option<u64> },
    ConfigSta { t_cmd_sent: Option<u64> },
    ConfigAP { t_cmd_sent: Option<u64> },
    StaIp { t_cmd_sent: Option<u64> },
    ApIp { t_cmd_sent: Option<u64> },
    EnablingMultiConx { t_cmd_sent: Option<u64> }, // Check Order!!!
    StartingTcpIpServer { t_cmd_sent: Option<u64> },
    Ready,
}
