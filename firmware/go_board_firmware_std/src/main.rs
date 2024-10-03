use std::iter::{Enumerate, FlatMap, Map};
use std::num::NonZeroU32;
use std::slice::Iter;
use std::str;
use std::sync::Arc;

use crate::encoder::{EncoderInfo, RotaryEncoderState};
use crate::neopixel::led_ctrl::{led_ctrl, DisplayOnLeds, LedChange, LedOverlay};
use crate::neopixel::rgb::Rgb;
use crate::onlinego::api;
use crate::onlinego::api::{
    test_connection, BoardColor, BoardState, GameListData, OauthResponseValid, OnlineGoLoginInfo,
    Player,
};
use crate::onlinego::auth_token::AuthToken;
use crate::storage::SaveInNvs;
use crate::wifi::{WifiCredentials, WifiLoop};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
pub use embedded_svc::http::client::Client;
pub use embedded_svc::http::Method;
pub use embedded_svc::io::Read;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::gpio::InterruptType;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys;
use esp_idf_svc::sys::{esp, esp_app_desc};
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver as BrReceiver, Sender as BrSender};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{broadcast, mpsc};
use tokio::time::sleep;
use tokio::time::Duration;
use tokio::{join, select};

mod encoder;
mod neopixel;
mod onlinego;
mod settings;
mod storage;
mod wifi;

const BOARD_SIZE: usize = 16;
const CHANNEL_SIZE: usize = BOARD_SIZE * 2;

// To test, run `cargo run`, then when the settings is up, use `nc -v espressif 12345` from
// a machine on the same Wi-Fi network.
const TCP_LISTENING_PORT: u16 = 12345;

//
// esp_app_desc!();

fn main() -> Result<()> {
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

    let wifi_creds = WifiCredentials::get_saved_in_nvs_with_default(
        nvs.clone(),
        WifiCredentials::get_from_env()?,
    )?;

    // GPIOS
    let board_leds = peripherals.pins.gpio3;
    let channel0 = peripherals.rmt.channel0;
    let rotary_encoder_btn_pin = peripherals.pins.gpio4;
    let rotary_encoder_clk_pin = peripherals.pins.gpio5;
    let rotary_encoder_dt_pin = peripherals.pins.gpio6;

    info!("Initializing rotary encoder...");

    let rotary_encoder = RotaryEncoderState::init(
        rotary_encoder_btn_pin.into(),
        rotary_encoder_clk_pin.into(),
        rotary_encoder_dt_pin.into(),
    )?;

    info!("checking if should boot into settings mode...");
    let re_button_pressed = rotary_encoder.is_button_pressed();
    if (re_button_pressed) {
        info!("going into settings mode");
        settings::runner::run(nvs, peripherals.modem, sysloop.clone(), &wifi_creds)
            .map_err(|e| anyhow!(e))?;
        error!("[ERROR] exited settings without error, this should not happen...");
        return Ok(());
    } else {
        info!("going into game-play mode");
    }

    info!("Initializing Wi-Fi...");
    let wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs.clone()))?,
        sysloop,
        timer.clone(),
    )?;

    let (tx, rx) = mpsc::channel::<LedChange>(CHANNEL_SIZE);
    let (tx_ei, rx_ei) = broadcast::channel::<EncoderInfo>(100);

    info!("Starting async run loop");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let mut wifi_loop = WifiLoop::new(wifi);
            wifi_loop.configure(&wifi_creds).await?;
            wifi_loop.initial_connect().await?;
            //TODO: restart to settings if wifi doesn't connect

            // check online-go authorization
            let login_info = match OnlineGoLoginInfo::get_saved_in_nvs(nvs.clone())? {
                None => todo!("Restart in settings mode"),
                Some(tok) => tok,
            };
            let OauthResponseValid { access_token, .. } = match login_info.auth_with_password()? {
                Ok(token) => token,
                Err(err) => {
                    error!("{:?}", err);
                    todo!("Restart in settings mode");
                }
            };
            // todo: handle getting new token when first expires
            let auth_token = access_token;

            // Check for current

            info!("Preparing to launch rotary encoder monitor...");
            let mut rc = tokio::spawn(async {
                let mut rotary_encoder = rotary_encoder;
                rotary_encoder.monitor_encoder_spin(tx_ei).await
            });
            info!("Preparing to launch led blinker...");
            let mut led = tokio::spawn(led_ctrl::<{ BOARD_SIZE * BOARD_SIZE }, { BOARD_SIZE }>(
                board_leds, channel0, rx,
            ));
            // info!("Preparing to launch echo settings...");
            // tokio::spawn(echo_server(tx.clone()));

            info!("Entering main Wi-Fi run loop...");
            let mut wifi = tokio::spawn(wifi_loop.stay_connected());
            info!("starting main loop");
            let mut main_loop = tokio::spawn(main_loop(tx.clone(), tx_ei.subscribe(), &auth_token));

            return select! {
                result = &mut rc =>{
                    info!("Rotation Control exited");
                    result?
                }
               result = &mut led => {
                      info!("LED exited");
                    result?
                }
                result = &mut main_loop => {
                      info!("main_loop exited");
                    result?
                }
                result = &mut wifi =>{
                      info!("Wifi exited");
                    let r:Result<()> = result?.map_err(|e| anyhow!(e));
                    r
                }
            };
        })?;
    Ok(())
}

const BLACK_SPOT: Rgb = Rgb::new(50, 0, 0);
const WHITE_SPOT: Rgb = Rgb::new(0, 50, 0);

const EMPTY_SPOT: Rgb = Rgb::new(0, 0, 0);

async fn main_loop(
    led_tx: Sender<LedChange>,
    encoder_rx: BrReceiver<EncoderInfo>,
    auth_token: &AuthToken,
) -> Result<()> {
    let current_player = api::get_current_player(&auth_token)?;

    let game_list = api::get_current_player_games(&auth_token)?;

    // TODO:Select A specific game, rn just picks the first in the list

    if let None = game_list.games.first() {
        return Err(anyhow!(
            "Failed to get a game in game list, game list len  < 1"
        ));
    }

    let current_game = Arc::new(game_list.games.first().unwrap().clone());

    let game_board_data = current_game.get_detail(&auth_token)?;
    // TODO: handle the errors in a way that the user can see, maybe store in nvs?

    let overlay = LedOverlay::<{ BOARD_SIZE }, { BOARD_SIZE }, { 2 }>::new();

    // is the game complete?
    if (current_game.is_game_over()) {
        let gameboard_changes:Vec<LedChange> = game_board_data.board_iter().map(|(x, y, v)| {
            let color: BoardColor = {
                let res: Result<BoardColor> = (*v).try_into();
                res.unwrap_or_else(|err| {
                    error!("Unknown Board Color:{err}");
                    BoardColor::Empty
                })
            };

            let rgb = match color {
                BoardColor::Empty => EMPTY_SPOT,
                BoardColor::Black => BLACK_SPOT,
                BoardColor::White => WHITE_SPOT,
            };

            LedChange::new(x , y , rgb)
        }).collect();
        let sccore_changes:Vec<LedChange> = 
        // todo: create score display for led panel
        // let score_changes = current_game.
        loop {}
    } else {
    }
    Ok(())
}

async fn requester(tx: Sender<LedChange>) -> Result<()> {
    let mut t = true;
    // loop {
    // get("https://google.com")?;

    let bs = test_connection()?;

    neopixel::go_board::show_board(&tx, &bs.board, bs.height(), bs.width()).await?;

    //     sleep(Duration::from_millis(5000)).await;
    // }
    loop {
        tx.send(LedChange::new(0, 0, Rgb::new(0, 0, 50))).await?;
        sleep(Duration::from_millis(1000)).await;
        tx.send(LedChange::new(0, 0, Rgb::new(0, 0, 0))).await?;
        sleep(Duration::from_millis(1000)).await;
        info!("looping reqs");
    }
    Ok(())
}

async fn echo_server(tx: Sender<LedChange>) -> Result<()> {
    let addr = format!("0.0.0.0:{TCP_LISTENING_PORT}");

    info!("Binding to {addr}...");
    let listener = TcpListener::bind(&addr).await?;

    loop {
        info!("Waiting for new connection on socket: {listener:?}");
        let (socket, _) = listener.accept().await?;

        info!("Spawning handle for: {socket:?}...");
        tokio::spawn(async move {
            info!("Spawned handler!");
            let peer = socket.peer_addr();
            if let Err(e) = serve_client(socket).await {
                info!("Got error handling {peer:?}: {e:?}");
            }
        });
    }
}

async fn serve_client(mut stream: TcpStream) -> Result<()> {
    info!("Handling {stream:?}...");

    let mut buf = [0u8; 512];
    loop {
        info!("About to read...");
        let n = stream.read(&mut buf).await?;
        info!("Read {n} bytes...");

        if n == 0 {
            break;
        }

        stream.write_all(&buf[0..n]).await?;
        info!("Wrote {n} bytes back...");
    }

    Ok(())
}
