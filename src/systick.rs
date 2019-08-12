// Copyright © 2019 Robin Gearn, James Wescott
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use super::Milliseconds;

// use cast::u32;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use stm32f30x_hal::rcc::Clocks;
// use stm32f30x_hal::time::Hertz;

// this code is modified from
// https://docs.rs/stm32f30x-hal/0.2.0/src/stm32f30x_hal/delay.rs.html#11-14
/// System timer (SysTick) as a delay provider and clock
///
/// # Examples
///
/// ```no_run
/// use timer::systick::Systick;
///
/// use f3::hal::prelude::*;
/// use f3::hal::stm32f30x;
/// # use cortex_m_rt::entry;
///
/// # #[entry]
/// fn main() -> ! {
///     // get processor and discovery board peripherals
///     let cp = cortex_m::Peripherals::take().unwrap();
///     let peripherals = stm32f30x::Peripherals::take().unwrap();
///
///     let mut rcc = dp.RCC.constrain();
///
///     // set up system timer using default settings of 8 MHz
///     let hal_clocks = rcc.cfgr.freeze(&mut flash.acr);
///     // set up systick clock with period of 6ms
///     let mut systick = Systick::new(cp.SYST, hal_clocks, 6).unwrap();
///
///     loop {
///         iprintln!(
///             "{}ms have passed since timer initialization",
///             systick.now()
///             );
///
///         systick.wait_til_wrapped();
///     }
/// }
/// ```
pub struct Systick {
    /// Contains clock frequencies
    _clocks: Clocks,
    /// The system timer.
    syst: SYST,
    /// The length of a single tick, in ms. Can range from 1 to 0x00ff_ffff
    period: u32,
    /// The current time since initialization in ms, accurate to `period`
    /// ms. Note: Only accurate if self.wait_til_wrapped is called
    /// frequently enough!
    currently: Milliseconds,
}

impl Systick {
    /// Configures the system timer (SysTick) as a tick provider
    ///
    /// # Arguments
    ///
    /// * `period` - The number of milliseconds that make up a timer
    /// 'tick'.
    pub fn new(mut syst: SYST, clocks: Clocks, period: Milliseconds) -> Option<Self> {
        syst.set_clock_source(SystClkSource::Core);

        // convert ms to ticks
        let ticks = (clocks.sysclk().0 as u64 * period as u64) / 1000 - 1; // change ms to clockticks

        // check to see if in range
        if ticks >= 1 && ticks <= 0x00ff_ffff {
            // set countdown ticks, zero current time, start the timer
            syst.set_reload(ticks as u32);
            syst.clear_current();
            syst.enable_counter();
            Some(Systick {
                syst: syst,
                _clocks: clocks,
                period: period,
                currently: 0,
            })
        } else {
            None
        }
    }

    /// Blocks until a tick has occurred since this was last called.
    /// Updates the current time.
    pub fn wait_til_wrapped(&mut self) {
        while !SYST::has_wrapped(&mut self.syst) {}
        self.currently += self.period as Milliseconds;
    }

    /// Returns time since init in ms. Doesn't update until
    /// self.wait_til_wrapped() is called.
    pub fn now(&self) -> Milliseconds {
        self.currently
    }
}
