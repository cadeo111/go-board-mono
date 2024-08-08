use std::time::Duration;

use anyhow::{anyhow, Result};
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::rmt::{FixedLengthSignal, PinState, Pulse, RmtChannel, TxRmtDriver, VariableLengthSignal};
use esp_idf_svc::hal::rmt::config::TransmitConfig;
use esp_idf_svc::hal::units::Hertz;
use crate::neopixel::rgb;
use crate::neopixel::rgb::Rgb;

pub fn neopixel(rgb: Rgb, tx: &mut TxRmtDriver) -> Result<()> {
    let color: u32 = rgb.into();
    let ticks_hz = tx.counter_clock()?;
    let (t0h, t0l, t1h, t1l) = (
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
    );
    let mut signal = FixedLengthSignal::<24>::new();
    for i in (0..24).rev() {
        let p = 2_u32.pow(i);
        let bit: bool = (p & color) != 0;
        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
        signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }


    let mut signal2 = FixedLengthSignal::<24>::new();
    let rgb2 =Rgb::from_hsv(50, 50, 20)?;
    for i in (0..24).rev() {
        let color: u32 = rgb2.into();
        let p = 2_u32.pow(i);
        let bit: bool = (p & color) != 0;
        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
        signal2.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }
    tx.start_blocking(&signal)?;
    tx.start_blocking(&signal)?;
    tx.start_blocking(&signal2)?;
    tx.start_blocking(&signal)?;
    Ok(())
}


pub fn neopixel2(rgb: Rgb, tx: &mut TxRmtDriver, num_leds: u32) -> Result<()> {
    let color: u32 = rgb.into();
    let ticks_hz = tx.counter_clock()?;
    let (t0h, t0l, t1h, t1l) = (
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
    );

    
    let mut signal = VariableLengthSignal::new();
    // // convert to pulses;
    // for n in 0..(24 * num_leds) {
    //     let i = n % 24;
    //     let p = 2_u32.pow(i);
    //     let bit: bool = (p & color) != 0; // mask of each bit progressively
    //     let [high_pulse, low_pulse] = if bit { [&t1l,&t1h ] } else { [&t0l,&t0h, ] };
    //     signal.push([low_pulse,high_pulse ])?;
    // }
    let zero = Pulse::zero();
    for _ in 0..num_leds {
        let mut s:[[&Pulse;2];24] = [[&zero,&zero];24];
        for i in (0..24).rev() {
            let p = 2_u32.pow(i);
            let bit: bool = (p & color) != 0;
            s[(23-i) as usize] = if bit { [&t1h, &t1l] } else { [&t0h, &t0l] };
        }
        for item in s {
            signal.push(item)?;
        }
    }
    
    tx.start_blocking(&signal)?;
    Ok(())
}


pub fn neopixe3(rgb: Rgb, tx: &mut TxRmtDriver) -> Result<()> {
    let color: u32 = rgb.into();
    let ticks_hz = tx.counter_clock()?;
    let (t0h, t0l, t1h, t1l) = (
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
    );
    let mut signal = FixedLengthSignal::<24>::new();
    for i in (0..24).rev() {
        let p = 2_u32.pow(i);
        let bit: bool = (p & color) != 0;
        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
        signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }


    let mut signal2 = FixedLengthSignal::<24>::new();
    let rgb2 =Rgb::from_hsv(50, 50, 20)?;
    for i in (0..24).rev() {
        let color: u32 = rgb2.into();
        let p = 2_u32.pow(i);
        let bit: bool = (p & color) != 0;
        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
        signal2.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }
    tx.start_blocking(&signal)?;
    tx.start_blocking(&signal)?;
    tx.start_blocking(&signal2)?;
    tx.start_blocking(&signal)?;
    Ok(())
}

pub struct LedStrip<'tx, const SIZE: usize> {
    config: TransmitConfig,
    tx: TxRmtDriver<'tx>,
    data: [Rgb; SIZE],
}

impl<'tx, const SIZE: usize> LedStrip<'tx, SIZE> {
    pub fn new(led_pin: impl Peripheral<P=impl OutputPin> + 'tx, channel: impl Peripheral<P: RmtChannel> + 'tx) -> Result<Self> {
        let config: TransmitConfig = TransmitConfig::new().clock_divider(1);
        let tx: TxRmtDriver = TxRmtDriver::new(channel, led_pin, &config)?;
        Ok(LedStrip { config, tx, data: [Rgb::new(0, 0, 0); SIZE] })
    }

    pub fn clear(&mut self) {
        for i in 0..SIZE{
            self.data[i] = Rgb::new(0, 0, 0);
        }
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
        let mut s:[[&Pulse;2];24] = [[&zero,&zero];24];
        let mut signal = VariableLengthSignal::new();

        for rgb in self.data {
            let color: u32 = rgb.into();
            for i in (0..24).rev() {
                let p = 2_u32.pow(i);
                let bit: bool = (p & color) != 0;
                s[(23-i) as usize] = if bit { [&t1h, &t1l] } else { [&t0h, &t0l] };
            }
            for item in s {
                signal.push(item)?;
            }
        }
        self.tx.start_blocking(&signal)?;
        Ok(())
    }
}

fn generate_pulse_id_iter_for_color(rgb: Rgb, ticks_hz: Hertz) -> Result<FixedLengthSignal::<24>> {
    let color: u32 = rgb.into();
    let mut signal = FixedLengthSignal::<24>::new();
    for i in (0..24).rev() {
        let p = 2_u32.pow(i);
        let bit: bool = (p & color) != 0;
        let (high_pulse, low_pulse) = if bit { (PulseId::HighOneBit.pulse(ticks_hz)?, PulseId::LowOneBit.pulse(ticks_hz)?) } else { (PulseId::HighZeroBit.pulse(ticks_hz)?, PulseId::LowZeroBit.pulse(ticks_hz)?) };
        signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }
    Ok(signal)
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum PulseId {
    HighZeroBit,
    LowZeroBit,
    HighOneBit,
    LowOneBit,
}

impl PulseId {
    fn pulse(self: &Self, ticks_hz: Hertz) -> Result<Pulse> {
        Ok(match self {
            PulseId::HighZeroBit => {
                Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?
            }
            PulseId::LowZeroBit => {
                Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?
            }
            PulseId::HighOneBit => {
                Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?
            }
            PulseId::LowOneBit => {
                Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?
            }
        })
    }
}






