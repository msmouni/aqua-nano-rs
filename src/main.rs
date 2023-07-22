#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(panic_info_message)]

mod app;
mod drivers;
mod esp01;
mod panic;
mod serial;
mod tools;

use app::Application;

use drivers::time::sys_timer::{CtcTimer, ImplTimer, SysTimer};
use esp01::{
    EspApConfig, EspIp, EspIpConfig, EspRespHandler, EspSerial, EspWifiConfig, EspWifiHandler,
    GetTime, SsidPassword, WifiEncryption,
};
use serial::{SerialHandler, RX_BUFFER_SIZE};

#[arduino_hal::entry]
fn main() -> ! {
    // let mut app = Application::new();

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    /////////////////////////////////////////////////
    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);

    // core::mem::size_of::<EspWifiHandler<RX_BUFFER_SIZE, SerialHandler>>(); // size:263

    // let mut resp_h: EspRespHandler<RX_BUFFER_SIZE> = Default::default();

    let mut esp_wifi: EspWifiHandler<RX_BUFFER_SIZE, SerialHandler> = EspWifiHandler::new(
        SerialHandler::new(serial),
        EspWifiConfig::ApSta {
            sta_config: SsidPassword {
                ssid: "Bbox-9A370343",
                password: "QdQ3kPrVaRe6udkax9",
            },
            sta_ip: EspIpConfig::Static {
                ip: EspIp {
                    ip: "192.168.1.88",
                    gw: "192.168.1.1",
                    mask: "255.255.255.0",
                },
            },
            ap_config: EspApConfig {
                wifi: SsidPassword {
                    ssid: "test_ap_3",
                    password: "87321654",
                },
                chanel_id: 4,
                encryption: WifiEncryption::Wpa2Psk,
                max_sta_nb: 4, //////////// MAX_CLIENTS
                hide_ssid: false,
            },
            ap_ip: EspIpConfig::Static {
                ip: EspIp {
                    ip: "192.168.5.1",
                    gw: "192.168.5.1",
                    mask: "255.255.255.0",
                },
            },
            tcp_port: 2_000,
        },
    );
    /////////////////////////////////////////////////

    let mut sys_timer: SysTimer<CtcTimer<16, 64, 250>> = SysTimer::new(dp.TC0);

    sys_timer.init();

    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    let mut led_pin = pins.d13.into_output();

    loop {
        // app.update();

        led_pin.set_low();

        if esp_wifi.update(&sys_timer) {
            led_pin.set_high();
        }

        sys_timer.delay_micros(1_000_000);
    }
}

impl EspSerial for SerialHandler {
    fn write_bytes(&mut self, bytes: &[u8]) -> bool {
        self.try_write_bytes(bytes)
    }

    fn write_str(&mut self, s: &str) -> bool {
        self.try_write_str(s)
    }

    fn write_fmt(&mut self, args: core::fmt::Arguments) -> bool {
        self.try_write_fmt(args)
    }
}

impl<WhichTimer: ImplTimer> GetTime for SysTimer<WhichTimer> {
    fn now_us(&self) -> u64 {
        self.micros()
    }
}
