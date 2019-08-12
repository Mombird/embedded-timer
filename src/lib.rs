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
    clock: Systick,
    leds: Leds,
    is_running: bool,
    time_remaining: Milliseconds,
    /// Length of time each LED represents
    period: Milliseconds,
}

const LONG_ON: Milliseconds = 600;
const LONG_OFF: Milliseconds = 200;
const SHORT_ON: Milliseconds = 133;
const SHORT_OFF: Milliseconds = 400;

impl SimpleTimer {
    pub fn new(start: Buttons, time: Buttons, clock: Systick, leds: Leds, period: Milliseconds) -> Self {
        Self {
            start_button: start,
            time_button: time,
            clock: clock,
            leds: leds,
            is_running: false,
            time_remaining: 0,
            period: period,
        }
    }
}

