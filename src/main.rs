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

    // let mut resp_h: EspRespHandler<RX_BUFFER_SIZE> = Default::default();

    let mut esp_wifi: EspWifiHandler<RX_BUFFER_SIZE, SerialHandler> = EspWifiHandler::new(
        SerialHandler::new(serial),
        EspWifiConfig::ApSta {
            sta_config: SsidPassword {
                ssid: "Ssid",
                password: "password",
            },
            sta_ip: EspIpConfig::Static {
                ip: EspIp {
                    ip: "xxx.xxx.xxx.xxx",
                    gw: "xxx.xxx.xxx.xxx",
                    mask: "xxx.xxx.xxx.xxx",
                },
            },
            ap_config: EspApConfig {
                wifi: SsidPassword {
                    ssid: "ap_ssid",
                    password: "ap_pass",
                },
                chanel_id: 4,
                encryption: WifiEncryption::Wpa2Psk,
                max_sta_nb: 4,
                hide_ssid: false,
            },
            ap_ip: EspIpConfig::Static {
                ip: EspIp {
                    ip: "xxx.xxx.xxx.xxx",
                    gw: "xxx.xxx.xxx.xxx",
                    mask: "xxx.xxx.xxx.xxx",
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

        if !esp_wifi.is_ready() {
            led_pin.set_low();
        }

        // serial_h.write_str("AT\r\n");

        // sys_timer.delay_micros(3_000_000);

        if esp_wifi.update(&sys_timer) {
            led_pin.set_high();
        }

        sys_timer.delay_micros(1_000_000);

        /*
        Note:
         AT
         Bytes(7):
         ERROR
         end
         (7):ERROR

         AT
         Bytes(11):
         (7):ERROR
         end
         (11):(7):ERROR

         AT
        */

        /*if let Some(_resp) = resp_h.poll() {
            led_pin.set_high();

            serial_h.write_str("Bytes:");
            // serial_h.write_fmt(format_args!("Bytes({}):\n", idx));
            serial_h.write_bytes(resp_h.get_resp_buff());
            serial_h.write_str("end\n");
            if let Some(rs) = resp_h.get_resp_str() {
                serial_h.write_fmt(format_args!("({}):{}\n", rs.len(), rs)); // .write_str(rs);
            }
            resp_h.clear_buff();
        }

        sys_timer.delay_micros(1_000_000);*/
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
