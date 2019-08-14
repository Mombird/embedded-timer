# Rust Project Embedded Discovery Board Project
Copyright (c) 2019 Robin Gearn <rgearn@pdx.edu>, James Wescott <james@wescottdesign.com>

<!--TODO:  change this to acknowledgement of discovery tutorial and embedded rust book -->
This project starts with an exploration of A template for building applications for ARM 
    Cortex-M microcontrollers. This project is developed and maintained by the [Cortex-M team][team]


This project is also supported with the help of Tim Wescott <tim@wescottdesign.com>, 

## Project Description
This project runs on a STM32 F3 discovery board and includes 4 different programs.

push\_button and timer use just the discovery board.
two\_button uses the discovery board with a button hooked up to pin PC1 and a buzzer hooked up to PC3.

### push\_button  
This simple program test's the implementation of the INPUT PIN trait in the stm32f30x-hal crate.  Each press of the user button changes the state of the next led in a clockwise manner starting with the North led. We tested this programm with multiple loops through the code.  We also tested for button bounce and found the button on our board had no need for debounce correction.

### timer
The timer program is very similar to the roulette project in the discover tutorial.  It turns on an led for 3.5 seconds.  It turns another led each second.  This code is designed to use our timer module code rather than the simple delay function provided by the stm3f30x-hal crate.  This timer code allows things to be effectively running concurrently.

### two\_button
The two\button program tests the button module that implements more varied button input functionallity.  There was thought put in to correct for bounce though both of the buttons we used had no problems with bounce and this it is turned off.  The user button has the same functionallity as in the push\_button program.  The additional knob button will sound the buzzer as long as it is pressed.  If one button is pressed down the other button wont work.

## Development Environment

To build these projects you need 
* cross compilation support for the ARM Cortex-M
* OpenOCD
* a gdb for the cortexm chip. For our development we used gdb-multiarch


Please follow the instructions in the [embedded rust book](https://rust-embedded.github.io/book/intro/install.html) or the [discovery tutorial](https://rust-embedded.github.io/discovery/03-setup/index.html)

## License

This program is licensed under the "MIT License".  Please
see the file `LICENSE` in the source distribution of this
software for license terms.

[team]: https://github.com/rust-embedded/wg#the-cortex-m-team

