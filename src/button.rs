// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use super::Milliseconds;

//TODO: Consult w/ industry consultant on appropriate value
/// Delay separating two single presses from a double-press
const PRESS_BREAK: Milliseconds = 250;

//TODO: Consult w/ industry consultant on appropriate value
/// Delay until a hold is released
const HOLD_BREAK: Milliseconds = 100;

//TODO: Consult w/ industry consultant on appropriate value
/// Delay separating a press from a hold
const HOLD_DELAY: Milliseconds = 750;

//TODO: Consult w/ industry consultant on appropriate value
/// Time to wait after a press to ignore switch bounce
const DEBOUNCE_DELAY: Milliseconds = 20;

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
    poll_limit: Option<Milliseconds>,
    prev_presses: u8,
    holding: bool,
    button: BTN,
}

impl<BTN: PushButton> Button<BTN> {
    pub fn new(button: BTN) -> Button<BTN> {
        Button {
            last_state: false,
            last_change_time: 0,
            poll_limit: None,
            prev_presses: 0,
            holding: false,
            button: button,
        }
    }

    pub fn update(&mut self, now: Milliseconds) -> Option<ButtonEvent> {
        if let Some(s) = self.poll_limit {
            if s < now {
                return None
            } else {
                self.poll_limit = None;
            }
        }
        let current_state = self.button.is_pressed();
        let duration = now - self.last_change_time;

        if current_state {
            // button is pressed
            if self.last_state {
                // button is *still* pressed
                if duration >= HOLD_DELAY {
                    // button has been pressed long enough to count as being 
                    // held
                    self.holding = true;
                    Some(ButtonEvent::Hold(self.prev_presses))
                } else {
                    None
                }
            } else { // !self.last_state
                if duration < PRESS_BREAK {
                    self.prev_presses += 1;
                }
                self.last_state = true;
                self.last_change_time = now;
                None
            } // fi self.last_state
        } else { // !current_state
            if self.last_state {
                self.last_state = false;
                self.last_change_time = now;
                self.poll_limit = Some(now + DEBOUNCE_DELAY);
                None
            } else { // !self.last_state
                if self.holding {
                    if duration >= HOLD_BREAK {
                        self.holding = false;
                        self.prev_presses = 0;
                        // Some(ButtonEvent::Release)
                        None
                    } else {
                        // None
                        Some(ButtonEvent::Hold(self.prev_presses))
                    }
                } else { // !self.holding
                    if duration >= PRESS_BREAK {
                        let presses = self.prev_presses + 1;
                        self.prev_presses = 0;
                        Some(ButtonEvent::Press(presses))
                    } else {
                        None
                    }
                } // fi self.holding
            } // fi self.last_state
        } // fi current_state
    } // end fn Button.update
} // end impl<BTN: PushButton> Button<BTN>
