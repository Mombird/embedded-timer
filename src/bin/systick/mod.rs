// Copyright © 2019 Robin Gearn
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.



// use cast::u32;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use stm32f30x_hal::rcc::Clocks;

// this code is modified from
// https://docs.rs/stm32f30x-hal/0.2.0/src/stm32f30x_hal/delay.rs.html#11-14
/// System timer (SysTick) as a delay provider
pub struct Systick {
    // Keeping clocks for now.  future plans are to use Clocks.sysclk
    // to get the clock hertz.  Keeping it means that is a similar interface
    // as hal::delay
    #[allow(dead_code)]
    clocks: Clocks,
    syst: SYST,
}

impl Systick {
    /// Configures the system timer (SysTick) as a tick provider
    pub fn new(mut syst: SYST, clocks: Clocks, value: u32) -> Option<Self> {
        syst.set_clock_source(SystClkSource::Core);

        let value = 8000 * value -1;  // change ms to clockticks

        // check to see if in range 
        if value >= 1 && value <=0x00ffffff {
            // set countdown ticks, zero current time, start the timer
            syst.set_reload(value);
            syst.clear_current();
            syst.enable_counter();
            Some (Systick { syst, clocks })
        } else {
            None
        }

    }


    pub fn wait_til_wrapped(&mut self) {
        while !SYST::has_wrapped(&mut self.syst) {};
    }

}

