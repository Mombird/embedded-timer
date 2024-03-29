// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

#![no_std]
#![no_main]
#![allow(deprecated)]

/// The button on the discovery board lights up the leds one by one till all lit
/// then turns them off one by one.
///
/// The button on the proto board beeps the buzzer
extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use timer::button::Button;

use timer::systick;

use f3::hal::stm32f30x;

use f3::hal::gpio::gpioc::PC1;
use f3::hal::gpio::{Floating, Input};
use f3::hal::prelude::*;
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
    let mut systick = systick::Systick::new(cp.SYST, hal_clocks, 20).unwrap();

    // enable (power on) buttons
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);

    // set both buttons
    let pa0 = gpioa
        .pa0
        .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);
    let mut discovery_button = Button::new(pa0, 0);

    let pc1 = gpioc
        .pc1
        .into_floating_input(&mut gpioc.moder, &mut gpioc.pupdr);
    let mut knob_button: Button<PC1<Input<Floating>>> = Button::new(pc1, 0);

    // initialize buzzer
    let mut buzzer = gpioc
        .pc3
        .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

    // initialize leds
    let mut index = 0;
    let mut leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));

    loop {
        // if the knob button is pressed, beep the buzzer till
        // the knob button is released
        if knob_button.update(systick.now()).is_pressed() {
            buzzer.set_high();
            while knob_button.update(systick.now()).is_pressed() {
                systick.wait_til_wrapped();
            }
            buzzer.set_low();
        }

        // change the state of the next led if user button pressed
        // then wate till the button is released
        if discovery_button.update(systick.now()).is_pressed() {
            index = update_leds(&mut leds, index);
            while discovery_button.update(systick.now()).is_pressed() {
                systick.wait_til_wrapped();
            }
        } else {
            systick.wait_til_wrapped();
        }
    }
}

//  update_leds
//  turns on or off the next led,
//  starting at 0 it turns on the next led,
//  for index's from 8-15 it turns off the next led
//
//  if idx is larger than 15, it turns off all the leds
//  returns the next index
fn update_leds(leds: &mut Leds, index: usize) -> usize {
    match index {
        idx if idx < 8 => leds[idx].on(),
        idx if idx < 16 => leds[idx - 8].off(),
        _ => {
            for led in leds.iter_mut() {
                led.off();
                return 0;
            }
        }
    }

    // calculate the index of the next led to change
    (index + 1) % 16
}
