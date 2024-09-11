// pub fn main() {
//     #[cfg(esp_idf_version_major = "4")]
//     example::main().unwrap();
//
//     // Note that the ESP IDF websocket client IS available on ESP IDF >= 5 too
//     // It is just that it is now an external component, so to use it, you need
//     // to put the following snippet at the end of the `Cargo.toml` file of your binary crate:
//     //
//     // ```toml
//     // [[package.metadata.esp-idf-sys.extra_components]]
//     // remote_component = { name = "espressif/esp_websocket_client", version = "1.1.0" }
//     // ```
//
// }

use core::time::Duration;

use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::tls::X509;
use esp_idf_svc::wifi::*;
use esp_idf_svc::ws::client::{
    EspWebSocketClient, EspWebSocketClientConfig, FrameType, WebSocketEvent, WebSocketEventType,
};
use log::info;

use std::sync::mpsc;

const ECHO_SERVER_URI: &str = "wss://echo.websocket.org";

const WS_URL: &str = "wss://online-go.com/socket.io/?EIO=3&transport=websocket";


/// The relevant events for this example as it connects to the server,
/// sends a message, receives the same message, and closes the connection.
#[derive(Debug, PartialEq)]
enum ExampleEvent {
    Connected,
    MessageReceived,
    Closed,
}

pub fn test() -> Result<()> {
    // Connect websocket
    let config = EspWebSocketClientConfig {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    };
    let timeout = Duration::from_secs(10);
    let (tx, rx) = mpsc::channel::<ExampleEvent>();
    let mut client = EspWebSocketClient::new(ECHO_SERVER_URI, &config, timeout, move |event| {
        handle_event_test(&tx, event)
    })?;
    assert_eq!(rx.recv(), Ok(ExampleEvent::Connected));
    assert!(client.is_connected());

    // Send message and receive it back
    let message = "Hello, World!";
    info!("Websocket send, text: {}", message);
    client.send(FrameType::Text(false), message.as_bytes())?;
    assert_eq!(rx.recv(), Ok(ExampleEvent::MessageReceived));

    // Close websocket
    drop(client);
    assert_eq!(rx.recv(), Ok(ExampleEvent::Closed));

    Ok(())
}

fn handle_event_test(tx: &mpsc::Sender<ExampleEvent>, event: &Result<WebSocketEvent, EspIOError>) {
    if let Ok(event) = event {
        match event.event_type {
            WebSocketEventType::BeforeConnect => {
                info!("Websocket before connect");
            }
            WebSocketEventType::Connected => {
                info!("Websocket connected");
                tx.send(ExampleEvent::Connected).ok();
            }
            WebSocketEventType::Disconnected => {
                info!("Websocket disconnected");
            }
            WebSocketEventType::Close(reason) => {
                info!("Websocket close, reason: {reason:?}");
            }
            WebSocketEventType::Closed => {
                info!("Websocket closed");
                tx.send(ExampleEvent::Closed).ok();
            }
            WebSocketEventType::Text(text) => {
                info!("Websocket recv, text: {text}");
                if text == "Hello, World!" {
                    tx.send(ExampleEvent::MessageReceived).ok();
                }
            }
            WebSocketEventType::Binary(binary) => {
                info!("Websocket recv, binary: {binary:?}");
            }
            WebSocketEventType::Ping => {
                info!("Websocket ping");
            }
            WebSocketEventType::Pong => {
                info!("Websocket pong");
            }
        }
    }
}

pub fn setup() -> Result<()> {
    let config = EspWebSocketClientConfig {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    };
    let timeout = Duration::from_secs(10);
    let (tx, rx) = mpsc::channel::<ExampleEvent>();
    let mut client = EspWebSocketClient::new(WS_URL, &config, timeout, move |event| {
        if let Ok(event) = event {
            match event.event_type {
                WebSocketEventType::BeforeConnect => {
                    info!("Websocket before connect");
                }
                WebSocketEventType::Connected => {
                    info!("Websocket connected");
                    tx.send(ExampleEvent::Connected).ok();
                }
                WebSocketEventType::Disconnected => {
                    info!("Websocket disconnected");
                }
                WebSocketEventType::Close(reason) => {
                    info!("Websocket close, reason: {reason:?}");
                }
                WebSocketEventType::Closed => {
                    info!("Websocket closed");
                    tx.send(ExampleEvent::Closed).ok();
                }
                WebSocketEventType::Text(text) => {
                    info!("Websocket recv, text: {text}");
                    if text == "Hello, World!" {
                        tx.send(ExampleEvent::MessageReceived).ok();
                    }
                }
                WebSocketEventType::Binary(binary) => {
                    info!("Websocket recv, binary: {binary:?}");
                }
                WebSocketEventType::Ping => {
                    info!("Websocket ping");
                }
                WebSocketEventType::Pong => {
                    info!("Websocket pong");
                }
            }
        }
    })?;

    Ok(())
}
