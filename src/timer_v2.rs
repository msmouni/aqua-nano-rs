use arduino_hal::pac::TC0;
use avr_device::interrupt::Mutex;
use core::cell::Cell;

static OVER_FLOW_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
pub struct Time {
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    milli_second: u32,
    micro_second: u32,
}
impl Time {
    fn add(&mut self, other: &Self) {
        self.day += other.day;
        self.hour += other.hour;
        self.minute += other.minute;
        self.second += other.second;
        self.milli_second += other.milli_second;
        self.micro_second += other.micro_second;
    }
    fn sub(&self, other: &Self) -> Self {
        Self {
            day: self.day - other.day,
            hour: self.hour - other.hour,
            minute: self.minute - other.minute,
            second: self.second - other.second,
            milli_second: self.milli_second - other.milli_second,
            micro_second: self.micro_second - other.micro_second,
        }
    }
    pub fn compute(&mut self, elapsed_micros: u32) {
        let elapsed_millis = elapsed_micros / 1_000;

        let micro_second = elapsed_micros - (elapsed_millis * 1_000);

        let elapsed_sec = elapsed_millis / 1_000;

        let milli_second = elapsed_millis - (elapsed_sec * 1_000);

        let elapsed_min = elapsed_sec / 60;

        let second = elapsed_sec - (elapsed_min * 60);

        let elapsed_hours = elapsed_min / 60;

        let minute = elapsed_min - (elapsed_hours * 60);

        let day = elapsed_hours / 24;

        let hour = elapsed_hours - (day * 24);

        self.add(&Self {
            day,
            hour,
            minute,
            second,
            milli_second,
            micro_second,
        })
    }

    pub fn elapsed(&self, new_time: &Self) -> Option<Self> {
        if self < new_time {
            Some(new_time.sub(self))
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub enum Prescaler {
    Prescaler1 = 1,
    Prescaler8 = 8,
    Prescaler64 = 64,
    Prescaler256 = 256,
    Prescaler1024 = 1024,
}
// Example:
//
// ╔═══════════╦══════════════╦═══════════════════╗
// ║ PRESCALER ║ TIMER_COUNTS ║ Overflow Interval ║
// ╠═══════════╬══════════════╬═══════════════════╣
// ║        64 ║          250 ║              1 ms ║
// ║       256 ║          125 ║              2 ms ║
// ║       256 ║          250 ║              4 ms ║
// ║      1024 ║          125 ║              8 ms ║
// ║      1024 ║          250 ║             16 ms ║
// ╚═══════════╩══════════════╩═══════════════════╝

pub struct CtcTimer<const SYS_CLK_HZ: u32, const PRESCALER: u32, const OF_COUNT: u8> {
    timer_counter: TC0,
    over_flow_period_us: u32,
    time: Time,
}
impl<const SYS_CLK_MHZ: u32, const PRESCALER: u32, const OF_COUNT: u8>
    CtcTimer<SYS_CLK_MHZ, PRESCALER, OF_COUNT>
{
    const _PRESCALER_TEST: () = assert!(
        (PRESCALER == 1)
            || (PRESCALER == 8)
            || (PRESCALER == 64)
            || (PRESCALER == 256)
            || (PRESCALER == 1024)
    );

    const _ROUND_TEST: () = assert!(
        (PRESCALER * ((OF_COUNT + 1) as u32) / SYS_CLK_MHZ) * SYS_CLK_MHZ
            == PRESCALER * ((OF_COUNT + 1) as u32),
        "OF_PERIOD_US: u32 = PRESCALER * (OF_COUNT as u32) / SYS_CLK_MHZ"
    );

    pub fn new(timer_counter: TC0) -> Self {
        let over_flow_period_us = (PRESCALER * ((OF_COUNT + 1) as u32)) / SYS_CLK_MHZ;
        Self {
            timer_counter,
            over_flow_period_us,
            time: Time::default(),
        }
    }

    pub fn init(&mut self) {
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
        avr_device::interrupt::free(|cs| {
            OVER_FLOW_COUNTER.borrow(cs).set(0);
        });
    }

    pub fn micros(&self) -> u32 {
        self.over_flow_period_us
            * avr_device::interrupt::free(|cs| OVER_FLOW_COUNTER.borrow(cs).get())
    }

    pub fn millis(&self) -> u32 {
        self.micros() / 1_000
    }

    pub fn get_time(&mut self) -> Time {
        let elapsed_micros = self.over_flow_period_us
            * avr_device::interrupt::free(|cs| {
                let borrowed_ovflw_counter = OVER_FLOW_COUNTER.borrow(cs);
                let counter = borrowed_ovflw_counter.get();

                // Reset counter
                borrowed_ovflw_counter.set(0);

                counter
            });

        self.time.compute(elapsed_micros);

        self.time.clone()
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

//////////////////////////////////////////////////////////////////////////////
pub struct FastPwmTimer<const SYS_CLK_HZ: u32, const PRESCALER: u32> {
    timer_counter: TC0,
    presc_clk_period_us: u32,
    time: Time,
}
impl<const SYS_CLK_MHZ: u32, const PRESCALER: u32> FastPwmTimer<SYS_CLK_MHZ, PRESCALER> {
    const _PRESCALER_TEST: () = assert!(
        (PRESCALER == 1)
            || (PRESCALER == 8)
            || (PRESCALER == 64)
            || (PRESCALER == 256)
            || (PRESCALER == 1024)
    );

    pub fn new(timer_counter: TC0) -> Self {
        let presc_clk_period_us = (PRESCALER * 256) / SYS_CLK_MHZ;
        Self {
            timer_counter,
            presc_clk_period_us,
            time: Time::default(),
        }
    }

    pub fn init(&mut self) {
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
        avr_device::interrupt::free(|cs| {
            OVER_FLOW_COUNTER.borrow(cs).set(0);
        });
    }
    // pub fn micros(&self) -> u32 {
    //     let presc_clk_period_count = avr_device::interrupt::free(|cs| {
    //         let ovflow_count = OVER_FLOW_COUNTER.borrow(cs).get();
    //         ovflow_count * 256 + (self.timer_counter.tcnt0.read().bits() as u32)
    //     });

    //     self.presc_clk_period_us * presc_clk_period_count
    // }

    // pub fn millis(&self) -> u32 {
    //     self.micros() / 1_000
    // }

    pub fn get_time(&mut self) -> Time {
        let elapsed_micros = avr_device::interrupt::free(|cs| {
            let borrowed_ovflw_counter = OVER_FLOW_COUNTER.borrow(cs);
            let counter = borrowed_ovflw_counter.get();

            // Reset counter
            borrowed_ovflw_counter.set(0);

            (counter * 256 + (self.timer_counter.tcnt0.read().bits() as u32))
                * self.presc_clk_period_us
        });

        self.time.compute(elapsed_micros);

        self.time.clone()
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

pub struct SysTimer {
    timer_counter: TC0,
    sys_clk_hz: u32,
}

// impl SysTimer {
//     pub fn init(&mut self) {
//         // Configure the timer for the above interval (in CTC mode)
//         // and enable its interrupt.

//         // TCCR0 (Timer/Counter Control Register): WGM01:0 Waveform Generation Mode : CTC "Clear Timer on Compare Match
//         self.timer_counter.tccr0a.write(|w| w.wgm0().ctc());

//         self.timer_counter
//             .ocr0a
//             .write(|w| unsafe { w.bits(TIMER_COUNTS as u8) });
//         self.timer_counter.tccr0b.write(|w| match PRESCALER {
//             8 => w.cs0().prescale_8(),
//             64 => w.cs0().prescale_64(),
//             256 => w.cs0().prescale_256(),
//             1024 => w.cs0().prescale_1024(),
//             _ => panic!(),
//         });
//         self.timer_counter.timsk0.write(|w| w.ocie0a().set_bit());

//         // Reset the global millisecond counter
//         avr_device::interrupt::free(|cs| {
//             MILLIS_COUNTER.borrow(cs).set(0);
//         });
//     }
// }
