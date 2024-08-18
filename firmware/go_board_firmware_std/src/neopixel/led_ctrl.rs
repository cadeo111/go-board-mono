use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::time::Duration;

use anyhow::anyhow;
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::rmt::RmtChannel;
use tokio::sync::mpsc::Receiver;
use tokio::time;
use tokio::time::{timeout_at, Instant};

use super::rgb::Rgb;
use super::strip::LedStrip;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct LedChange {
    pub x: u8,
    pub y: u8,
    pub color: Rgb,
}

impl Display for LedChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let LedChange{x,y, color} = self;
        write!(f, "({x},{y},{color})")
    }
}

impl LedChange {
    pub fn new(x: u8, y: u8, color: Rgb) -> Self {
        Self { x, y, color }
    }
}

pub async fn led_ctrl<const LED_STRIP_SIZE: usize, const LED_STRIP_SQUARE_SIDE: usize>(
    led_pin: impl Peripheral<P = impl OutputPin>,
    channel: impl Peripheral<P: RmtChannel>,
    mut rx: Receiver<LedChange>,
) -> anyhow::Result<()> {
    // let led = peripherals.pins.gpio2;
    // let channel = peripherals.rmt.channel0;

    let mut strip: LedStrip<LED_STRIP_SIZE> = LedStrip::new(led_pin, channel)?;
    // let config = TransmitConfig::new().clock_divider(1);
    // let mut tx = TxRmtDriver::new(channel, led_pin, &config)?;
    strip.clear();
    strip.refresh()?;
    time::sleep(Duration::from_millis(100)).await;

    // 3 seconds white at 10% brightness
    strip.set_led(26, Rgb::new(25, 25, 25))?;
    strip.refresh()?;
    time::sleep(Duration::from_secs(10)).await;
    // have a loop where we receive updates
    // if 1 second has elapse cancel waiting and
    let start  =Instant::now();
    let mut stop_at = Instant::now().add(Duration::from_secs(1));
    let mut dirty = false;
    loop {
        if let Ok(change_opt) = timeout_at(stop_at, rx.recv()).await {
            match change_opt {
                None => return Err(anyhow!("Led Channel closed unexpectedly!")),
                Some(change) => {
                    println!("{:?} ---> Got Change! {change}", start.elapsed());
                    strip.set_led_change(&change, LED_STRIP_SQUARE_SIDE)?;
                    if !dirty {
                        dirty = true;
                    }
                }
            }
        } else {
            if dirty {
                println!("{:?} ---> Refreshing!", start.elapsed());
                dirty = false;
                // reset timer, aim for every 1 seconds
                strip.refresh()?
            }
            stop_at = Instant::now().add(Duration::from_secs(1));
        }
    }
}
