use super::{reset_time, ImplTimer, OVER_FLOW_COUNTER};
use arduino_hal::pac::TC0;

/// Example:
///
/// ╔═══════════╦══════════════╦═══════════════════╗
/// ║ PRESCALER ║ TIMER_COUNTS ║ Overflow Interval ║
/// ╠═══════════╬══════════════╬═══════════════════╣
/// ║        64 ║          250 ║              1 ms ║
/// ║       256 ║          125 ║              2 ms ║
/// ║       256 ║          250 ║              4 ms ║
/// ║      1024 ║          125 ║              8 ms ║
/// ║      1024 ║          250 ║             16 ms ║
/// ╚═══════════╩══════════════╩═══════════════════╝
pub struct CtcTimer<const SYS_CLK_MHZ: u32, const PRESCALER: u32, const OF_COUNT: u8> {
    timer_counter: TC0,
    over_flow_period_us: u32,
}
impl<const SYS_CLK_MHZ: u32, const PRESCALER: u32, const OF_COUNT: u8>
    CtcTimer<SYS_CLK_MHZ, PRESCALER, OF_COUNT>
{
    const PRESCALER_TEST: () = assert!(
        (PRESCALER == 1)
            || (PRESCALER == 8)
            || (PRESCALER == 64)
            || (PRESCALER == 256)
            || (PRESCALER == 1024)
    );

    const ROUND_TEST: () = assert!(
        (PRESCALER * ((OF_COUNT + 1) as u32) / SYS_CLK_MHZ) * SYS_CLK_MHZ
            == PRESCALER * ((OF_COUNT + 1) as u32),
        "OF_PERIOD_US: u32 = PRESCALER * (OF_COUNT as u32) / SYS_CLK_MHZ"
    );
}

impl<const SYS_CLK_MHZ: u32, const PRESCALER: u32, const OF_COUNT: u8> ImplTimer
    for CtcTimer<SYS_CLK_MHZ, PRESCALER, OF_COUNT>
{
    fn new(timer_counter: TC0) -> Self {
        let _ = Self::PRESCALER_TEST;
        let _ = Self::ROUND_TEST;

        let over_flow_period_us = (PRESCALER * ((OF_COUNT + 1) as u32)) / SYS_CLK_MHZ;
        Self {
            timer_counter,
            over_flow_period_us,
        }
    }

    fn init(&mut self) {
        // TCCR0 (Timer/Counter Control Register): WGM01:0 Waveform Generation Mode : CTC "Clear Timer on Compare Match
        self.timer_counter.tccr0a.write(|w| w.wgm0().ctc());

        // OCR0 (Output Compare Register)
        self.timer_counter
            .ocr0a
            .write(|w| unsafe { w.bits(OF_COUNT) });

        // TCCR0 (Timer/Counter Control Register): CS0 (Clock Select)
        self.timer_counter.tccr0b.write(|w| match PRESCALER {
            1 => w.cs0().direct(),
            8 => w.cs0().prescale_8(),
            64 => w.cs0().prescale_64(),
            256 => w.cs0().prescale_256(),
            1024 => w.cs0().prescale_1024(),
            _ => unreachable!(),
        });

        // TIMSK (Timer/Counter Interrupt Mask): OCIE0 Timer/Counter0 Output Compare Match Interrupt Enable
        self.timer_counter.timsk0.write(|w| w.ocie0a().set_bit());

        // Reset the global counter
        reset_time()
    }

    fn micros(&self) -> u64 {
        (self.over_flow_period_us as u64)
            * avr_device::interrupt::free(|cs| OVER_FLOW_COUNTER.borrow(cs).get())
    }
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let borrowed_ovflw_counter = OVER_FLOW_COUNTER.borrow(cs);
        let counter = borrowed_ovflw_counter.get();
        borrowed_ovflw_counter.set(counter + 1)
    })
}
