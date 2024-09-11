use crate::onlinego;
use crate::onlinego::api::{get_current_player_games, GameList, OnlineGoLoginInfo};
use crate::onlinego::status_codes::StatusCode;
use crate::settings::server::deserialize_json_req::{
    deserialize_json_from_request, DataResponseOrValue,
};
use crate::settings::server::server::{CaptiveServerHandler, DataResponse};
use crate::storage::SaveInNvs;
use crate::wifi::WifiCredentials;
use anyhow::{anyhow, Error, Result};
use embedded_svc::http::server::Request;
use embedded_svc::http::Method;
use esp_idf_svc::hal::reset;
use esp_idf_svc::http::server::EspHttpConnection;
use esp_idf_svc::io::Write;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use unicode_segmentation::UnicodeSegmentation;

pub enum HandlerRoute {
    WifiCredentials,
    WifiStatus,
    OnlineGoAccountStatus,
    OnlineGoLoginInfo,
    OnlineGoGamesList,
}
impl AsRef<str> for HandlerRoute {
    fn as_ref(&self) -> &'static str {
        match self {
            HandlerRoute::WifiCredentials => "/save-wifi-credentials",
            HandlerRoute::WifiStatus => "/wifi-status",
            HandlerRoute::OnlineGoAccountStatus => "/online-go-status",
            HandlerRoute::OnlineGoLoginInfo => "/online-go-login",
            HandlerRoute::OnlineGoGamesList => "/online-go-games-list",
        }
    }
}

impl CaptiveServerHandler<HandlerRoute> for WifiCredentials {
    type RequestExtraParameters = (EspNvsPartition<NvsDefault>);

    fn method() -> Method {
        Method::Post
    }

    fn url() -> HandlerRoute {
        HandlerRoute::WifiCredentials
    }

    fn create_handler(
        partition: Self::RequestExtraParameters,
    ) -> impl for<'r> Fn(&mut Request<&mut EspHttpConnection<'r>>) -> Result<DataResponse> + Send + 'static
    {
        move |request| {
            let data = deserialize_json_from_request::<Self>(request);
            match data {
                DataResponseOrValue::Response(dr) => Ok(dr),
                DataResponseOrValue::Value(creds) => {
                    creds.set_saved_in_nvs(partition.clone())?;
                    info!("Saved new wifi credentials {creds:?}, restarting...");
                    reset::restart();
                }
            }
        }
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
        let letters = creds
            .password
            .graphemes(true)
            .collect::<heapless::Vec<&str, 32>>();
        let count = letters.len() - 1;
        let letter = letters[0];
        Self {
            connected,
            ssid: creds.ssid.to_string(),
            first_letter_of_password: letter.to_string(),
            length_of_password: count as u8,
        }
    }
}
impl CaptiveServerHandler<HandlerRoute> for WifiStatus {
    type RequestExtraParameters = (WifiStatus);

    fn method() -> Method {
        Method::Get
    }
    fn url() -> HandlerRoute {
        HandlerRoute::WifiStatus
    }

    fn create_handler(
        (status): Self::RequestExtraParameters,
    ) -> impl for<'r> Fn(&mut Request<&mut EspHttpConnection<'r>>) -> Result<DataResponse> + Send + 'static
    {
        move |_| Ok(DataResponse::Ok(Some(serde_json::to_value(&status)?)))
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Default)]
pub struct OnlineGoAccountStatus {
    pub authorized: bool,
    pub username: String,
    pub first_letter_of_password: String,
    pub length_of_password: u8,
}

impl OnlineGoAccountStatus {
    pub fn new(authorized: bool, username: &str, password: &str) -> Self {
        let letters = password
            .graphemes(true)
            .collect::<heapless::Vec<&str, 32>>();
        let count = letters.len() - 1;
        let letter = letters[0];
        Self {
            authorized,
            username: username.to_string(),
            first_letter_of_password: letter.to_string(),
            length_of_password: count as u8,
        }
    }
}

impl CaptiveServerHandler<HandlerRoute> for OnlineGoAccountStatus {
    type RequestExtraParameters = (EspNvsPartition<NvsDefault>);

    fn method() -> Method {
        Method::Get
    }

    fn url() -> HandlerRoute {
        HandlerRoute::OnlineGoAccountStatus
    }

    fn create_handler(
        nvs: Self::RequestExtraParameters,
    ) -> impl for<'r> Fn(&mut Request<&mut EspHttpConnection<'r>>) -> Result<DataResponse> + Send + 'static
    {
        move |request| {
            let saved_info = OnlineGoLoginInfo::get_saved_in_nvs(nvs.clone())?;
            match saved_info {
                Some(saved_info) => match saved_info.auth_with_password()? {
                    Ok(_) => {
                        let res_data = OnlineGoAccountStatus::new(
                            true,
                            &saved_info.username,
                            &saved_info.password,
                        );
                        Ok(DataResponse::Ok(Some(serde_json::to_value(&res_data)?)))
                    }
                    Err(err) => Ok(DataResponse::HandledErr(
                        StatusCode::UNAUTHORIZED,
                        serde_json::to_value(&err)?,
                    )),
                },
                None => Ok(DataResponse::HandledErr(
                    StatusCode::UNAUTHORIZED,
                    serde_json::to_value(&OnlineGoAccountStatus::default())?,
                )),
            }
        }
    }
}

impl CaptiveServerHandler<HandlerRoute> for OnlineGoLoginInfo {
    type RequestExtraParameters = (EspNvsPartition<NvsDefault>);

    fn method() -> Method {
        Method::Get
    }

    fn url() -> HandlerRoute {
        HandlerRoute::OnlineGoLoginInfo
    }

    fn create_handler(
        partition: Self::RequestExtraParameters,
    ) -> impl for<'r> Fn(&mut Request<&mut EspHttpConnection<'r>>) -> Result<DataResponse> + Send + 'static
    {
        move |request| {
            let data = deserialize_json_from_request::<Self>(request);
            match data {
                DataResponseOrValue::Response(dr) => Ok(dr),
                DataResponseOrValue::Value(login_info) => {
                    login_info.set_saved_in_nvs(partition.clone())?;
                    info!("Saved new online-go login info {login_info:?}, checking status...");
                    Ok(DataResponse::Ok(None))
                }
            }
        }
    }
}

pub struct OnlineGoGamesList {}

impl CaptiveServerHandler<HandlerRoute> for OnlineGoGamesList {
    type RequestExtraParameters = EspNvsPartition<NvsDefault>;

    fn method() -> Method {
        Method::Get
    }

    fn url() -> HandlerRoute {
        HandlerRoute::OnlineGoGamesList
    }

    /// sends [GameList]
    fn create_handler(
        nvs: Self::RequestExtraParameters,
    ) -> impl for<'r> Fn(&mut Request<&mut EspHttpConnection<'r>>) -> Result<DataResponse> + Send + 'static
    {
        move |_| {
            let saved_info = OnlineGoLoginInfo::get_saved_in_nvs(nvs.clone())?;
            match saved_info {
                Some(saved_info) => match saved_info.auth_with_password()? {
                    Ok(valid) => {
                        let games: GameList = get_current_player_games(&valid.access_token)?;
                        Ok(DataResponse::Ok(Some(serde_json::to_value(&games)?)))
                    }
                    Err(err) => Ok(DataResponse::HandledErr(
                        StatusCode::UNAUTHORIZED,
                        serde_json::to_value(&err)?,
                    )),
                },
                None => Ok(DataResponse::HandledErr(
                    StatusCode::UNAUTHORIZED,
                    serde_json::to_value(&OnlineGoAccountStatus::default())?,
                )),
            }
        }
    }
}
