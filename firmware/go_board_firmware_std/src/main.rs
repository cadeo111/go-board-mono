#![feature(never_type)] // allows for signifying functions never return, this is already used by esprs

use std::iter::{Enumerate, FlatMap, Map};
use std::num::NonZeroU32;
use std::slice::Iter;
use std::str;
use std::sync::Arc;

use crate::encoder::{EncoderInfo, RotaryEncoderState};
use crate::neopixel::led_ctrl::{led_ctrl, DisplayOnLeds, LedChange, LedOverlay};
use crate::neopixel::led_font::score_board;
use crate::neopixel::rgb::Rgb;
use crate::onlinego::api;
use crate::onlinego::api::{
    test_connection, BoardColor, BoardState, GameListData, OauthResponseValid, OnlineGoLoginInfo,
    Player,
};
use crate::onlinego::auth_token::AuthToken;
use crate::restart_recovery::{restart_with_recover_option, RecoverOption};
use crate::setup::setup;
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
mod restart_recovery;
mod settings;
mod setup;
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
    let (
        (wifi_creds, wifi),
        (rotary_encoder_state, encoder_info_tx, encoder_info_rx),
        (led_change_rx, led_change_tx, board_led_grid_pin, rmt_channel0),
        nvs,
    ) = setup()?;

    info!("Starting async run loop");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let mut wifi_loop = WifiLoop::new(wifi);
            wifi_loop.configure(&wifi_creds).await?;
            let wifi_connect_result = wifi_loop.initial_connect().await;
            //TODO: restart to settings if wifi doesn't connect
            if let Err(error) = wifi_connect_result {
                error!("Failed to start async wifi: {error}");
                // this will exit the program and force the settings panel
                restart_with_recover_option(RecoverOption::ForceSettingsPanel, nvs.clone())?;
            }

            // check online-go authorization
            let login_info = match OnlineGoLoginInfo::get_saved_in_nvs(nvs.clone())? {
                None => {
                    error!("failed to get online go login info! restarting... ");
                    restart_with_recover_option(RecoverOption::ForceSettingsPanel, nvs.clone())?;
                },
                Some(tok) => tok,
            };
            let OauthResponseValid { access_token, .. } = match login_info.auth_with_password()? {
                Ok(token) => token,
                Err(err) => {
                    error!("Failed to log in to online-go: {:?} \n restarting...", err);
                    restart_with_recover_option(RecoverOption::ForceSettingsPanel, nvs.clone())?;
                }
            };
            // todo: handle getting new token when first expires
            let auth_token = access_token;

            // Check for current

            info!("Preparing to launch rotary encoder monitor...");
            let mut rc = tokio::spawn(async {
                let mut rotary_encoder = rotary_encoder_state;
                rotary_encoder.monitor_encoder_spin(encoder_info_tx).await
            });
            info!("Preparing to launch led blinker...");
            let mut led = tokio::spawn(led_ctrl::<{ BOARD_SIZE * BOARD_SIZE }, { BOARD_SIZE }>(
                board_led_grid_pin,
                rmt_channel0,
                led_change_rx,
            ));
            // info!("Preparing to launch echo settings...");
            // tokio::spawn(echo_server(tx.clone()));

            info!("Entering main Wi-Fi run loop...");
            let mut wifi = tokio::spawn(wifi_loop.stay_connected());
            info!("starting main loop");
            let mut main_loop = tokio::spawn(main_loop(
                led_change_tx.clone(),
                encoder_info_rx,
                auth_token,
            ));

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
    mut encoder_rx: BrReceiver<EncoderInfo>,
    auth_token: AuthToken,
) -> Result<()> {
    let current_player = api::get_current_player(&auth_token)?;

    let game_list = api::get_current_player_games(&auth_token)?;

    // TODO:Select A specific game, rn just picks the first in the list

    if let None = game_list.games.first() {
        return Err(anyhow!(
            "Failed to get a game in game list, game list len  < 1"
        ));
    }
    // TODO: Make sure this ARC<game> is required, prob could get away with local refs
    let current_game = Arc::new(game_list.games.first().unwrap().clone());

    let game_board_data = current_game.get_detail(&auth_token)?;
    // TODO: handle the errors in a way that the user can see, maybe store in nvs?

    let overlay = LedOverlay::<{ BOARD_SIZE }, { BOARD_SIZE }, { 2 }>::new();

    // is the game complete?
    if (current_game.is_game_over()
        //TODO: Remove this after testing
        || true)
    {
        let gameboard_changes: Vec<LedChange> = game_board_data
            .board_iter()
            .map(|(x, y, v)| {
                let color: BoardColor = {
                    let res = (*v).try_into();
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

                LedChange::new(x, y, rgb)
            })
            .collect();
        let score_changes: heapless::Vec<LedChange, 68> = score_board(0, 0, 123, 432);

        // todo: create score display for led panel
        // let score_changes = current_game.

        let mut show_board = true;
        loop {
            // wait for encoder to move
            let _ = encoder_rx.recv().await?;
            // alternate between showing the end board state and the score
            if (show_board) {
                for change in &gameboard_changes {
                    led_tx.send(*change).await?;
                }
            } else {
                for change in &score_changes {
                    led_tx.send(*change).await?;
                }
            }
            show_board = !show_board;
        }
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
