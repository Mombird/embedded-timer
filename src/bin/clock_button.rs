// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

#![no_std]
#![no_main]
#![allow(deprecated)]

extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use timer::button::{ButtonEvent, Buttons};
use timer::systick;
use timer::Milliseconds;

use f3::hal::prelude::*;
use f3::hal::stm32f30x;

// use f3::hal::gpio::gpioa;
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

    // enable (power on) button
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

    // set button as input, floating
    let pa0 = gpioa
        .pa0
        .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);

    let leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));
    // for led in leds.iter_mut() {
    //     led.off();
    // }

    let mut snake = LedSnake::new(leds, Buttons::pa0(pa0, 0), 1000, 500, 4);

    // set up system timer using default settings of 8 MHz
    let hal_clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut systick = systick::Systick::new(cp.SYST, hal_clocks, 6).unwrap();

    loop {
        snake.update(systick.now());
        systick.wait_til_wrapped();
    }
}

struct LedSnake {
    leds: Leds,
    on_idx: usize,
    off_idx: usize,
    button: Buttons,
    period: Milliseconds,
    next_on: Milliseconds,
    next_off: Milliseconds,
    running: bool,
}

impl LedSnake {
    fn new(
        leds: Leds,
        button: Buttons,
        period: Milliseconds,
        offset: Milliseconds,
        max_on: u8,
    ) -> LedSnake {
        LedSnake {
            leds,
            on_idx: 0,
            off_idx: 0,
            button,
            period,
            next_on: 0,
            next_off: period * (max_on - 1) as Milliseconds + offset,
            running: true,
        }
    }

    fn update(&mut self, now: Milliseconds) {
        if self.running {
            if now >= self.next_on {
                self.leds[self.on_idx].on();
                self.next_on += self.period;
                if self.on_idx < 7 {
                    self.on_idx += 1;
                } else {
                    self.on_idx = 0;
                }
            }
            if now >= self.next_off {
                self.leds[self.off_idx].off();
                self.next_off += self.period;
                if self.off_idx < 7 {
                    self.off_idx += 1;
                } else {
                    self.off_idx = 0;
                }
            }
        }
        // Doing this last means now CANNOT be greater then
        // self.next_{on,off}.
        if ButtonEvent::Push == self.button.update(now) {
            self.toggle(now);
        }
    }

    fn toggle(&mut self, now: Milliseconds) {
        if self.running {
            self.next_on -= now;
            self.next_off -= now;
        } else {
            self.next_on += now;
            self.next_off += now;
        }
        self.running = !self.running;
    }
}
