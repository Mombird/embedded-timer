// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! creates a snake of leds that move around the circle in a
//! clockwise fashion.  This code test the systick timer code rather
//! than using a simple delay function

#![no_std]
#![no_main]
#![allow(deprecated)]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use timer::systick;

use f3::hal::prelude::*;
use f3::hal::stm32f30x;

// use f3::hal::gpio::gpioa::PA0;
// use f3::hal::gpio::{Input, Floating};
use f3::led::Leds;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // get processor and discovery board peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // enable (power on) leds
    let mut leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));
    let num_leds = &leds.len();

    // set up system timer using default settings of 8 MHz
    let hal_clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut systick = systick::Systick::new(cp.SYST, hal_clocks, 6).unwrap();

    let led_period = 1000;
    let toggle_offset = 500;
    let start_delay = 1000;
    let off_delay = 3000;

    let mut on_idx = 0;
    let mut off_idx = 0;
    let mut next_on = start_delay;
    let mut next_off = next_on + off_delay + toggle_offset;

    loop {
        // Turn on the next led every second
        if systick.now() >= next_on {
            leds[on_idx].on();
            next_on += led_period;
            if on_idx < num_leds - 1 {
                on_idx += 1;
            } else {
                on_idx = 0;
            }
        }

        // Turn off the next led every second
        if systick.now() >= next_off {
            leds[off_idx].off();
            next_off += led_period;
            if off_idx < num_leds - 1 {
                off_idx += 1;
            } else {
                off_idx = 0;
            }
        }
        systick.wait_til_wrapped();
    }
}
