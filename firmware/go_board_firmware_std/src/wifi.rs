use esp_idf_svc::{sys, wifi::{
    ClientConfiguration as WifiClientConfiguration,
    Configuration as WifiConfiguration,
}};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use log::info;

use crate::{WIFI_PASSWORD, WIFI_SSID};

pub struct WifiLoop<'a> {
    wifi: AsyncWifi<EspWifi<'a>>,
}

impl<'a> WifiLoop<'a> {
    pub fn new(wifi: AsyncWifi<EspWifi<'a>>) -> Self {
        Self { wifi }
    }

    pub async fn configure(&mut self) -> anyhow::Result<(), EspError> {
        info!("Setting Wi-Fi credentials...");
        self.wifi.set_configuration(&WifiConfiguration::Client(WifiClientConfiguration {
            ssid: WIFI_SSID.parse().unwrap(),
            password: WIFI_PASSWORD.parse().unwrap(),
            ..Default::default()
        }))?;

        info!("Starting Wi-Fi driver...");
        self.wifi.start().await
    }

    pub async fn initial_connect(&mut self) -> anyhow::Result<(), EspError> {
        self.do_connect_loop(true).await
    }

    pub async fn stay_connected(mut self) -> anyhow::Result<(), EspError> {
        self.do_connect_loop(false).await
    }

    async fn do_connect_loop(
        &mut self,
        exit_after_first_connect: bool,
    ) -> anyhow::Result<(), EspError> {
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