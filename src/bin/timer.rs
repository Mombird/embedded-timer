// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

#![no_std]
#![no_main]
#![allow(deprecated)]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use timer::button::Buttons;
use timer::systick::Systick;
use timer::SimpleTimer;

use f3::hal::prelude::*;
use f3::hal::stm32f30x;

// use f3::hal::gpio::gpioa::PA0;
// use f3::hal::gpio::gpioc::{PC1,PC3};
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

    // enable (power on) buttons
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);

    // set both buttons
    let pa0 = gpioa
        .pa0
        .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);
    let discovery_button = Buttons::pa0(pa0, 0);

    let pc1 = gpioc
        .pc1
        .into_floating_input(&mut gpioc.moder, &mut gpioc.pupdr);
    let knob_button = Buttons::pc1(pc1, 0);

    // initialize buzzer
    // let mut buzzer = gpioc
    //     .pc3
    //     .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

    // initialize leds
    let leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));

    // set up system timer using default settings of 8 MHz
    let hal_clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut systick = Systick::new(cp.SYST, hal_clocks, 6).unwrap();

    let mut timer = SimpleTimer::new(knob_button, discovery_button, leds, 15000);

    loop {
        timer.update(systick.now());
        systick.wait_til_wrapped();
    }
}
