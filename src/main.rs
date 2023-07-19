#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod app;
mod drivers;

use app::Application;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let mut app = Application::new();

    loop {
        app.update();
    }
}
