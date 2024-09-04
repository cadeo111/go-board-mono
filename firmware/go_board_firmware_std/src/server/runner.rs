use crate::onlinego::https::{request as outside_request, RequestType};
use crate::server::captive_portal::CaptivePortal;
use crate::server::dns::SimpleDns;
use crate::{WIFI_PASSWORD, WIFI_SSID};
use anyhow::{anyhow, Error, Result};
use axum::response;
use embedded_svc::http::server::Request;
use embedded_svc::ipv4::ClientConfiguration as ipv4ClientConfiguration;
use esp_idf_svc::hal::reset;
use esp_idf_svc::http::server::EspHttpConnection;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::wifi::BlockingWifi;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    http::{
        server::{Configuration, EspHttpServer},
        Method,
    },
    io::Write,
    ipv4::{self, Mask, RouterConfiguration, Subnet},
    log::EspLogger,
    netif::{EspNetif, NetifConfiguration, NetifStack},
    nvs::EspDefaultNvsPartition,
    sys::{self, EspError},
    wifi::{self, AccessPointConfiguration, ClientConfiguration, EspWifi, WifiDriver},
};
use log::info;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{ErrorKind, Read};
use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
use unicode_segmentation::UnicodeSegmentation;

pub const IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(192, 168, 42, 1);

pub fn run() -> Result<()> {
    unsafe {
        sys::nvs_flash_init();
    }
    sys::link_patches();
    EspLogger::initialize_default();

    let event_loop = EspSystemEventLoop::take()?;
    let peripherals = Peripherals::take()?;

    info!("Starting Wi-Fi...");
    let wifi_driver = WifiDriver::new(
        peripherals.modem,
        event_loop.clone(),
        EspDefaultNvsPartition::take().ok(),
    )?;
    let mut wifi = EspWifi::wrap_all(
        wifi_driver,
        EspNetif::new(NetifStack::Sta)?,
        // EspNetif::new_with_conf(&NetifConfiguration {
        //     ip_configuration: ipv4::Configuration::Client(ipv4ClientConfiguration {
        //         dns: Some(Ipv4Addr::new(1, 1, 1, 1)),
        //         dhcp_enabled: true,
        //     }),
        //     ..NetifConfiguration::wifi_default_client()
        // })?,
        EspNetif::new_with_conf(&NetifConfiguration {
            ip_configuration: ipv4::Configuration::Router(RouterConfiguration {
                subnet: Subnet {
                    gateway: IP_ADDRESS,
                    mask: Mask(24),
                },
                dhcp_enabled: true,
                dns: Some(IP_ADDRESS),
                secondary_dns: Some(Ipv4Addr::new(1, 0, 0, 1)),
            }),
            ..NetifConfiguration::wifi_default_router()
        })?,
    )
    .expect("WiFi init failed");

    let ssid = WIFI_SSID.to_string();
    let wifi_password = WIFI_PASSWORD.to_string();

    wifi.set_configuration(&wifi::Configuration::Mixed(
        ClientConfiguration {
            ssid: (&ssid).parse().unwrap(),
            password: (&wifi_password).parse().unwrap(),
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "Go_Board_Settings".parse().unwrap(),
            ..Default::default()
        },
    ))?;
    let mut wifi = BlockingWifi::wrap(wifi, event_loop.clone())?;

    wifi.start()?;
    info!("Wifi started");

    let result = wifi.connect();
    let is_connected;
    if let Err(err) = result {
        info!("Wifi NOT connected {err}");
        is_connected = false;
    } else {
        info!("Wifi connected");
        is_connected = true;
        println!("CONNECTED  connected!");
    }
    let result = wifi.wait_netif_up();
    if let Err(err) = result {
        info!("wifi Netif NOT up {err}");
    } else {
        info!("wifi netif up");
    }
    let wifi_status = WifiStatus::new(is_connected, ssid, wifi_password);
    info!("Starting DNS server...");
    let mut dns = SimpleDns::try_new(IP_ADDRESS).expect("DNS server init failed");
    thread::spawn(move || loop {
        dns.poll().ok();
        sleep(Duration::from_millis(50));
    });
    info!("DNS server started");

    let store = Arc::new(Mutex::new(String::new()));

    info!("Starting HTTP server...");
    let config = Configuration::default();
    let mut server = EspHttpServer::new(&config).expect("HTTP server init failed");
    CaptivePortal::attach(&mut server, IP_ADDRESS).expect("Captive portal attach failed");

    let result = crate::onlinego::https::request(RequestType::Get {
        url: "https://jsonplaceholder.typicode.com/todos/1",
    });
    match result {
        Ok((status, txt)) => {
            info!("\n\nREQ:\n\n{status}\n {txt}");
        }
        Err(err) => {
            info!("\n\nREQ:\n\n{err:?}");
        }
    }
    server.fn_handler("/vite.svg", Method::Get, move |request| {
        request
            .into_ok_response()?
            .write_all(include_bytes!("web/vite.svg"))?;
        Ok(()) as Result<()>
    })?;
    server.fn_handler("/assets/index.js", Method::Get, move |request| {
        request
            .into_response(
                200,
                None,
                &[("Content-Type", "text/javascript; charset=utf-8")],
            )?
            .write_all(include_bytes!("web/assets/index-DrAcb8O5.js"))?;
        Ok(()) as Result<()>
    })?;
    server.fn_handler("/assets/index.css", Method::Get, move |request| {
        request
            .into_response(200, None, &[("Content-Type", "text/css; charset=utf-8")])?
            .write_all(include_bytes!("web/assets/index-C6-IzDiT.css"))?;
        Ok(()) as Result<()>
    })?;
    server.fn_handler("/styles.css", Method::Get, |request| {
        request
            .into_response(200, None, &[("Content-Type", "text/css; charset=utf-8")])?
            .write_all(include_bytes!("web/styles.css"))?;
        Ok(()) as Result<()>
    })?;
    server.fn_handler(
        WifiSaveData::url_string(),
        Method::Post,
        WifiSaveData::handle_request,
    )?;
    server.fn_handler(WifiStatus::url_string(), Method::Get, move |request| {
        wifi_status.handle_request(request)
    })?;

    server.fn_handler("/", Method::Get, move |request| {
        let page = include_str!("web/index.html");
        request.into_ok_response()?.write_all(page.as_bytes())?;
        Ok(()) as Result<()>
    })?;

    let memo = store.clone();
    server.fn_handler("/", Method::Post, move |mut request| {
        let mut scratch = [0; 256];
        let len = request.read(&mut scratch)?;
        let req = std::str::from_utf8(&scratch[0..len])?;
        if let Some(("memo", req)) = req.split_once('=') {
            *memo.lock().map_err(|e| anyhow!(e.to_string()))? =
                urlencoding::decode(req)?.into_owned();
        };
        request.into_response(302, None, &[("Location", "/")])?;
        Ok(()) as Result<()>
    })?;

    info!("HTTP server started");

    loop {
        sleep(Duration::from_millis(100));
    }
}

struct ReadableRequest<'r, 'c>(Request<&'r mut EspHttpConnection<'c>>);

impl<'r, 'c> ReadableRequest<'r, 'c> {
    fn into_request(self) -> Request<&'r mut EspHttpConnection<'c>> {
        self.0
    }
}

impl<'r, 'c> Read for &mut ReadableRequest<'r, 'c> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf).map_err(|e| EspIOReaderError(e).into())
    }
}

struct EspIOReaderError(EspIOError);

impl Into<std::io::Error> for EspIOReaderError {
    fn into(self) -> std::io::Error {
        std::io::Error::new(ErrorKind::Other, self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
enum DataRequestResponseOption {
    Error,
    Ok,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
struct DataRequestResponse {
    is_ok: DataRequestResponseOption,
    value: Option<String>,
}

impl DataRequestResponse {
    fn ok_response() -> DataRequestResponse {
        const OK_RESPONSE: DataRequestResponse = DataRequestResponse {
            is_ok: DataRequestResponseOption::Ok,
            value: None,
        };
        OK_RESPONSE
    }
    fn err_response(err: Error) -> DataRequestResponse {
        DataRequestResponse {
            is_ok: DataRequestResponseOption::Ok,
            value: Some(format!("{:#?}", err)),
        }
    }
}

/// an error would basically be unexpected failure, if it is an expected failure then a DataRequestResponse will be sent
fn handle_json_request<'a: 'a, 'b: 'b, T>(
    request: Request<&'a mut EspHttpConnection<'b>>,
) -> Result<Option<(Request<&'a mut EspHttpConnection<'b>>, T)>>
where
    T: DeserializeOwned,
{
    // TODO: Handle error if incorrect data sent in request
    let (v, data) = {
        let mut rr = ReadableRequest(request);
        let data: Result<T> = serde_json::from_reader(&mut rr).map_err(|e| anyhow!(e));
        (rr.into_request(), data)
    };
    match data {
        Ok(data) => Ok(Some((v, data))),
        Err(err) => {
            v.into_status_response(400)?.write_all(
                serde_json::to_string(&DataRequestResponse::err_response(err))?.as_bytes(),
            )?;
            Ok(None)
        }
    }

    // // TODO: Handle error if incorrect data sent in request
}

#[derive(Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct WifiSaveData {
    pub ssid: String,
    pub password: String,
}

impl WifiSaveData {
    fn handle_request(request: Request<&mut EspHttpConnection>) -> Result<()> {
        let possible_data = handle_json_request::<WifiSaveData>(request)?;
        if let Some((req, data)) = possible_data {
            info!("GOT  DATA: {data:?}");
            reset::restart();
            req.into_ok_response()?;
        }
        Ok(()) as Result<()>
    }

    fn url_string() -> &'static str {
        "/save-wifi-credentials"
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct WifiStatus {
    pub connected: bool,
    pub ssid: String,
    pub first_letter_of_password: String,
    pub length_of_password: u8,
}

impl WifiStatus {
    pub fn new(connected: bool, ssid: String, password: String) -> Self {
        let letter = password.graphemes(true).take(1).collect::<String>();
        let count = password.graphemes(true).take(1).count() as u8;

        Self {
            connected,
            ssid,
            first_letter_of_password: letter,
            length_of_password: count,
        }
    }

    fn handle_request(&self, request: Request<&mut EspHttpConnection>) -> Result<()> {
        // let possible_data = handle_json_request::<WifiSaveData>(request)?;
        request
            .into_response(
                200,
                None,
                &[("Content-Type", "text/javascript; charset=utf-8")],
            )?
            .write_all(serde_json::to_string(self)?.as_bytes())?;

        Ok(()) as Result<()>
    }

    fn url_string() -> &'static str {
        "/wifi-status"
    }
}
