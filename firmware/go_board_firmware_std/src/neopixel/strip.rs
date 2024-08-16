use std::time::Duration;

use super::led_ctrl::LedChange;
use super::rgb::Rgb;
use anyhow::{anyhow, Result};
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::rmt::config::TransmitConfig;
use esp_idf_svc::hal::rmt::{PinState, Pulse, RmtChannel, TxRmtDriver, VariableLengthSignal};

pub struct LedStrip<'tx, const SIZE: usize> {
    config: TransmitConfig,
    tx: TxRmtDriver<'tx>,
    data: [Rgb; SIZE],
}

impl<'tx, const SIZE: usize> LedStrip<'tx, SIZE> {
    pub fn new(
        led_pin: impl Peripheral<P = impl OutputPin> + 'tx,
        channel: impl Peripheral<P: RmtChannel> + 'tx,
    ) -> Result<Self> {
        let config: TransmitConfig = TransmitConfig::new().clock_divider(1);
        let tx: TxRmtDriver = TxRmtDriver::new(channel, led_pin, &config)?;
        Ok(LedStrip {
            config,
            tx,
            data: [Rgb::new(0, 0, 0); SIZE],
        })
    }

    pub fn clear(&mut self) {
        for i in 0..SIZE {
            self.data[i] = Rgb::new(0, 0, 0);
        }
    }
    pub fn set_led_change(&mut self, change: &LedChange, size: usize) -> Result<()> {
        let LedChange { x, y, color } = *change;
        let x: usize = x.into();
        let y: usize = y.into();
        let index = if x % 2 == 0 {
            x * size + y
        } else {
            (x + 1) * size - y - 1
        };
        self.set_led(index, color)
    }

    pub fn set_led(&mut self, index: usize, rgb: Rgb) -> Result<()> {
        if index >= SIZE {
            return Err(anyhow!("index out of range of led strip!"));
        }
        self.data[index] = rgb;

        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        let ticks_hz = self.tx.counter_clock()?;

        let (t0h, t0l, t1h, t1l) = (
            Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
            Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
            Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
            Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
        );

        let zero = Pulse::zero();
        let mut s: [[&Pulse; 2]; 24] = [[&zero, &zero]; 24];
        let mut signal = VariableLengthSignal::new();

        for rgb in self.data {
            let color: u32 = rgb.into();
            for i in (0..24).rev() {
                let p = 2_u32.pow(i);
                let bit: bool = (p & color) != 0;
                s[(23 - i) as usize] = if bit { [&t1h, &t1l] } else { [&t0h, &t0l] };
            }
            for item in s {
                signal.push(item)?;
            }
        }
        self.tx.start_blocking(&signal)?;
        Ok(())
    }
}
