use crate::onlinego::api::OnlineGoLoginInfo;
use crate::onlinego::status_codes::StatusCode;
use crate::settings::captive_portal::CaptivePortal;
use crate::settings::server::handlers::{OnlineGoAccountStatus, WifiStatus};
use crate::wifi::WifiCredentials;
use anyhow::{anyhow, Result};
use embedded_svc::http::server::Request;
use embedded_svc::http::Method;
use embedded_svc::wifi::asynch::Wifi;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer};
use esp_idf_svc::io::Write;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use serde_json::json;
use std::fmt::{Debug, Display};
use std::net::Ipv4Addr;

pub const IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(192, 168, 42, 1);
pub struct CaptiveServer<'s> {
    server: EspHttpServer<'s>,
}

enum MIMEtype {
    Javascript,
    CSS,
    HTML,
}

impl AsRef<str> for MIMEtype {
    fn as_ref(&self) -> &'static str {
        match &self {
            MIMEtype::Javascript => "text/javascript",
            MIMEtype::CSS => "text/css",
            MIMEtype::HTML => "text/plain ",
        }
    }
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
    fn add_static_file(
        &mut self,
        route: &str,
        mut file_str: &'static str,
        mimetype: MIMEtype,
    ) -> Result<()> {
        let h = format!("{}; charset=utf-8", mimetype.as_ref());
        self.server.fn_handler(route, Method::Get, move |request| {
            request
                .into_response(200, None, &[("Content-Type", h.as_str())])?
                .write_all(file_str.as_bytes())?;
            Ok(()) as Result<()>
        })?;
        Ok(())
    }

    fn set_up_pages_routes(&mut self) -> Result<()> {
        self.add_static_file(
            "/assets/index.js",
            include_str!("web/assets/index.js"),
            MIMEtype::Javascript,
        )?;
        self.add_static_file(
            "/styles.css",
            include_str!("web/styles.css"),
            MIMEtype::CSS,
        )?;
        self.add_static_file(
            "/assets/index.js",
            include_str!("web/assets/index.css"),
            MIMEtype::CSS,
        )?;
        self.add_static_file("/", include_str!("web/index.html"), MIMEtype::HTML)?;
        Ok(())
    }

    pub fn set_up_route<T: CaptiveServerHandler<R>, R: AsRef<str>>(
        &mut self,
        options: T::RequestExtraParameters,
    ) -> Result<()> {
        let handler = T::create_handler(options);
        self.server.fn_handler(
            T::url().as_ref(),
            T::method(),
            move |mut request| -> Result<()> {
                let result = handler(&mut request);
                let (response) = result.unwrap_or_else(|error| {
                    DataResponse::UnhandledError(StatusCode::INTERNAL_SERVER_ERROR, error)
                });
                response.send(request)
            },
        )?;
        Ok(())
    }
    fn set_up_data_routes(
        &mut self,
        partition: EspNvsPartition<NvsDefault>,
        wifi_status: WifiStatus,
    ) -> Result<()> {
        WifiStatus::set_up_route(self, wifi_status)?;
        WifiCredentials::set_up_route(self, partition.clone())?;
        OnlineGoAccountStatus::set_up_route(self, partition.clone())?;
        OnlineGoLoginInfo::set_up_route(self, partition.clone())?;
        Ok(())
    }
}

pub trait CaptiveServerHandler<R: AsRef<str>>: Sized {
    /// type for extra parameters needed to handle the specific request, used to generate a "pure" request handler closure
    type RequestExtraParameters;

    /// the HTTP method to handle for the specified [Self::url]
    fn method() -> Method;

    /// the url string / route to handle using [Self::create_handler]
    fn url() -> R;

    /// create a closure to handle incoming requests to [Self::url] with the http method [Self::method]
    /// it provides a closure so that each implementation may require different parameters using [Self::RequestExtraParameters]
    fn create_handler(
        arg: Self::RequestExtraParameters,
    ) -> impl for<'r> Fn(&mut Request<&mut EspHttpConnection<'r>>) -> Result<DataResponse> + Send + 'static;

    fn set_up_route(server: &mut CaptiveServer, arg: Self::RequestExtraParameters) -> Result<()> {
        server.set_up_route::<Self, R>(arg)
    }
}

#[derive(Debug)]
pub enum DataResponse {
    UnhandledError(StatusCode, anyhow::Error),
    HandledErr(StatusCode, serde_json::value::Value),
    Ok(Option<serde_json::value::Value>),
}
impl DataResponse {
    fn to_response_value(&self) -> (serde_json::value::Value, u16) {
        match &self {
            DataResponse::UnhandledError(status, err) => {
                let disp = format!("{:#?}", err);
                (
                    json!({
                            "is_ok":false,
                            "value":disp
                        }
                    ),
                    (*status).into(),
                )
            }
            DataResponse::HandledErr(status, value) => (
                json!({
                        "is_ok":false,
                        "value":value
                    }
                ),
                (*status).into(),
            ),
            DataResponse::Ok(value) => (
                json!({
                        "is_ok":false,
                        "value":value
                    }
                ),
                200,
            ),
        }
    }
    fn send(self, r: Request<&mut EspHttpConnection>) -> Result<()> {
        let (value, code) = self.to_response_value();

        r.into_response(code, None, &[("Content-Type", "text/json; charset=utf-8")])?
            .write_all(value.to_string().as_bytes())?;

        if let DataResponse::UnhandledError(_, err) = self {
            return Err(err);
        }

        Ok(())
    }
}
