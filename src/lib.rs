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
use f3::led::{Leds,Led};

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
    /// How much of the latest period to spend blinking quickly
    short_time: Milliseconds,
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
            display: CompassDisplay::new(leds),
            is_running: false,
            time_remaining: 0,
            period: period,
            short_time: period / 3,
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
        self.update_display(now);
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

    fn update_display(&mut self, now: Milliseconds) {
        let til_next_solid = (self.time_remaining % self.period) as usize;
        let num_solid_leds = (self.time_remaining / self.period) as usize;
        match til_next_solid {
            0 if self.is_running => {
                // if we're just transitioning to a new solid LED
                if self.time_remaining > 0 {
                    self.display.update(now, num_solid_leds - 1, BlinkKind::Slow);
                } else { // if time is up
                    self.display.update(now, 0, BlinkKind::All);
                }
            },
            0   => self.display.update(now, num_solid_leds, BlinkKind::None),
            x if x <= (self.short_time as usize)    => self.display.update(now, num_solid_leds, BlinkKind::Fast),
            _   => self.display.update(now, num_solid_leds, BlinkKind::Slow),
        }
    }
}

/// A blinking LED
struct Blinky {
    led_idx: Option<usize>,
    is_on: bool,
    short_on: Milliseconds,
    short_off: Milliseconds,
    long_on: Milliseconds,
    long_off: Milliseconds,
    next_toggle: Milliseconds,
}

impl Blinky {
    fn new(idx: Option<(usize, &mut Leds)>,
        short_on: Milliseconds,
        short_off: Milliseconds,
        long_on: Milliseconds,
        long_off: Milliseconds
           ) -> Blinky {
        Blinky {
            led_idx: idx.map(|(i,l)| {l[i].off(); i}),
            is_on: false,
            short_on: short_on,
            short_off: short_off,
            long_on: long_on,
            long_off: long_off,
            next_toggle: 0,
        }
    }

    /// Blink the last LED of a group
    fn update_seq(&mut self, now: Milliseconds, leds: &mut Leds, led_idx: Option<usize>, is_fast: bool) {
        // Depending on whether we are, and were, blinking an led
        match (self.led_idx, led_idx) {
            (None, None)    => (), // nothing was or is happening, so nothing needs to.
            (None, Some(old))   => {
                // we're stopping blinking, so turn the old one off
                self.is_on = Self::set_led(&mut leds[old], true);
            },
            (Some(new), None)   => {
                // we're blinking an led where we weren't before.
                // New led is assumed to start as off
                self.is_on = false;
                self.next_toggle = now;
                self.toggle(&mut leds[new], is_fast);
            },
            (Some(new), Some(old)) if old == new    => {
                // we're continuing to blink the same LED
                if now >= self.next_toggle {
                    self.toggle(&mut leds[new], is_fast);
                }
            },
            (Some(new), Some(_))  =>{
                // we're changing which LED we blink
                // new LED should be opposite of old one
                self.is_on = !self.is_on;
                // make sure the next toggle time will be appropriate
                self.next_toggle = now;
                self.toggle(&mut leds[new], is_fast);
            },
        } //~ end match (led_idx, self.led_idx)
        self.led_idx = led_idx;
    } //~ end fn Blinky.update


    /// Turn the current LED off or on, returning its status.
    /// # Return
    /// `true` if the LED is on
    fn set_led(led: &mut Led, off: bool) -> bool {
        if off {
            led.off();
        } else {
            led.on();
        }
        !off
    }

    /// Toggle the state of an LED, recording the next time to toggle it at
    fn toggle(&mut self, led: &mut Led, is_fast: bool) {
        self.is_on = Self::set_led(led, self.is_on);
        self.next_toggle += match (is_fast, self.is_on) {
            (true, true)    => self.short_on,
            (true, false)   => self.short_off,
            (false, false)  => self.long_off,
            (false, true)   => self.long_on,
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlinkKind {
    Fast,
    Slow,
    None,
    All,
}

impl BlinkKind {
    fn to_some(&self, n: usize) -> Option<usize> {
        match self {
            &BlinkKind::Fast | &BlinkKind::Slow   => Some(n),
            &BlinkKind::None | &BlinkKind::All    => None,
        }
    }
}

/// Use the ring of 8 LEDs as a display.
pub struct CompassDisplay {
    leds: Leds,
    next_blink: Option<Milliseconds>,
    blink_on: bool,
    num_on: usize,
    blinky: Blinky,
}

impl CompassDisplay {
    pub fn new(mut leds: Leds) -> CompassDisplay {
        Self::set_all(&mut leds, true);
        CompassDisplay {
            leds: leds,
            next_blink: None,
            blink_on: false,
            num_on: 0,
            blinky: Blinky::new(None, SHORT_ON, SHORT_OFF, LONG_ON, LONG_OFF),
        }
    }

    /// Updates display.
    ///
    /// # Params
    /// * `solid` - The number of leds to be on solid.
    /// * `blink_idx` - Which LED to blink, and how.
    /// # Panics
    /// Will panic if given more than 8 leds to be solid
    pub fn update(&mut self, now: Milliseconds, solid: usize, blink: BlinkKind) {
        assert!(solid <= 8, "we only have 8 leds to be solid!");
        if BlinkKind::All == blink {
            self.blink(now);
        } else {
            if self.num_on != solid || None != self.next_blink {
                if None != self.next_blink {
                    self.blink_on = Self::set_all(&mut self.leds,true);
                    self.next_blink = None;
                }
                // if we're changing 
                if self.num_on != solid {
                    self.num_on = solid;
                }
                for idx in 0..self.num_on {
                    self.leds[idx].on();
                }
                for idx in self.num_on..8 {
                    self.leds[idx].off();
                }
            }

            self.blinky.update_seq(now, &mut self.leds, blink.to_some(solid), BlinkKind::Fast == blink);
        }
    }


    fn set_all(leds: &mut Leds, off: bool) -> bool {
        if off {
            for led in leds.iter_mut() {
                led.off();
            }
        } else {
            for led in leds.iter_mut() {
                led.on();
            }
        }
        !off
    }
    fn blink(&mut self, now: Milliseconds) {
        match self.next_blink {
            None    => {
                self.toggle(now);
            },
            Some(next) if now < next    => (),
            Some(_)  => self.toggle(now),
        }
    }
    fn toggle(&mut self, last: Milliseconds) {
        self.blink_on = Self::set_all(&mut self.leds,self.blink_on);
        self.next_blink = Some(last + BLINK);
    }
}
