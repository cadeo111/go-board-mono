use std::str;
use std::time::Duration;
use anyhow::{bail, Result};
use embedded_svc::{
    http::{client::Client, Method},
    io::Read,
};
use esp_idf_svc::{sys, wifi::{
    ClientConfiguration as WifiClientConfiguration,
    Configuration as WifiConfiguration,
}};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::*;

use esp_idf_svc::http::client::{
    client,
    Configuration as HttpConfiguration,
    EspHttpConnection,
};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{esp, esp_app_desc, EspError};
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;
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


    info!("Initializing Wi-Fi...");
    let wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs))?,
        sysloop,
        timer.clone())?;

    info!("Starting async run loop");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let mut wifi_loop = WifiLoop { wifi };
            wifi_loop.configure().await?;
            wifi_loop.initial_connect().await?;

            info!("Preparing to launch echo server...");
            tokio::spawn(echo_server());
            info!("Preparing to launch requester...");
            tokio::spawn(requester());
            info!("Entering main Wi-Fi run loop...");
            wifi_loop.stay_connected().await
            
            
        })?;
    Ok(())
}


pub struct WifiLoop<'a> {
    wifi: AsyncWifi<EspWifi<'a>>,
}

impl<'a> WifiLoop<'a> {
    pub async fn configure(&mut self) -> Result<(), EspError> {
        info!("Setting Wi-Fi credentials...");
        self.wifi.set_configuration(&WifiConfiguration::Client(WifiClientConfiguration {
            ssid: WIFI_SSID.parse().unwrap(),
            password: WIFI_PASSWORD.parse().unwrap(),
            ..Default::default()
        }))?;

        info!("Starting Wi-Fi driver...");
        self.wifi.start().await
    }

    pub async fn initial_connect(&mut self) -> Result<(), EspError> {
        self.do_connect_loop(true).await
    }

    pub async fn stay_connected(mut self) -> Result<(), EspError> {
        self.do_connect_loop(false).await
    }

    async fn do_connect_loop(
        &mut self,
        exit_after_first_connect: bool,
    ) -> Result<(), EspError> {
        let wifi = &mut self.wifi;
        loop {
            // Wait for disconnect before trying to connect again.  This loop ensures
            // we stay connected and is commonly missing from trivial examples as it's
            // way too difficult to showcase the core logic of an example and have
            // a proper Wi-Fi event loop without a robust async runtime.  Fortunately, we can do it
            // now!
            wifi.wifi_wait(|wifi| wifi.is_up(), None).await?;

            info!("Connecting to Wi-Fi...");
            wifi.connect().await?;

            info!("Waiting for association...");
            wifi.ip_wait_while(|wifi| wifi.is_up().map(|s| !s), None).await?;

            if exit_after_first_connect {
                return Ok(());
            }
        }
    }
}


fn get(url: impl AsRef<str>) -> Result<()> {
    // 1. Create a new EspHttpClient. (Check documentation)
    // ANCHOR: connection
    let connection = EspHttpConnection::new(&HttpConfiguration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    // ANCHOR_END: connection
    let mut client = Client::wrap(connection);

    // 2. Open a GET request to `url`
    let headers = [("accept", "text/plain")];
    let request = client.request(Method::Get, url.as_ref(), &headers)?;
    // 3. Submit write request and check the status code of the response.
    // Successful http status codes are in the 200..=299 range.
    let response = request.submit()?;
    let status = response.status();
    println!("Response code: {}\n", status);
    match status {
        200..=299 => {
            // 4. if the status is OK, read response data chunk by chunk into a buffer and print it until done
            //
            // NB. see http_client.rs for an explanation of the offset mechanism for handling chunks that are
            // split in the middle of valid UTF-8 sequences. This case is encountered a lot with the given
            // example URL.
            let mut buf = [0_u8; 256];
            let mut offset = 0;
            let mut total = 0;
            let mut reader = response;
            loop {
                if let Ok(size) = Read::read(&mut reader, &mut buf[offset..]) {
                    if size == 0 {
                        break;
                    }
                    total += size;
                    // 5. try converting the bytes into a Rust (UTF-8) string and print it
                    let size_plus_offset = size + offset;
                    match str::from_utf8(&buf[..size_plus_offset]) {
                        Ok(text) => {
                            print!("{}", text);
                            offset = 0;
                        }
                        Err(error) => {
                            let valid_up_to = error.valid_up_to();
                            unsafe {
                                print!("{}", str::from_utf8_unchecked(&buf[..valid_up_to]));
                            }
                            buf.copy_within(valid_up_to.., 0);
                            offset = size_plus_offset - valid_up_to;
                        }
                    }
                }
            }
            println!("Total: {} bytes", total);
        }
        _ => bail!("Unexpected response code: {}", status),
    }

    Ok(())
}


async fn requester() -> Result<()> {
    loop {
        get("https://google.com")?;
        sleep(Duration::from_millis(1000)).await;
    }
}


async fn echo_server() -> Result<()> {
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
