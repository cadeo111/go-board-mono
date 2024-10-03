use anyhow::Result;
use esp_idf_svc::hal::gpio::{AnyIOPin, AnyInputPin, Input, InterruptType, Level, PinDriver, Pull};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::Notify;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum SpinDirection {
    CounterClockwise,
    Clockwise,
}
pub type EncoderInfo = (i32, SpinDirection);

impl Display for SpinDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SpinDirection::CounterClockwise => f.write_str("(SD:Counter Clockwise)"),
            SpinDirection::Clockwise => f.write_str("(SD:Clockwise)"),
        }
    }
}

pub struct RotaryEncoderState<'a> {
    clk: PinDriver<'a, AnyInputPin, Input>,
    clk_notify: Arc<Notify>,
    dt: PinDriver<'a, AnyInputPin, Input>,
    button_notify: Arc<Notify>,
    button: PinDriver<'a, AnyIOPin, Input>,
}

impl<'a> RotaryEncoderState<'a> {
    pub fn init(
        rotary_encoder_btn_pin: AnyIOPin,
        rotary_encoder_clk_pin: AnyInputPin,
        rotary_encoder_dt_pin: AnyInputPin,
    ) -> Result<Self> {
        let (button_notify, button) = {
            let mut button = PinDriver::input(rotary_encoder_btn_pin)?;
            button.set_pull(Pull::Up)?;
            button.set_interrupt_type(InterruptType::PosEdge)?;
            let notify = Arc::new(Notify::new());
            let notifier = notify.clone();
            /// Make sure to call  `button.enable_interrupt()?;` before waiting for notification
            unsafe {
                button.subscribe(move || {
                    notifier.notify_one();
                })?;
            }
            (notify, button)
        };

        let (clk, clk_notify, dt) = {
            let mut clk = PinDriver::input(rotary_encoder_clk_pin)?;
            clk.set_interrupt_type(InterruptType::PosEdge)?;

            let dt = PinDriver::input(rotary_encoder_dt_pin)?;

            let clk_notify = Arc::new(Notify::new());
            let notifier = clk_notify.clone();
            unsafe {
                clk.subscribe(move || {
                    notifier.notify_one();
                })?;
            }
            (clk, clk_notify, dt)
        };

        Ok(Self {
            clk,
            clk_notify,
            dt,
            button_notify,
            button,
        })
    }

    pub fn is_button_pressed(&self) -> bool {
        // button is active low i'm pretty sure
        self.button.get_level() == Level::Low
    }

    pub async fn monitor_encoder_spin(&mut self, on_change: Sender<EncoderInfo>) -> Result<()> {
        let mut current_direction;
        let mut counter = 0;

        loop {
            self.clk.enable_interrupt()?;
            self.clk_notify.notified().await;
            if self.clk.get_level() != self.dt.get_level() {
                current_direction = SpinDirection::Clockwise;
                counter += 1;
            } else {
                current_direction = SpinDirection::CounterClockwise;
                counter -= 1;
            }
            println!("counter: {}, direction: {}", counter, current_direction);
            on_change.send((counter, current_direction)).await?;
        }
        Ok(())
    }
}
