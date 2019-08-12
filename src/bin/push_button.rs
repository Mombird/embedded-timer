// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

#![no_std]
#![no_main]
#![allow(deprecated)]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

// not currently using Delay
// use f3::hal::delay::Delay;
use f3::hal::prelude::*;
use f3::hal::stm32f30x;

use f3::hal::gpio::gpioa::PA0;
use f3::hal::gpio::{Floating, Input};
use f3::led::Leds;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // get processor and discovery board peripherals
    // not using time here
    // let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    // not using time here
    // let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // not using time here
    // set up system timer using default settings of 8 MHz
    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // let delay = Delay::new(cp.SYST, clocks);

    // enable (power on) button
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    // set button as input, floating
    let pa0 = gpioa
        .pa0
        .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);

    let mut leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));

    loop {
        // turn on a light with each press
        for led in leds.iter_mut() {
            wait_till_pressed(&pa0);
            led.on();
        }

        // turn off a light with each press
        for led in leds.iter_mut() {
            wait_till_pressed(&pa0);
            led.off();
        }
    }
}

// wait till the user button is pressed then released
pub fn wait_till_pressed(button: &PA0<Input<Floating>>) {
    // loop till you get a press
    while button.is_low() {}

    // now loop til not pressed
    while button.is_high() {}
}
