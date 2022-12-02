use super::{reset_time, ImplTimer, OVER_FLOW_COUNTER};
use arduino_hal::pac::TC0;

pub struct FastPwmTimer<const SYS_CLK_HZ: u32, const PRESCALER: u32> {
    timer_counter: TC0,
    presc_clk_period_us: u32,
}
impl<const SYS_CLK_MHZ: u32, const PRESCALER: u32> FastPwmTimer<SYS_CLK_MHZ, PRESCALER> {
    const PRESCALER_TEST: () = assert!(
        (PRESCALER == 1)
            || (PRESCALER == 8)
            || (PRESCALER == 64)
            || (PRESCALER == 256)
            || (PRESCALER == 1024)
    );
}

impl<const SYS_CLK_MHZ: u32, const PRESCALER: u32> ImplTimer
    for FastPwmTimer<SYS_CLK_MHZ, PRESCALER>
{
    fn new(timer_counter: TC0) -> Self {
        let _ = Self::PRESCALER_TEST;

        let presc_clk_period_us = (PRESCALER * 256) / SYS_CLK_MHZ;
        Self {
            timer_counter,
            presc_clk_period_us,
        }
    }

    fn init(&mut self) {
        // TCCR0 (Timer/Counter Control Register): WGM01:0 Waveform Generation Mode : FAST PWM
        self.timer_counter.tccr0a.write(|w| w.wgm0().pwm_fast());

        // TCCR0 (Timer/Counter Control Register): CS0 (Clock Select)
        self.timer_counter.tccr0b.write(|w| match PRESCALER {
            1 => w.cs0().direct(),
            8 => w.cs0().prescale_8(),
            64 => w.cs0().prescale_64(),
            256 => w.cs0().prescale_256(),
            1024 => w.cs0().prescale_1024(),
            _ => unreachable!(),
        });

        // TIMSK (Timer/Counter Interrupt Mask): TOIE0 Timer/Counter0 Overflow Interrupt Enable
        self.timer_counter.timsk0.write(|w| w.toie0().set_bit());

        // Reset the global counter
        reset_time()
    }
    fn micros(&self) -> u64 {
        let presc_clk_period_count = avr_device::interrupt::free(|cs| {
            let ovflow_count = OVER_FLOW_COUNTER.borrow(cs).get();
            ovflow_count * 256 + (self.timer_counter.tcnt0.read().bits() as u64)
        });

        (self.presc_clk_period_us as u64) * presc_clk_period_count
    }
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_OVF() {
    avr_device::interrupt::free(|cs| {
        let borrowed_ovflw_counter = OVER_FLOW_COUNTER.borrow(cs);
        let counter = borrowed_ovflw_counter.get();
        borrowed_ovflw_counter.set(counter + 1)
    })
}
