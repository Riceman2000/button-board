#![no_std]
#![no_main]

// debug
use defmt::info;
use defmt_rtt as _;
use embassy_rp::gpio::{Input, Pull};
use panic_probe as _;

// Embassy interfaces
use embassy_executor::Spawner;
use embassy_rp::config::Config;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::watchdog::Watchdog;
use embassy_time::{Duration, Timer};

const WATCHDOG_DURATION: u64 = 8;
const BEEP_DURATION: u64 = 300;
const DEBOUNCE_MS: u64 = 750;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Pac and watchdog init
    let p = embassy_rp::init(Config::default());
    let mut watchdog = Watchdog::new(p.WATCHDOG);
    watchdog.start(Duration::from_secs(WATCHDOG_DURATION));
    let mut led = Output::new(p.PIN_25, Level::Low);
    blink_led(&mut led, 10, 100).await;

    spawner
        .spawn(watchdog_task(led, watchdog))
        .expect("Failed to start watchdog task");

    let butt1 = Input::new(p.PIN_20, Pull::Down);
    let butt2 = Input::new(p.PIN_19, Pull::Down);
    let butt3 = Input::new(p.PIN_18, Pull::Down);
    let get_state = || (butt1.is_high(), butt2.is_high(), butt3.is_high());

    let mut beeper1 = Output::new(p.PIN_14, Level::Low);
    let mut beeper2 = Output::new(p.PIN_15, Level::Low);

    let mut last_state = get_state();
    info!("Entering main loop CORE0");
    loop {
        let state = get_state();
        info!("Button states: {:?}", state);
        if state != last_state {
            info!("State change, beeping then debouncing");
            beep(&mut beeper1, &mut beeper2).await;
            Timer::after_millis(DEBOUNCE_MS).await;
            last_state = get_state();
        }
        Timer::after_millis(50).await;
    }
}

async fn beep(beeper1: &mut Output<'_>, beeper2: &mut Output<'_>) {
    beeper1.set_high();
    beeper2.set_high();
    Timer::after_millis(BEEP_DURATION / 2).await;
    beeper1.set_low();
    beeper2.set_low();
}

#[embassy_executor::task]
async fn watchdog_task(mut led: Output<'static>, mut watchdog: Watchdog) -> ! {
    loop {
        watchdog.feed();
        blink_led(&mut led, 4, 250).await;
    }
}

/// Note total delay will be `period` * `num_blinks`
async fn blink_led(led: &mut Output<'_>, num_blinks: usize, period: u64) {
    for _ in 0..num_blinks {
        led.set_high();
        Timer::after_millis(period / 2).await;
        led.set_low();
        Timer::after_millis(period / 2).await;
    }
}
