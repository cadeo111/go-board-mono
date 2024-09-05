use crate::settings::captive_portal::CaptivePortal;
use crate::settings::server::deserialize_json_req::deserialize_json_from_request;
use crate::wifi::WifiCredentials;
use anyhow::{anyhow, Error, Result};
use embedded_svc::http::server::Request;
use embedded_svc::http::Method;
use esp_idf_svc::hal::reset;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer};
use esp_idf_svc::io::{EspIOError, Write};
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use log::info;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{ErrorKind, Read};
use std::net::Ipv4Addr;
use unicode_segmentation::UnicodeSegmentation;

pub const IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(192, 168, 42, 1);
pub struct CaptiveServer<'s> {
    server: EspHttpServer<'s>,
}

impl<'s> CaptiveServer<'s> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            server: EspHttpServer::new(&Configuration::default()).map_err(|e| anyhow!(e))?,
        })
    }

    pub fn init(
        &mut self,
        partition: EspNvsPartition<NvsDefault>,
        wifi_status: WifiStatus,
    ) -> Result<()> {
        self.attach_captive_portal()?;
        self.set_up_pages_routes()?;
        self.set_up_data_routes(partition, wifi_status)?;
        Ok(())
    }

    /// add correctly urls for redirecting captive portals
    fn attach_captive_portal(&mut self) -> Result<()> {
        CaptivePortal::attach(&mut self.server, IP_ADDRESS)
            .map_err(|e| e.context("Captive portal attach failed"))
    }
    fn set_up_pages_routes(&mut self) -> Result<()> {
        self.server
            .fn_handler("/assets/index.js", Method::Get, move |request| {
                request
                    .into_response(
                        200,
                        None,
                        &[("Content-Type", "text/javascript; charset=utf-8")],
                    )?
                    .write_all(include_bytes!("web/assets/index-BhptfRq0.js"))?;
                Ok(()) as Result<()>
            })?;
        self.server
            .fn_handler("/assets/index.css", Method::Get, move |request| {
                request
                    .into_response(200, None, &[("Content-Type", "text/css; charset=utf-8")])?
                    .write_all(include_bytes!("web/assets/index-C6-IzDiT.css"))?;
                Ok(()) as Result<()>
            })?;
        self.server
            .fn_handler("/styles.css", Method::Get, |request| {
                request
                    .into_response(200, None, &[("Content-Type", "text/css; charset=utf-8")])?
                    .write_all(include_bytes!("web/styles.css"))?;
                Ok(()) as Result<()>
            })?;
        self.server.fn_handler("/", Method::Get, move |request| {
            request
                .into_ok_response()?
                .write_all(include_bytes!("web/index.html"))?;
            Ok(()) as Result<()>
        })?;
        Ok(())
    }

    fn set_up_data_routes(
        &mut self,
        partition: EspNvsPartition<NvsDefault>,
        wifi_status: WifiStatus,
    ) -> Result<()> {
        self.server
            .fn_handler(WifiSaveData::url_string(), Method::Post, move |request| {
                WifiSaveData::handle_request(request, partition.clone().into())
            })?;
        self.server
            .fn_handler(WifiStatus::url_string(), Method::Get,  move |request| {
                wifi_status.handle_request(request)
            })?;
        // 
        // self.server.fn_handler("/online-go-status", Method::Get, move(){})?;
        // 
        // self.server.fn_handler("/check_online_go_creds", Method::Post, move |request1| {
        //     
        // })
        
        
        
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct WifiSaveData {
    pub ssid: String,
    pub password: String,
}

impl TryInto<WifiCredentials> for WifiSaveData {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<WifiCredentials, Self::Error> {
        WifiCredentials::new(&self.ssid, &self.password)
    }
}

impl WifiSaveData {
    fn handle_request(
        request: Request<&mut EspHttpConnection>,
        partition: EspNvsPartition<NvsDefault>,
    ) -> anyhow::Result<()> {
        let possible_data = deserialize_json_from_request::<WifiSaveData>(request)?;
        if let Some((req, data)) = possible_data {
            let creds: WifiCredentials = data.try_into()?;
            creds.set_saved_credentials(partition)?;
            info!("Saved new wifi credentials {creds:?}");
            reset::restart();
            req.into_ok_response()?;
        }
        Ok(()) as anyhow::Result<()>
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
    pub fn new(connected: bool, creds: &WifiCredentials) -> Self {
        let letters = creds.password.graphemes(true).collect::<heapless::Vec<&str, 32>>();
        let count = letters.len() -1;
        let letter = letters[0];
        
        info!("letter {letter} count {count}");
        Self {
            connected,
            ssid: creds.ssid.to_string(),
            first_letter_of_password: letter.to_string(),
            length_of_password: count as u8,
        }
    }

    fn handle_request(&self, request: Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
        // let possible_data = handle_json_request::<WifiSaveData>(request)?;
        request
            .into_response(
                200,
                None,
                &[("Content-Type", "text/javascript; charset=utf-8")],
            )?
            .write_all(serde_json::to_string(self)?.as_bytes())?;

        Ok(()) as anyhow::Result<()>
    }

    fn url_string() -> &'static str {
        "/wifi-status"
    }
}
