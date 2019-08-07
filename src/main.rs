// Copyright © 2019 Robin Gearn
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.


#![no_std]
#![no_main]

#[allow(unused_extern_crates)]

// pick a panicking behavior
// extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use f3::hal::delay::Delay;
use f3::hal::prelude::*;
//use f3::hal::stm32f30x;
use f3::hal::stm32f30x::rcc;
use f3::hal::stm32f30x::{self, GPIOA, RCC};

//use f3::hal::gpio::gpioa::{self,PA0};
//use f3::hal::gpio::{Input, Floating};
//use stm32f30x;
use f3::led::Leds;
use cortex_m_rt::ExceptionFrame;


use cortex_m_rt::entry;
use cortex_m_rt::exception;

#[entry]
fn main() -> ! {
    // get processor and discovery board peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // set up system timer
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let delay = Delay::new(cp.SYST, clocks);

    // enable (power on) button
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let ptr = unsafe {&*GPIOA::ptr() };
    //rcc.ahbenr.modify(|_, w| w.iopaen().set_bit());
    //rcc.ahb.enr().modify(|_, w| w.iopaen().enabled());


    // set button as input, floating
    // gpioa.moder.modify(|_,w| w.moder0().input());
    // gpioa.pupdr.write(|w| w.pupdr0().bits(0));
        let pa0 = gpioa.pa0.into_floating_input
            (&mut gpioa.moder, &mut gpioa.pupdr);

    let mut leds = Leds::new(dp.GPIOE.split(&mut rcc.ahb));

    


    loop {
        // turn on a light with each press
        for led in leds.iter_mut() {
            wait_till_pressed(ptr);
            led.on();
        }

        // turn off a light with each press
        for led in leds.iter_mut() {
            wait_till_pressed(ptr);
            led.off();
        }
    }
}



// returns true if button pressed 
// if button is pressed it waits until the button is released
    pub fn wait_till_pressed(gpioa: &'static stm32f30x::gpioa::RegisterBlock) {
        
        // loop till you get a press
        while !gpioa.idr.read().idr0().bit_is_set() {};
        
        // now loop till not pressed
        while gpioa.idr.read().idr0().bit_is_set() {};
        
    }




#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}


#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

/*
 * pub struct Button {
    pa0 = PA0<Input<Floating>>,
}


impl Button {
    pub fn new(mut gpioa: gpioa::Parts) -> Self {
        let pa0 = gpioa.pa0.into_floating_input
            (&mut gpioa.moder, &mut gpioa.pupdr)
    }
*/
