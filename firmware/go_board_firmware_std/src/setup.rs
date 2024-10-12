use crate::encoder::{EncoderInfo, RotaryEncoderState};
use crate::neopixel::led_ctrl::LedChange;
use crate::restart_recovery::{get_and_clear_recover_option, RecoverOption};
use crate::storage::SaveInNvs;
use crate::wifi::WifiCredentials;
use crate::{settings, CHANNEL_SIZE};
use anyhow::{anyhow, Result};
use esp_idf_svc::eventloop::{EspEventLoop, EspSystemEventLoop, System};
use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::rmt::RmtChannel;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvsPartition, NvsDefault};
use esp_idf_svc::sys;
use esp_idf_svc::sys::esp;
use esp_idf_svc::timer::{EspTaskTimerService, EspTimerService, Task};
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use log::{error, info};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

fn setup_basic_esp_stuff() -> Result<(
    Peripherals,
    EspEventLoop<System>,
    EspTimerService<Task>,
    EspNvsPartition<NvsDefault>,
)> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // eventfd is needed by our mio poll implementation.  Note you should set max_fds
    // higher if you have other code that may need eventfd.
    info!("Setting up eventfd...");
    let config = sys::esp_vfs_eventfd_config_t {
        max_fds: 1,
        ..Default::default()
    };

    {
        esp! { unsafe { sys::esp_vfs_eventfd_register(&config) } }
    }?;

    info!("Setting up board...");

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let timer = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    Ok((peripherals, sysloop, timer, nvs))
}

pub fn handle_switch_to_settings(
    rotary_encoder_state: &RotaryEncoderState,
    modem: Modem,
    nvs: EspNvsPartition<NvsDefault>,
    sysloop: EspEventLoop<System>,
    wifi_credentials: &WifiCredentials,
) -> Result<(Modem, RecoverOption)> {
    // TODO: figure out way to force settings menu on restart (via nvs?)
    let recovery_option = get_and_clear_recover_option(nvs.clone())?;
    info!("got recovery option: {recovery_option:?}");

    let should_open_settings_panel = recovery_option == RecoverOption::ForceSettingsPanel || {
        info!("will check if should boot into settings mode in 2 sec...");
        // outside of tokio context so use std thread sleep
        std::thread::sleep(Duration::from_secs(2));

        info!(
            "checking if should boot into settings mode... {}",
            rotary_encoder_state.is_button_pressed()
        );
        let re_button_pressed = rotary_encoder_state.is_button_pressed();
        info!(
            "checking if should boot into settings mode... {} var: {}",
            rotary_encoder_state.is_button_pressed(),
            re_button_pressed
        );
        re_button_pressed
    };

    if should_open_settings_panel {
        info!("going into settings mode");
        settings::runner::run(nvs, modem, sysloop, &wifi_credentials).map_err(|e| anyhow!(e))?;
        Err(anyhow!(
            "[ERROR] exited settings without error, this should not happen..."
        ))
    } else {
        info!("going into game-play mode");
        Ok((modem, recovery_option))
    }
}

pub fn setup<'esp_wifi, 'rotary_encoder>() -> Result<(
    (WifiCredentials, AsyncWifi<EspWifi<'esp_wifi>>),
    (
        RotaryEncoderState<'rotary_encoder>,
        broadcast::Sender<EncoderInfo>,
        broadcast::Receiver<EncoderInfo>,
    ),
    (
        mpsc::Receiver<LedChange>,
        mpsc::Sender<LedChange>,
        impl Peripheral<P = impl OutputPin>,
        impl Peripheral<P: RmtChannel>,
    ),
    EspNvsPartition<NvsDefault>,
)> {
    // ESP
    let (peripherals, sysloop, timer, nvs) = setup_basic_esp_stuff()?;

    // WIFI CREDENTIALS
    let wifi_creds = WifiCredentials::get_saved_in_nvs_with_default(
        nvs.clone(),
        WifiCredentials::get_from_env()?,
    )?;

    // GPIOS
    let board_led_grid_pin = peripherals.pins.gpio3;
    let rmt_channel0 = peripherals.rmt.channel0;

    // ROTARY ENCODER
    let rotary_encoder_state = {
        info!("Initializing rotary encoder...");
        RotaryEncoderState::init(
            // btn
            peripherals.pins.gpio4.into(),
            // clk
            peripherals.pins.gpio5.into(),
            // dt
            peripherals.pins.gpio6.into(),
        )?
    };

    // IF SHOULD LAUNCH SETTINGS PANEL
    let (modem, _recovery_option) = handle_switch_to_settings(
        &rotary_encoder_state,
        peripherals.modem,
        nvs.clone(),
        sysloop.clone(),
        &wifi_creds,
    )?;

    // MAIN RUNNER SET UP

    info!("Initializing Wi-Fi...");
    let wifi = AsyncWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs.clone()))?,
        sysloop,
        timer.clone(),
    )?;

    let (led_change_tx, led_change_rx) = mpsc::channel::<LedChange>(CHANNEL_SIZE);
    let (tx_encoder_info, rx_encoder_info) = broadcast::channel::<EncoderInfo>(100);

    Ok((
        (wifi_creds, wifi),
        (rotary_encoder_state, tx_encoder_info, rx_encoder_info),
        (
            led_change_rx,
            led_change_tx,
            board_led_grid_pin,
            rmt_channel0,
        ),
        nvs,
    ))
}
