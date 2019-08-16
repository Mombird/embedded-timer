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

use button::{ButtonEvent, Buttons};
use f3::led::{Led, Leds};

/// Tracks timer state
pub struct SimpleTimer {
    start_button: Buttons,
    time_button: Buttons,
    /// The last time this updated
    was: Milliseconds,
    display: CompassDisplay,
    is_running: bool,
    time_remaining: Milliseconds,
    /// Length of time each LED represents
    period: Milliseconds,
    /// How much of the latest period to spend blinking quickly
    fast_time: Milliseconds,
}

const LONG_ON: Milliseconds = 1250;
const LONG_OFF: Milliseconds = 750;
const SHORT_ON: Milliseconds = 625;
const SHORT_OFF: Milliseconds = 375;
const BLINK: Milliseconds = 600;

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
            period,
            fast_time: period / 3,
        }
    }

    /*
     * /// Start the SimpleTimer's runloop
     * pub fn run(&mut self, clock: &mut Systick) {
     *     loop {
     *         self.update(clock.now());
     *         clock.wait_til_wrapped();
     *     }
     * }
     */

    /// Update the state of the SimpleTimer
    pub fn update(&mut self, now: Milliseconds) {
        if self.is_running {
            self.time_remaining = self
                .time_remaining
                // using saturating sub to avoid panic
                // by using wrapping_sub, this should also work with
                // non-timer timekeeper
                .saturating_sub(now.wrapping_sub(self.was));
        }
        if ButtonEvent::Push == self.start_button.update(now) {
            self.is_running = !self.is_running;
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
        // important for figuring out how fast to blink and whether the
        // division truncated anything
        let til_next_period = (self.time_remaining % self.period) as usize;
        // the number of whole periods remaining
        let whole_periods = (self.time_remaining / self.period) as usize;

        match til_next_period {
            // exactly at time and running
            0 if self.is_running => {
                // if we're just transitioning to a new solid LED
                if self.time_remaining > 0 {
                    self.display.update(now, whole_periods - 1, BlinkKind::Slow);
                } else {
                    // if time is up
                    self.display.update(now, 0, BlinkKind::All);
                }
            }

            // exactly at time and __not__ running
            0 => self.display.update(now, whole_periods, BlinkKind::None),
            // not exactly at time and not running, either
            _ if !self.is_running => {
                // display any partial seconds as solidly on
                self.display.update(now, whole_periods + 1, BlinkKind::None);
            }

            // running and with a fast time left on the latest interval
            x if x <= (self.fast_time as usize) => {
                self.display.update(now, whole_periods, BlinkKind::Fast);
            }

            // running and with plenty of time left on the latest interval
            _ => self.display.update(now, whole_periods, BlinkKind::Slow),
        }
    }
}

/// A blinking LED
struct Blinky {
    /// Which led to blink, if any.
    led_idx: Option<usize>,
    /// Whether the led is currently on
    is_on: bool,
    /// How long to stay on when blinking fast
    fast_on: Milliseconds,
    /// How long to stay off when blinking fast
    fast_off: Milliseconds,
    /// How long to stay on when blinking slow
    slow_on: Milliseconds,
    /// How long to stay off when blinking slow
    slow_off: Milliseconds,
    /// When next to toggle
    next_toggle: Milliseconds,
}

impl Blinky {
    fn new(
        idx: Option<(usize, &mut Leds)>,
        fast_on: Milliseconds,
        fast_off: Milliseconds,
        slow_on: Milliseconds,
        slow_off: Milliseconds,
    ) -> Blinky {
        Blinky {
            led_idx: idx.map(|(i, l)| {
                l[i].off();
                i
            }),
            is_on: false,
            fast_on,
            fast_off,
            slow_on,
            slow_off,
            next_toggle: 0,
        }
    }

    /// Blink the last LED of a group
    fn update_seq(
        &mut self,
        now: Milliseconds,
        leds: &mut Leds,
        led_idx: Option<usize>,
        is_fast: bool,
    ) {
        // Depending on whether we are, and were, blinking an led
        match (led_idx, self.led_idx) {
            (None, None) => (), // nothing was or is happening, so nothing needs to.
            (None, Some(old)) => {
                // we're stopping blinking, so turn the old one off
                self.is_on = Self::set_led(&mut leds[old], true);
            }
            (Some(new), None) => {
                // we've begun blinking. where we weren't before.
                // Assume the led is on to begin with
                self.is_on = true;
                self.next_toggle = now;
                self.toggle(&mut leds[new], is_fast);
            }
            (Some(new), Some(old)) if old == new => {
                // we're continuing to blink the same LED
                if now >= self.next_toggle {
                    self.toggle(&mut leds[new], is_fast);
                }
            }
            (Some(new), Some(_)) => {
                // we're changing which LED we blink
                // new LED should be opposite of old one
                self.is_on = !self.is_on;
                // make sure the next toggle time will be appropriate
                self.next_toggle = now;
                self.toggle(&mut leds[new], is_fast);
            }
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
            (true, true) => self.fast_on,
            (true, false) => self.fast_off,
            (false, false) => self.slow_off,
            (false, true) => self.slow_on,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlinkKind {
    /// Blink fast
    Fast,
    /// Blink slow
    Slow,
    /// Don't blink
    None,
    /// Blink all LEDs
    All,
    // /// Blink as many LEDs as would otherwise be on or blinking
    // Partial,
}

impl BlinkKind {
    fn to_some(self, n: usize) -> Option<usize> {
        match self {
            BlinkKind::Fast | BlinkKind::Slow => Some(n),
            BlinkKind::None | BlinkKind::All => None,
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
            leds,
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
        // what we do depends on how we're blinking
        // If we're blinking all LEDs, that's all we need to worry about.
        if BlinkKind::All == blink {
            self.blink(now);
        } else {
            // If we just stopped blinking all LEDs, or changed the number
            // that are on solid, make sure to re-assert correct status
            if self.num_on != solid || None != self.next_blink {
                // If we stopped blinking all
                if None != self.next_blink {
                    // turn all LEDs off and record them as being such
                    self.blink_on = Self::set_all(&mut self.leds, true);
                    // record that we've stopped blinking
                    self.next_blink = None;
                }
                // if we're changing the number of solid LEDs
                if self.num_on != solid {
                    // mark the change
                    self.num_on = solid;
                }
                // turn on the LEDs that should be solidly on
                for idx in 0..self.num_on {
                    self.leds[idx].on();
                }
                // turn off the LEDs that shouldn't be solidly on
                for idx in self.num_on..8 {
                    self.leds[idx].off();
                }
            }

            self.blinky.update_seq(
                now,
                &mut self.leds,
                blink.to_some(solid),
                BlinkKind::Fast == blink,
            );
        }
    }

    /// Set all LEDs off or on
    /// # Return
    /// `true` if LEDs were turned on
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
            // We've started blinking
            None => {
                self.toggle(now);
            }
            // It's not time to blink yet
            Some(next) if now < next => (),
            // Time to blink!
            Some(next) => self.toggle(next),
        }
    }
    fn toggle(&mut self, last: Milliseconds) {
        // toggle LEDs and record status
        self.blink_on = Self::set_all(&mut self.leds, self.blink_on);
        // set time of next toggle
        self.next_blink = Some(last + BLINK);
    }
}
