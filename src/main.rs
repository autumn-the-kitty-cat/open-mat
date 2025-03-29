#![no_main]
#![no_std]

use core::panic::PanicInfo;

use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    adc::{
        config::{AdcConfig, SampleTime},
        Adc,
    },
    pac,
    prelude::*,
};

#[panic_handler]
fn panic(i: &PanicInfo) -> ! {
    rprintln!("{}", i);
    loop {}
}

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    rtt_init_print!();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();

    let sensor1 = gpioa.pa6.into_analog();
    let sensor2 = gpioa.pa7.into_analog();
    let mut adc = Adc::adc1(p.ADC1, true, AdcConfig::default());

    let mut red_led = gpiob.pb4.into_push_pull_output();
    let mut green_led = gpiob.pb10.into_push_pull_output();
    let mut blue_led = gpioa.pa8.into_push_pull_output();

    let reset_button = gpiob.pb0.into_input();

    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(36.MHz()).freeze();
    let mut delay = cortex_m::Peripherals::take().unwrap().SYST.delay(&clocks);

    let mut timer = p.TIM1.counter_us(&clocks);

    #[allow(clippy::empty_loop)]
    let mut time_over_boundary = 0;
    loop {
        let mut sample1 = adc.convert(&sensor1, SampleTime::Cycles_480) / 4;
        let mut sample2 = adc.convert(&sensor2, SampleTime::Cycles_480) / 4;
        // rprintln!("{} | {}", sample1, sample2);

        if sample1 > 800 && sample2 > 800 {
            time_over_boundary += 1;
        } else {
            time_over_boundary = 0;
        }

        if time_over_boundary > 3000 {
            time_over_boundary = 0;
            red_led.set_low();
            green_led.set_high();
            blue_led.set_low();
            while sample1 > 800 && sample2 > 800 {
                sample1 = adc.convert(&sensor1, SampleTime::Cycles_480) / 4;
                sample2 = adc.convert(&sensor2, SampleTime::Cycles_480) / 4;
            }

            delay.delay_ms(10);

            let mut time = 0.0;

            timer.start(1.millis()).unwrap();
            let mut blink_at = 0;
            while sample1 < 800 || sample2 < 800 || time < 0.001 {
                if timer.wait().is_ok() {
                    time += 0.001;
                    sample1 = adc.convert(&sensor1, SampleTime::Cycles_480) / 4;
                    sample2 = adc.convert(&sensor2, SampleTime::Cycles_480) / 4;
                    blink_at += 1;
                    if blink_at == 40 {
                        green_led.toggle();
                        blink_at = 0;
                    }
                }
            }
            timer.cancel().unwrap();

            rprintln!("{:.3}", time);

            red_led.set_high();
            green_led.set_high();
            blue_led.set_low();
            while sample1 > 800 && sample2 > 800 {
                sample1 = adc.convert(&sensor1, SampleTime::Cycles_480) / 4;
                sample2 = adc.convert(&sensor2, SampleTime::Cycles_480) / 4;
            }

            while !reset_button.is_high() {}
        } else if time_over_boundary > 10 {
            red_led.set_high();
            green_led.set_high();
            blue_led.set_low();
        } else {
            red_led.set_high();
            green_led.set_low();
            blue_led.set_low();
        }
    }
}
