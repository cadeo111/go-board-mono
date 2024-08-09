use std::str;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
pub use embedded_svc::http::client::Client;
pub use embedded_svc::http::Method;
pub use embedded_svc::io::Read;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys;
use esp_idf_svc::sys::{esp, esp_app_desc};
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio::time::Instant;
use tokio::time::sleep;
use tokio::time::timeout;
use tokio::time::timeout_at;

use crate::https::get;
use crate::neopixel::led_ctrl::{led_ctrl, LedChange};
use crate::neopixel::rgb::Rgb;
use crate::wifi::WifiLoop;

mod neopixel;
mod wifi;
mod https;
mod encoder;

const BOARD_SIZE: usize = 16;
const CHANNEL_SIZE: usize = BOARD_SIZE * 2;


const WIFI_SSID: &'static str = env!("WIFI_SSID");
const WIFI_PASSWORD: &'static str = env!("WIFI_PASSWORD");

// To test, run `cargo run`, then when the server is up, use `nc -v espressif 12345` from
// a machine on the same Wi-Fi network.
const TCP_LISTENING_PORT: u16 = 12345;

esp_app_desc!();



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
    (esp! { unsafe { sys::esp_vfs_eventfd_register(&config) } })?;

    info!("Setting up board...");
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let timer = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    info!("Initializing LED...");

    let led = peripherals.pins.gpio3;
    let channel0 = peripherals.rmt.channel0;


    info!("Initializing Wi-Fi...");
    let wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?,
        sysloop,
        timer.clone())?;

    let (tx, rx) = mpsc::channel::<LedChange>(CHANNEL_SIZE);

    info!("Starting async run loop");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let mut wifi_loop = WifiLoop::new(wifi);
            wifi_loop.configure().await?;
            wifi_loop.initial_connect().await?;

            info!("Preparing to launch led blinker...");
            tokio::spawn(led_ctrl::<{ BOARD_SIZE * BOARD_SIZE }>(led, channel0, rx));
            info!("Preparing to launch echo server...");
            tokio::spawn(echo_server(tx.clone()));
            info!("Preparing to launch requester...");
            tokio::spawn(requester(tx.clone()));
            info!("Entering main Wi-Fi run loop...");
            wifi_loop.stay_connected().await
        })?;
    Ok(())
}


async fn requester(tx: Sender<LedChange>) -> Result<()> {
    let mut t = true;
    loop {
        get("https://google.com")?;
        tx.send(LedChange::new(0, 0,
                               if t {
                                   t = false;
                                   Rgb::new(0, 0, 16)
                               } else {
                                   t = true;
                                   Rgb::new(0, 16, 0)
                               },
        )).await?;


        sleep(Duration::from_millis(5000)).await;
    }
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

