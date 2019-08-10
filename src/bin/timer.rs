// Copyright © 2019 Robin Gearn
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.


#![no_std]
#![no_main]
#![allow(deprecated)]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use timer::systick;

use f3::hal::prelude::*;
use f3::hal::stm32f30x;

use f3::hal::gpio::gpioa::PA0;
use f3::hal::gpio::{Input, Floating};
use f3::led::Leds;


use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // get processor and discovery board peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // set up system timer using default settings of 8 MHz
    let hal_clocks = rcc.cfgr.freeze(&mut flash.acr);
    // let mut systick = systick::Systick::new(cp.SYST, hal_clocks, 50);
    let mut sysclock = systick::Sysclock::new(cp.SYST, hal_clocks);

    // enable (power on) button
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);


    // set button as input, floating
        let pa0 = gpioa.pa0.into_floating_input
            (&mut gpioa.moder, &mut gpioa.pupdr);

    let mut leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));
    let num_leds = &leds.len();


    let led_period = sysclock.freq() as u64;
    let toggle_offset = sysclock.freq() as u64 / 2;
    let start_delay = sysclock.freq() as u64 / 100;
    let off_delay = led_period * 3;
    
    let now = sysclock.now();
    let mut next_on = now + start_delay;
    let mut next_off = next_on + off_delay + toggle_offset;
    let mut on_idx = 0;
    let mut off_idx = 0;


    loop {
        // Turn on the next led every second
        if sysclock.now() >= next_on {
            leds[on_idx].on();
            next_on += led_period;
            if on_idx < num_leds - 1 {
                on_idx += 1;
            } else {
                on_idx = 0;
            }
        }

        // Turn off the next led every second
        if sysclock.now() >= next_off {
            leds[off_idx].off();
            next_off += led_period;
            if off_idx < num_leds - 1 {
                off_idx += 1;
            } else {
                off_idx = 0;
            }
        }
    }
}

