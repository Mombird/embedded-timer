// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

#![no_std]


/// For representing buttons
pub mod button;
/// For using the system clock to keep track of time in a loop
pub mod systick;

pub type Milliseconds = u32;
