use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::time::Duration;

use anyhow::{anyhow, Result};
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::rmt::RmtChannel;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time;
use tokio::time::{timeout_at, Instant};

use super::rgb::Rgb;
use super::strip::LedStrip;

pub trait DisplayOnLeds {
    async fn display(&self, tx: Sender<LedChange>) -> Result<()>;
}

struct XYZGrid<T: Clone>(Vec<Vec<Vec<T>>>);

impl<T: Clone> XYZGrid<T> {
    fn z_height(&self) -> usize {
        self.0.len()
    }
    fn get(&self, x: usize, y: usize, z: usize) -> T {
        self.0[z][x][y].clone()
    }

    fn set(&mut self, x: usize, y: usize, z: usize, value: T) {
        self.0[z][x][y] = value;
    }

    pub fn new(z: usize, x: usize, y: usize, val: T) -> Self {
        Self(vec![vec![vec![val; y]; x]; z])
    }
}
impl XYZGrid<Rgb> {
    fn get_visible(&self, x: usize, y: usize) -> Option<Rgb> {
        // starts as self.z_height - 1
        for z in (0..self.z_height()).rev() {
            let val = self.get(x, y, z);
            // if a higher layer is not off return that highest layer
            if !val.is_off() {
                return Some(val);
            }
        }
        None
    }
}

pub struct LedOverlay<const X_SIZE: usize, const Y_SIZE: usize, const DEPTH: usize> {
    levels: XYZGrid<Rgb>,
}

impl<const X_SIZE: usize, const Y_SIZE: usize, const DEPTH: usize>
    LedOverlay<X_SIZE, Y_SIZE, DEPTH>
{
    pub fn update(&mut self, level_idx: u8, change: LedChange) -> Result<Option<LedChange>> {
        if (change.x as usize) >= X_SIZE {
            return Err(anyhow!(
                "tried to change led in overlay that is {} >= {X_SIZE}",
                change.x
            ));
        } else if (change.y as usize) >= Y_SIZE {
            return Err(anyhow!(
                "tried to change led in overlay that is {} >= {Y_SIZE}",
                change.y
            ));
        } else if (level_idx as usize) >= DEPTH {
            return Err(anyhow!(
                "tried to change led in overlay depth that is {level_idx} >= {DEPTH}"
            ));
        }
        // set the value on the level
        self.levels.set(
            change.x as usize,
            change.y as usize,
            level_idx as usize,
            change.color,
        );
        // get the visible value
        Ok(self
            .levels
            .get_visible(change.x as usize, change.y as usize)
            .map(|color| LedChange { color, ..change }))
    }

    pub fn new() -> Self {
        Self {
            levels: XYZGrid::new(DEPTH, X_SIZE, Y_SIZE, Rgb::new(0, 0, 0)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct LedChange {
    pub x: u8,
    pub y: u8,
    pub color: Rgb,
}

impl Display for LedChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let LedChange { x, y, color } = self;
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
) -> Result<()> {
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
    let start = Instant::now();
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
