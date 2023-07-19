use super::cmd::EspCmd;

pub enum EspState<'cmd> {
    Idle,
    Reset {
        t_cmd_sent: Option<u64>,
        cmd: EspCmd<'cmd>,
    },
    WaitReady {
        t_start_wait: u64,
    },
    ConfigWifiMode {
        t_cmd_sent: Option<u64>,
        cmd: EspCmd<'cmd>,
    },
    ConfigSta {
        t_cmd_sent: Option<u64>,
        cmd: EspCmd<'cmd>,
    },
    WaitStaConnected {
        t_start_wait: u64,
    },
    ConfigAP {
        t_cmd_sent: Option<u64>,
        cmd: EspCmd<'cmd>,
    },
    StaIp {
        t_cmd_sent: Option<u64>,
        cmd: EspCmd<'cmd>,
    },
    WaitStaGotIp {
        t_start_wait: u64,
    },
    ApIp {
        t_cmd_sent: Option<u64>,
        cmd: EspCmd<'cmd>,
    },
    EnablingMultiConx {
        t_cmd_sent: Option<u64>,
        cmd: EspCmd<'cmd>,
    },
    StartingTcpIpServer {
        t_cmd_sent: Option<u64>,
        cmd: EspCmd<'cmd>,
    },
    Ready,
}
