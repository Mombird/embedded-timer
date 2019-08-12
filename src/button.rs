// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use super::Milliseconds;
use f3::hal::gpio::gpioa::PA0;
use f3::hal::gpio::gpioc::PC1;
use f3::hal::gpio::{Floating, Input};
use f3::hal::prelude::*;

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
/// Time to wait after state change to ignore switch bounce
const DEBOUNCE_DELAY: Milliseconds = 50;

/// A button that can be pressed or not.
pub trait PushButton {
    fn is_pressed(&self) -> bool;
}


// implement PushButton for both buttons used in this application

impl PushButton for PA0<Input<Floating>> {
    fn is_pressed(&self) -> bool {

        // embedded_hal::digital::v1 is deprecated; stuck until 
        // stm32f30x-hal updates
#[allow(deprecated)]
        // board button is high when pushed
        self.is_high()
    }
}

impl PushButton for PC1<Input<Floating>> {
    fn is_pressed(&self) -> bool {

        // embedded_hal::digital::v1 is deprecated; stuck until 
        // stm32f30x-hal updates
#[allow(deprecated)]
        // encoder button is low when pushed
        self.is_low()
    }
}

/// Represents a button event.
#[derive(Clone, Copy, PartialEq)]
pub enum ButtonEvent {
    /// Button pressed
    Push,
    /// Button held
    Pressed,
    /// Button released
    Release,
    /// Button not pushed
    NotPressed,
}

impl ButtonEvent {
    /// `true` if event corresponds to button being pressed
    pub fn is_pressed(&self) -> bool {
        use ButtonEvent::*;
        match self {
            &Push | &Pressed  => true,
            &Release | &NotPressed    => false,
        }
    }

    /// `true` if event corresponds to a change in state
    pub fn is_change(&self) -> bool {
        use ButtonEvent::*;
        match self {
            &Push | &Release    => true,
            &Pressed | &NotPressed  => false,
        }
    }
}

/// A struct to represent an (optionally) debounced button inside a clocked 
/// loop.
pub struct Button<BTN> {
    last_state: ButtonEvent,
    debounce_delay: Option<Milliseconds>,
    debouncing_till: Option<Milliseconds>,
    button: BTN,
}

impl<BTN: PushButton> Button<BTN> {
    /// Create a new Button.
    /// #Params
    /// `button` - The button this is representing.
    /// `debounce` - The amount of time to ignore the button after a state 
    /// change. Set to 0 (or less, if `Milliseconds` is signed) to disable 
    /// debouncing.
    pub fn new(button: BTN, debounce: Milliseconds) -> Button<BTN> {
        use ButtonEvent::*;
        // let state = if button.is_pressed() { Pressed } else { NotPressed };
        Button {
            last_state: NotPressed,
            debounce_delay: if 0 >= debounce { None } else { Some(debounce) },
            debouncing_till: None,
            button: button,
        }
    }

    /// Check the button state (if not debouncing) and return the current 
    /// state as a `ButtonEvent`.
    ///
    /// #Params
    /// `now` - The current time in milliseconds. Note that this *must* be 
    /// a reasonably accurate representation of the actual time for the 
    /// debouncing to work as expected.
    pub fn update(&mut self, now: Milliseconds) -> ButtonEvent {
        use ButtonEvent::*;
        if self.debounce(now) {
            return self.last_state;
        }

        match (self.last_state.is_pressed(), self.button.is_pressed()) {
            // if button was pressed and is still pressed
            (true, true)    => Pressed,
            // if button was not pressed and is still not pressed
            (false, false)  => NotPressed,
            // if button was not pressed and now is
            (false, true)   => {
                // set debounce delay
                self.set_debounce(now);
                self.last_state = Pressed;
                Push
            },
            // if button was pressed and now is not
            (true, false)   => {
                self.set_debounce(now);
                self.last_state = NotPressed;
                Release
            },
        }
    }

    /// Convenience function. Sets self to return last button state without 
    /// polling the button until `now + DEBOUNCE_DELAY`.
    fn set_debounce(&mut self, now: Milliseconds) {
        self.debouncing_till = self.debounce_delay.map(|d| now + d);
    }

    /// Handles debounce delay.
    /// 
    /// #Returns
    /// `bool` - `true` if we're waiting for debounce period
    fn debounce(&mut self, now: Milliseconds) -> bool {
        match self.debouncing_till {
            None    => false,
            Some(s) if s < now  => true,
            Some(_) => {
                self.debouncing_till = None;
                false
            },
        }
    }
}


/// Represents a fancy button event.
#[derive(PartialEq)]
pub enum MultiButtonEvent {
    /// Button pressed. u8 is number of presses (double, triple, etc.)
    Press(u8),
    /// Button being held. u8 is preceding number of presses (v^,v^^^^^
    /// would be Hold(1)).
    Hold(u8),
    /// Button hold released.
    Release,
}

pub struct FancyButton<BTN> {
    last_state: bool, // true if pressed
    last_change_time: Milliseconds,
    debouncing_till: Option<Milliseconds>,
    prev_presses: u8,
    holding: bool,
    button: BTN,
}

impl<BTN: PushButton> FancyButton<BTN> {
    pub fn new(button: BTN) -> FancyButton<BTN> {
        FancyButton {
            last_state: false,
            last_change_time: 0,
            debouncing_till: None,
            prev_presses: 0,
            holding: false,
            button: button,
        }
    }

    pub fn update(&mut self, now: Milliseconds) -> Option<MultiButtonEvent> {
        if let Some(s) = self.debouncing_till {
            if s < now {
                return None;
            } else {
                self.debouncing_till = None;
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
                    Some(MultiButtonEvent::Hold(self.prev_presses))
                } else {
                    None
                }
            } else {
                // !self.last_state
                if duration < PRESS_BREAK {
                    self.prev_presses += 1;
                }
                self.last_state = true;
                self.last_change_time = now;
                None
            } // fi self.last_state
        } else {
            // !current_state
            if self.last_state {
                self.last_state = false;
                self.last_change_time = now;
                self.debouncing_till = Some(now + DEBOUNCE_DELAY);
                None
            } else {
                // !self.last_state
                if self.holding {
                    if duration >= HOLD_BREAK {
                        self.holding = false;
                        self.prev_presses = 0;
                        // Some(MultiButtonEvent::Release)
                        None
                    } else {
                        // None
                        Some(MultiButtonEvent::Hold(self.prev_presses))
                    }
                } else {
                    // !self.holding
                    if duration >= PRESS_BREAK {
                        let presses = self.prev_presses + 1;
                        self.prev_presses = 0;
                        Some(MultiButtonEvent::Press(presses))
                    } else {
                        None
                    }
                } // fi self.holding
            } // fi self.last_state
        } // fi current_state
    } // end fn Button.update
} // end impl<BTN: PushButton> Button<BTN>
