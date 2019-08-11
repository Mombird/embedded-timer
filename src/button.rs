// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use super::Milliseconds;
use f3::hal::gpio::Input;

//TODO: Consult w/ industry consultant on appropriate value
/// Delay separating two single presses from a double-press
const PRESS_BREAK: Milliseconds = 250;

//TODO: Consult w/ industry consultant on appropriate value
/// Delay separating a double-press from a bounce
const DEBOUNCE_DELAY: Milliseconds = 100;

//TODO: Consult w/ industry consultant on appropriate value
/// Delay separating a press from a hold
const HOLD_DELAY: Milliseconds = 750;

//TODO: Consult w/ industry consultant on appropriate value
/// Minimum press length to be counted
const MIN_PRESS: Milliseconds = 50;

/// A button that can be pressed or not.
pub trait PushButton {
    fn is_pressed(&self) -> bool;
}

/// Represents a button event.
pub enum ButtonEvent {
    /// Button pressed. u8 is number of presses (double, triple, etc.)
    Press(u8),
    /// Button being held. u8 is preceding number of presses (v^,v^^^^^ 
    /// would be Hold(1)).
    Hold(u8),
    // /// Button hold released.
    // Release,
}

pub struct Button<BTN> {
    last_state: bool, // true if pressed
    last_change_time: Milliseconds,
    prev_presses: u8,
    holding: bool,
    pressing: bool,
    button: BTN,
}

impl<BTN: PushButton> Button<BTN> {
    pub fn new(button: BTN) -> Button<BTN> {
        Button {
            last_state: false,
            last_change_time: 0,
            prev_presses: 0,
            holding: false,
            pressing: false,
            button: button,
        }
    }
}
