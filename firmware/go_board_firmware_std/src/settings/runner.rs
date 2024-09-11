use crate::onlinego::https::{request as outside_request, RequestType};
use crate::settings::captive_portal::CaptivePortal;
use crate::settings::dns::SimpleDns;
use crate::settings::server::handlers::WifiStatus;
use crate::settings::server::server::CaptiveServer;
use crate::storage::{NvsNamespace, SaveInNvs};
use crate::wifi::{get_sync_wifi_ap_sta, WifiCredentials};
use anyhow::{anyhow, Error, Result};
use embedded_svc::http::server::Request;
use embedded_svc::ipv4::ClientConfiguration as ipv4ClientConfiguration;
use esp_idf_svc::eventloop::{EspEventLoop, System};
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::reset;
use esp_idf_svc::http::server::EspHttpConnection;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
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

pub fn run(
    partition: EspNvsPartition<NvsDefault>,
    modem: Modem,
    event_loop: EspEventLoop<System>,
    wifi_creds: &WifiCredentials,
) -> Result<()> {
    // let wifi_creds = WifiCredentials::get_saved_in_nvs_with_default(
    //     partition.clone(),
    //     WifiCredentials::get_from_env()?,
    // )?;

    let mut wifi = get_sync_wifi_ap_sta(
        &wifi_creds,
        modem,
        event_loop.clone(),
        partition.clone().into(),
    )?;
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
    let wifi_status = WifiStatus::new(is_connected, wifi_creds);
    info!("Starting DNS settings...");
    let mut dns = SimpleDns::try_new(IP_ADDRESS).expect("DNS settings init failed");
    thread::spawn(move || loop {
        dns.poll().ok();
        sleep(Duration::from_millis(50));
    });
    info!("DNS settings started");

    info!("Starting HTTP settings...");
    let mut server = CaptiveServer::new().map_err(|e| e.context("HTTP settings init failed"))?;
    server.init(partition.clone(), wifi_status)?;
    info!("HTTP settings started");
    info!("sending outside HTTPS request...");
    {
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
        info!("doing web socket stuff");
        // crate::onlinego::websocket::test()?;
    }
    info!("Sent outside HTTPS request");
    loop {
        sleep(Duration::from_millis(1000));
    }
}
