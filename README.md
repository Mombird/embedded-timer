# Rust Project Embedded Discovery Board Project
Copyright (c) 2019 Robin Gearn <rgearn@pdx.edu>, James Wescott 
<james@wescottdesign.com>

This project could not have been completed without the [Discovery][discovery] tutorial maintained by the [Embedded Resources team][team]


This project is also supported with the help of Tim Wescott 
<tim@wescottdesign.com>, 

## Project Description
This project runs on a STM32 F3 discovery board and includes 5 different 
programs.

`push_button`, `clock`, and `clock_button` use just the discovery board.
`two_button` and `timer` use the discovery board with a button 
hooked up to pin PC1 and a buzzer hooked to PC3.

### push\_button
This simple program tests the implementation of the `InputPin` trait in the 
stm32f30x-hal crate.  Each press of the user button changes the state of 
the next led in a clockwise manner, starting with the North led. We tested 
this program with multiple loops through the code.  We also tested for 
button bounce and found the button on our board had no need for debounce 
correction.

### clock
The `clock` program is very similar to the roulette project in the 
discovery tutorial.  It turns on an led for 3.5 seconds.  It turns another 
led each second.  This code is designed to use our timer module code rather 
than the simple delay function provided by the stm3f30x-hal crate.  This 
timer code allows things to be effectively running concurrently.

### two\_button
The `two_button` program tests the button module that implements more 
varied button input functionallity.  There was thought put in to correct 
for bounce though both of the buttons we used had no problems with bounce 
and this it is turned off.  The user button has the same functionallity as 
in `push_button`.  The additional knob button will sound the buzzer as long 
as it is pressed.  If one button is pressed down the other button wont 
work.

### clock\_button
`clock_button` has the same functionality as `clock`, with the addition of 
a start-stop button.  It was designed to test our ability to save our state.

### timer
The actual timer program. A simple countdown timer, it counts down until it 
runs out, then starts beeping. Due to the limits of the board, time can 
only be set as up to eight intervals of 15 seconds (the latter number can 
be changed easily). The LEDs give a rough idea of how much time is left, 
flashing faster as it approaches time to move on to the next.

## Development Environment

To build these projects you need 
* cross compilation support for the ARM Cortex-M
* OpenOCD
* a gdb for the cortexm chip. For our development we used `gdb-multiarch` 
  and `arm-none-eabi-gdb`

After cloning the repository, run `git submodule init` and `git submodule 
update`.

To set up the development environment, please follow the instructions in 
the [embedded rust 
book](https://rust-embedded.github.io/book/intro/install.html) or the 
[discovery 
tutorial](https://rust-embedded.github.io/discovery/03-setup/index.html)

Note that if you want to use `cargo run` you may need to alter 
`.cargo/config`, especially if you don't use a debian-based distribution, 
and those who AREN'T using the old version of the discovery board will need 
to edit `openocd.cfg`.

## License

This program is licensed under the "MIT License".  Please
see the file `LICENSE` in the source distribution of this
software for license terms.

[discover]: https://rust-embedded.github.io/discovery/
[team]: https://github.com/rust-embedded/wg#the-resources-team

