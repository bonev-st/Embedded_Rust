//! STM32L562E-DK RTT / defmt demo.
//!
//! - Red LED LD9 (PD3) blinks as a heartbeat (logged at `trace` level).
//! - Green LED LD10 (PG12) is on while the USER button B2 (PC13) is pressed.
//! - Every button press/release is logged; a status struct is logged every second.
//! - Holding the button for 3 s triggers a deliberate panic to show panic-probe output.
//!
//! All output goes over RTT (defmt); read it with `cargo run`, `cargo embed`
//! or the probe-rs VS Code debugger.

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use defmt_rtt as _;
use panic_probe as _;
use stm32l5::stm32l562 as pac;

/// Core clock after reset: MSI at 4 MHz.
const CORE_CLOCK_HZ: u32 = 4_000_000;
/// Holding the button this long triggers the demo panic.
const PANIC_HOLD_MS: u32 = 3_000;

/// Milliseconds since boot, incremented by the SysTick exception.
static TICKS_MS: AtomicU32 = AtomicU32::new(0);

// Every defmt log line is prefixed with this timestamp.
defmt::timestamp!("{=u32:ms}", TICKS_MS.load(Ordering::Relaxed));

#[derive(defmt::Format, Clone, Copy, PartialEq)]
enum Button {
    Released,
    Pressed,
}

#[derive(defmt::Format)]
struct Status {
    uptime_ms: u32,
    presses: u32,
    button: Button,
    heartbeat_on: bool,
}

#[entry]
fn main() -> ! {
    let dp = defmt::unwrap!(pac::Peripherals::take());
    let mut cp = defmt::unwrap!(cortex_m::Peripherals::take());

    defmt::println!("==== STM32L562E-DK RTT demo ====");
    defmt::info!("core clock {=u32} Hz (MSI reset default)", CORE_CLOCK_HZ);

    // 1 ms SysTick time base
    cp.SYST.set_clock_source(SystClkSource::Core);
    cp.SYST.set_reload(CORE_CLOCK_HZ / 1_000 - 1);
    cp.SYST.clear_current();
    cp.SYST.enable_interrupt();
    cp.SYST.enable_counter();

    // PG[15:2] are supplied from VDDIO2, which must be marked valid (PWR_CR2.IOSV)
    // before GPIOG works. That needs the PWR clock first.
    dp.RCC.apb1enr1().modify(|_, w| w.pwren().set_bit());
    let _ = dp.RCC.apb1enr1().read(); // short delay after enabling a peripheral clock
    dp.PWR.cr2().modify(|_, w| w.iosv().set_bit());

    // GPIOC: button, GPIOD: red LED, GPIOG: green LED
    dp.RCC
        .ahb2enr()
        .modify(|_, w| w.gpiocen().set_bit().gpioden().set_bit().gpiogen().set_bit());
    let _ = dp.RCC.ahb2enr().read();

    // LEDs are active low: drive the pins high (LED off) before making them outputs
    dp.GPIOD.bsrr().write(|w| w.bs3().set_bit());
    dp.GPIOG.bsrr().write(|w| w.bs12().set_bit());
    dp.GPIOD.moder().modify(|_, w| w.moder3().output());
    dp.GPIOG.moder().modify(|_, w| w.moder12().output());

    // STM32L5 GPIOs reset to analog mode: make PC13 a digital input (button is active high)
    dp.GPIOC.pupdr().modify(|_, w| w.pupdr13().pull_down());
    dp.GPIOC.moder().modify(|_, w| w.moder13().input());

    defmt::info!("GPIO ready: LD9=PD3 (heartbeat), LD10=PG12 (button), B2=PC13");
    defmt::info!("press B2 to log events, hold it {=u32} ms for a demo panic", PANIC_HOLD_MS);

    let mut presses: u32 = 0;
    let mut press_history = [0u32; 4]; // timestamps of the last presses
    let mut last_button = Button::Released;
    let mut pressed_since: u32 = 0;
    let mut heartbeat_on = false;

    let mut next_poll: u32 = 0;
    let mut next_blink: u32 = 0;
    let mut next_status: u32 = 1_000;

    loop {
        let now = TICKS_MS.load(Ordering::Relaxed);

        // Button: sample every 10 ms (simple debounce)
        if now >= next_poll {
            next_poll = now + 10;

            let button = if dp.GPIOC.idr().read().idr13().bit_is_set() {
                Button::Pressed
            } else {
                Button::Released
            };

            if button != last_button {
                match button {
                    Button::Pressed => {
                        presses += 1;
                        pressed_since = now;
                        press_history.rotate_left(1);
                        press_history[press_history.len() - 1] = now;
                        dp.GPIOG.bsrr().write(|w| w.br12().set_bit()); // green on

                        defmt::info!("button pressed (#{=u32})", presses);
                        defmt::debug!("last presses at {} ms", press_history);
                        if presses % 5 == 0 {
                            defmt::warn!("{=u32} presses - that's a lot of clicking", presses);
                        }
                    }
                    Button::Released => {
                        dp.GPIOG.bsrr().write(|w| w.bs12().set_bit()); // green off
                        defmt::debug!("button released after {=u32} ms", now - pressed_since);
                    }
                }
                last_button = button;
            } else if button == Button::Pressed && now - pressed_since >= PANIC_HOLD_MS {
                defmt::error!("button held for {=u32} ms -> demo panic", PANIC_HOLD_MS);
                panic!("demo panic after {} presses", presses);
            }
        }

        // Heartbeat: toggle the red LED every 500 ms
        if now >= next_blink {
            next_blink = now + 500;
            heartbeat_on = !heartbeat_on;
            if heartbeat_on {
                dp.GPIOD.bsrr().write(|w| w.br3().set_bit()); // red on
            } else {
                dp.GPIOD.bsrr().write(|w| w.bs3().set_bit()); // red off
            }
            defmt::trace!("heartbeat {=bool}", heartbeat_on);
        }

        // Status: once per second
        if now >= next_status {
            next_status = now + 1_000;
            let status = Status {
                uptime_ms: now,
                presses,
                button: last_button,
                heartbeat_on,
            };
            defmt::info!("{}", status);

            let odr = dp.GPIOD.odr().read().bits();
            defmt::debug!("GPIOD ODR = {=u32:#010x}, PD3 high = {=bool}", odr, odr & (1 << 3) != 0);
        }
    }
}

#[exception]
fn SysTick() {
    TICKS_MS.fetch_add(1, Ordering::Relaxed);
}
