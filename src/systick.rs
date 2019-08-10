// Copyright © 2019 Robin Gearn
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.



// use cast::u32;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use stm32f30x_hal::rcc::Clocks;
// use stm32f30x_hal::time::Hertz;

// this code is modified from
// https://docs.rs/stm32f30x-hal/0.2.0/src/stm32f30x_hal/delay.rs.html#11-14
/// System timer (SysTick) as a delay provider and clock
pub struct Systick {
    _clocks: Clocks,
    syst: SYST,
    period: u32,
    // This can go more than 580 million years without wrapping.
    currently: u64,
}

impl Systick {
    /// Configures the system timer (SysTick) as a tick provider
    ///
    /// # Arguments
    ///
    /// * `period` - The number of milliseconds that make up a timer 
    /// 'tick'.
    pub fn new(mut syst: SYST, clocks: Clocks, period: u32) -> Option<Self> {
        syst.set_clock_source(SystClkSource::Core);

        // convert ms to ticks
        let ticks = (clocks.sysclk().0 as u64 * period as u64) / 1000 - 1;  // change ms to clockticks

        // check to see if in range 
        if ticks >= 1 && ticks <=0x00ff_ffff {
            // set countdown ticks, zero current time, start the timer
            syst.set_reload(ticks as u32);
            syst.clear_current();
            syst.enable_counter();
            Some (Systick {
                syst: syst,
                _clocks: clocks,
                period: period,
                currently: 0,
            })
        } else {
            None
        }

    }

    /// Blocks until a tick has occurred since this was last called
    pub fn wait_til_wrapped(&mut self) {
        while !SYST::has_wrapped(&mut self.syst) {};
        self.currently += self.period as u64;
    }

    /// Returns time since init in ms.
    pub fn now(&self) -> u64 {
        self.currently
    }
}

/// System clock as a clock
pub struct Sysclock {
    clocks: Clocks,
    syst: SYST,
    last_read: u32,
    current: u64,
}

impl Sysclock {
    /// Configures the system timer (SysTick) as a tick provider
    pub fn new(mut syst: SYST, clocks: Clocks) -> Self {
        syst.set_clock_source(SystClkSource::Core);
        // Using reload as wrap checker
        // set countdown ticks, zero current time, start the timer
        syst.set_reload(0x00ff_ffff);
        syst.clear_current();
        syst.enable_counter();
        Sysclock {
            clocks: clocks,
            syst: syst,
            current: 0,
            last_read: SYST::get_current(),
        }
    }

    /// Gets the clock frequency in Hz
    pub fn freq(&self) -> u32 {
        self.clocks.sysclk().0
    }

    /// Get the current time since initialization.
    /// Requires mutable reference to update 'last_read_time'
    pub fn now(&mut self) -> u64 {
        self.update();
        self.current
    }

    /// Get the current time as of the last call to `self.now()`.
    pub fn last_now(&self) -> u64 { self.current }

    /// Update the current time
    fn update(&mut self) {
        // Reading clock both before and after wrap-check to catch all 
        // cases (a ROLL check b, a check ROLL b).
        let a: u32 = SYST::get_current();
        let wrapped: bool = self.syst.has_wrapped();
        let b: u32 = SYST::get_current();
        // If wrapped is true, OR our second measurement is smaller, we've 
        // wrapped. If we've wrapped, calculate accordingly.
        self.current += if wrapped || b < a {
            0x00ff_ffff - self.last_read + b
        } else {
            b - self.last_read
        } as u64;
        self.last_read = b;
    }
}

