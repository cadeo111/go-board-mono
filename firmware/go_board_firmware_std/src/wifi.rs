use crate::server::runner::IP_ADDRESS;
use crate::storage::NvsNamespace;
use crate::{WIFI_PASSWORD, WIFI_SSID};
use anyhow::{anyhow, Result};
use embedded_svc::ipv4;
use embedded_svc::ipv4::ClientConfiguration as ipv4ClientConfiguration;
use embedded_svc::ipv4::{Mask, RouterConfiguration, Subnet};
use embedded_svc::wifi::{AccessPointConfiguration, ClientConfiguration};
use esp_idf_svc::eventloop::{EspEventLoop, EspSystemEventLoop, System};
use esp_idf_svc::hal::modem;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::netif::{EspNetif, NetifConfiguration, NetifStack};
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvsPartition, NvsDefault};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::timer::{EspTaskTimerService, EspTimerService, Task};
use esp_idf_svc::wifi::{AsyncWifi, BlockingWifi, EspWifi, WifiDriver};
use esp_idf_svc::{
    sys, wifi,
    wifi::{ClientConfiguration as WifiClientConfiguration, Configuration as WifiConfiguration},
};
use log::{info, warn};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

pub struct WifiState {
    pub mac_address: String,
    pub ssid: String,
    ip_addr: RwLock<Option<Ipv4Addr>>,
}

impl WifiState {
    pub async fn ip_addr(&self) -> Option<Ipv4Addr> {
        *self.ip_addr.read().await
    }
}

pub struct WifiConnection<'a> {
    pub state: Arc<WifiState>,
    wifi: AsyncWifi<EspWifi<'a>>,
}

impl<'a> WifiConnection<'a> {
    // Initialize the Wi-Fi driver but do not connect yet.
    pub async fn new(
        modem: Modem,
        event_loop: EspEventLoop<System>,
        timer: EspTimerService<Task>,
        default_partition: Option<EspDefaultNvsPartition>,
        ipv4addr: Ipv4Addr,
    ) -> Result<Self> {
        info!("Initializing wifi...");

        // let wifi_driver = WifiDriver::new(modem, event_loop.clone(), default_partition)?;
        let ipv4_config = ipv4::ClientConfiguration::DHCP(ipv4::DHCPClientSettings::default());
        let net_if = EspNetif::new_with_conf(&NetifConfiguration {
            ip_configuration: ipv4::Configuration::Client(ipv4_config),
            ..NetifConfiguration::wifi_default_client()
        })?;

        info!("Initializing mac...");

        // let netif = EspNetif::new(NetifStack::Sta)?;
        // Store the MAC address in the shared wifi state
        let mac = net_if.get_mac()?;
        let mac_address = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );
        let state = Arc::new(WifiState {
            ip_addr: RwLock::new(None),
            mac_address,
            ssid: WIFI_SSID.to_string(),
        });

        info!("Initializing async wifi");
        // Wrap the Wi-Fi driver in the async wrapper
        // let esp_wifi =
        //     EspWifi::wrap_all(wifi_driver, net_if, EspNetif::new(NetifStack::Ap)?)?;
        let mut wifi = AsyncWifi::wrap(
            EspWifi::wrap_all(
                WifiDriver::new(modem, event_loop.clone(), default_partition)?,
                net_if,
                EspNetif::new_with_conf(&NetifConfiguration {
                    ip_configuration: ipv4::Configuration::Router(RouterConfiguration {
                        subnet: Subnet {
                            gateway: ipv4addr,
                            mask: Mask(24),
                        },
                        dhcp_enabled: true,
                        dns: Some(ipv4addr),
                        secondary_dns: Some(ipv4addr),
                    }),
                    ..NetifConfiguration::wifi_default_router()
                })?,
            )?,
            event_loop,
            timer,
        )?;

        // Set the Wi-Fi configuration
        info!("Setting credentials...");
        let client_config = WifiClientConfiguration {
            ssid: WIFI_SSID.parse().unwrap(),
            password: WIFI_PASSWORD.parse().unwrap(),
            ..Default::default()
        };

        let ap_config = AccessPointConfiguration {
            ssid: "Go-Board-Settings".parse().unwrap(),

            ..Default::default()
        };

        wifi.set_configuration(&WifiConfiguration::Mixed(client_config, ap_config))?;

        info!("Starting...");
        wifi.start().await?;

        info!("Wi-Fi driver started successfully.");
        Ok(Self { state, wifi })
    }

    // Connect to Wi-Fi and stay connected. This function will loop forever.
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        loop {
            info!("Connecting to SSID '{}'...", self.state.ssid);
            if let Err(err) = self.wifi.connect().await {
                warn!("Connection failed: {err:?}");
                self.wifi.disconnect().await?;
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            info!("Acquiring IP address...");
            let timeout = Some(Duration::from_secs(10));
            if let Err(err) = self
                .wifi
                .ip_wait_while(|w| w.is_up().map(|s| !s), timeout)
                .await
            {
                warn!("IP association failed: {err:?}");
                self.wifi.disconnect().await?;
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            let ip_info = self.wifi.wifi().sta_netif().get_ip_info();
            *self.state.ip_addr.write().await = ip_info.ok().map(|i| i.ip);
            info!("Connected to '{}': {ip_info:#?}", self.state.ssid);

            // Wait for Wi-Fi to be down
            self.wifi.wifi_wait(|w| w.is_up(), None).await?;
            warn!("Wi-Fi disconnected.");
        }
    }
}

pub struct WifiLoop<'a> {
    wifi: AsyncWifi<EspWifi<'a>>,
}

impl<'a> WifiLoop<'a> {
    pub fn new(wifi: AsyncWifi<EspWifi<'a>>) -> Self {
        Self { wifi }
    }

    pub async fn configure(&mut self) -> anyhow::Result<(), EspError> {
        info!("Setting Wi-Fi credentials...");
        self.wifi
            .set_configuration(&WifiConfiguration::Client(WifiClientConfiguration {
                ssid: WIFI_SSID.parse().unwrap(),
                password: WIFI_PASSWORD.parse().unwrap(),
                ..Default::default()
            }))?;

        info!("Starting Wi-Fi driver...");
        self.wifi.start().await
    }

    pub async fn configure_ap(&mut self, ipv4addr: Ipv4Addr) -> anyhow::Result<(), EspError> {
        info!("Setting Wi-Fi credentials...");
        self.wifi.set_configuration(&WifiConfiguration::Mixed(
            WifiClientConfiguration {
                ssid: WIFI_SSID.parse().unwrap(),
                password: WIFI_PASSWORD.parse().unwrap(),
                ..Default::default()
            },
            AccessPointConfiguration {
                ssid: "Go-Board-Settings".parse().unwrap(),

                ..Default::default()
            },
        ))?;

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
            wifi.ip_wait_while(|wifi| wifi.is_up().map(|s| !s), None)
                .await?;

            if exit_after_first_connect {
                return Ok(());
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, MaxSize, Clone)]
pub struct WifiCredentials {
    pub ssid: heapless::String<32>,
    pub password: heapless::String<64>,
}

impl WifiCredentials {
    fn key() -> &'static str {
        "credentials"
    }
    fn namespace() -> &'static str {
        "wifi"
    }
    pub fn get_saved_credentials_with_default(
        partition: EspNvsPartition<NvsDefault>,
        default: Self,
    ) -> Result<Self> {
        let value = Self::get_saved_credentials(partition)?;
        match value {
            None => Ok(default),
            Some(v) => Ok(v),
        }
    }
    pub fn get_saved_credentials(partition: EspNvsPartition<NvsDefault>) -> Result<Option<Self>> {
        let mut nvs = NvsNamespace::access(partition, WifiCredentials::namespace(), false)?;

        let mut v = [0; WifiCredentials::POSTCARD_MAX_SIZE];

        let s = nvs.get_struct::<WifiCredentials>(WifiCredentials::key(), &mut v)?;
        Ok(s)
    }
    pub fn set_saved_credentials(&self, partition: EspNvsPartition<NvsDefault>) -> Result<()> {
        let mut nvs = NvsNamespace::access(partition, WifiCredentials::namespace(), false)?;

        nvs.set_struct::<WifiCredentials, { WifiCredentials::POSTCARD_MAX_SIZE }>(
            WifiCredentials::key(),
            self,
        )
    }

    pub fn get_from_env() -> Result<Self> {
        const WIFI_SSID: &'static str = env!("WIFI_SSID");
        const WIFI_PASSWORD: &'static str = env!("WIFI_PASSWORD");
        Self::new(WIFI_SSID, WIFI_PASSWORD)
    }

    pub fn new(ssid: &str, password: &str) -> Result<Self> {
        Ok(Self {
            ssid: ssid.parse().map_err(|_| anyhow!("Failed to parse ssid"))?,
            password: password
                .parse()
                .map_err(|_| anyhow!("Failed to parse password"))?,
        })
    }
}

pub fn get_sync_wifi_ap_sta<'d>(
    creds:&WifiCredentials,
    modem: Modem,
    event_loop: EspEventLoop<System>,
    default_partition: Option<EspDefaultNvsPartition>,
) -> Result<BlockingWifi<EspWifi<'d>>> {
    info!("Starting Wi-Fi...");
    let wifi_driver = WifiDriver::new(modem, event_loop.clone(), default_partition)?;
    let mut wifi = EspWifi::wrap_all(
        wifi_driver,
        EspNetif::new(NetifStack::Sta)?,
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

    wifi.set_configuration(&wifi::Configuration::Mixed(
        ClientConfiguration {
            ssid: creds.ssid.parse().unwrap(), // this should never fail,creds is heapless too
            password: creds.password.parse().unwrap(), // this should never fail, creds is heapless too
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "Go_Board_Settings".parse().unwrap(),
            ..Default::default()
        },
    ))?;
    let wifi = BlockingWifi::wrap(wifi, event_loop.clone())?;
    Ok(wifi)
}
