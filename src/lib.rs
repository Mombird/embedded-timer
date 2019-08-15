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
            (Some(new), Some(old))  =>{
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
enum BlinkKind {
    Fast,
    Slow,
    None,
    All,
}

impl BlinkKind {
    fn is_single(&self) -> bool {
        match self {
            &BlinkKind::Fast | &BlinkKind::Slow   => true,
            &BlinkKind::None | &BlinkKind::All    => false,
        }
    }
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
    period: Milliseconds,
    short_time: Milliseconds,
    num_on: usize,
    blinky: Blinky,
}

impl CompassDisplay {
    pub fn new(mut leds: Leds, period: Milliseconds) -> CompassDisplay {
        Self::set_all(&mut leds, true);
        CompassDisplay {
            leds: leds,
            next_blink: None,
            blink_on: false,
            period: period,
            short_time: period / 3,
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
        if None != self.next_blink && BlinkKind::All != blink {
            self.blink_on = Self::set_all(&mut self.leds,true);
            self.next_blink = None;
        }
        // if we're changing 
        if self.num_on != solid {
            for idx in 0..solid {
                self.leds[idx].on();
            }
            for idx in solid..8 {
                self.leds[idx].off();
            }
        }

        self.blinky.update_seq(now, &mut self.leds, blink.to_some(solid), BlinkKind::Fast == blink);
        if BlinkKind::All == blink {
            self.blink(now);
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
            Some(next)  => self.toggle(now),
        }
    }
    fn toggle(&mut self, last: Milliseconds) {
        self.blink_on = Self::set_all(&mut self.leds,self.blink_on);
        self.next_blink = Some(last + BLINK);
    }
}
