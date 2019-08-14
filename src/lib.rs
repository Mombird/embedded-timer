// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

#![no_std]

/// For representing buttons
pub mod button;
/// For using the system clock to keep track of time in a loop
pub mod systick;

/// Represents time in milliseconds
pub type Milliseconds = u32;


use button::{Buttons, ButtonEvent};
use systick::Systick;
use f3::led::Leds;

/// Tracks timer state
pub struct SimpleTimer {
    start_button: Buttons,
    time_button: Buttons,
    was: Milliseconds,
    display: CompassDisplay,
    is_running: bool,
    time_remaining: Milliseconds,
    /// Length of time each LED represents
    period: Milliseconds,
}

const LONG_ON: Milliseconds = 600;
const LONG_OFF: Milliseconds = 200;
const SHORT_ON: Milliseconds = 133;
const SHORT_OFF: Milliseconds = 400;
const BLINK: Milliseconds = 500;

impl SimpleTimer {
    /// Create a new SimpleTimer
    pub fn new(start: Buttons, time: Buttons, leds: Leds, period: Milliseconds) -> Self {
        Self {
            start_button: start,
            time_button: time,
            was: 0,
            display: CompassDisplay::new(leds, period),
            is_running: false,
            time_remaining: 0,
            period: period,
        }
    }

    /// Start the SimpleTimer's runloop
    pub fn run(&mut self, clock: &mut Systick) {
        loop {
            self.update(clock.now());
            clock.wait_til_wrapped();
        }
    }

    /// Update the state of the SimpleTimer
    pub fn update(&mut self, now: Milliseconds) {
        if self.is_running {
            self.time_remaining = self.time_remaining
                .saturating_sub(now.wrapping_sub(self.was));
        }
        if ButtonEvent::Push == self.start_button.update(now) {
            self.is_running = ! self.is_running;
        }
        if ButtonEvent::Push == self.time_button.update(now) {
            self.add_time();
        }
        self.display.update(now, self.is_running, self.time_remaining);
        self.was = now;
    }

    /// Add `self.period`ms to self.time_remaining, up to the maximum.
    fn add_time(&mut self) {
        let max = self.period * 8;
        if self.time_remaining >= max {
            self.time_remaining = self.period;
        } else {
            self.time_remaining += self.period;
            if self.time_remaining > max {
                self.time_remaining = max;
            }
        }
    }
}

/// Use the ring of 8 LEDs as a display.
pub struct CompassDisplay {
    leds: Leds,
    next_toggle: Milliseconds,
    last_was_on: bool,
    period: Milliseconds,
    short_time: Milliseconds,
    num_on: u8,
}

impl CompassDisplay {
    pub fn new(mut leds: Leds, period: Milliseconds) -> CompassDisplay {
        for led in leds.iter_mut() {
            led.off()
        }
        CompassDisplay {
            leds: leds,
            next_toggle: 0,
            last_was_on: true,
            period: period,
            short_time: period / 3,
            num_on: 0,
        }
    }

    /// Updates display.
    ///
    /// # Panics
    /// Panics can occur if time_remaining is ever more than 
    /// 8 * self.period.
    pub fn update(&mut self, now: Milliseconds, is_running: bool, time_remaining: Milliseconds) {
        // if we're going off
        if is_running && 0 == time_remaining {
                self.blink(now);
                return;
        } else if ! is_running && 0 == time_remaining {
            for led in self.leds.iter_mut() {
                led.off();
            }
        } else {
            let rem = time_remaining % self.period;
            let div = (time_remaining / self.period) as u8;

            if div != self.num_on {
                if div > self.num_on {
                    for i in (self.num_on.saturating_sub(1))..div {
                        self.leds[i as usize].on();
                    }
                    if 0 != rem && self.last_was_on {
                        self.leds[div as usize].on();
                    }
                } else { // div < self.num_on
                    for i in (div)..=(self.num_on) {
                        self.leds[i as usize].off();
                    }
                    self.last_was_on = false;
                    self.next_toggle = now;
                }

                self.num_on;
            }
            if rem != 0 {self.toggle_led(now, time_remaining);}
        }
    }

    fn toggle_led(&mut self, now: Milliseconds, time_remaining: Milliseconds) {
        if now >= self.next_toggle {
            if self.last_was_on {
                self.leds[self.num_on as usize].off();
                self.next_toggle += if (time_remaining % self.period) <= self.short_time {
                    SHORT_OFF
                } else {
                    LONG_OFF
                }
            } else {
                self.leds[self.num_on as usize].on();
                self.next_toggle += if (time_remaining % self.period) <= self.short_time {
                    SHORT_ON
                } else {
                    LONG_ON
                }
            }
            self.last_was_on = !self.last_was_on;
        }
    }

    fn blink(&mut self, now: Milliseconds) {
        if now >= self.next_toggle {
            if self.last_was_on {
                for led in self.leds.iter_mut() {
                    led.off()
                }
            } else {
                for led in self.leds.iter_mut() {
                    led.on()
                }
            }
            self.last_was_on = !self.last_was_on;
            self.next_toggle += BLINK;
        }
    }
}
