use crate::onlinego::status_codes::StatusCode;
use crate::settings::server::server::DataResponse;
use anyhow::{anyhow, Error};
use embedded_svc::http::server::Request;
use esp_idf_svc::http::server::EspHttpConnection;
use esp_idf_svc::io::{EspIOError, Write};
use log::warn;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{ErrorKind, Read};

struct ReadableRequest<'a, 'r, 'c>(&'a mut Request<&'r mut EspHttpConnection<'c>>);

// impl<'r, 'c> ReadableRequest<'r, 'c> {
//     fn into_request(self) -> Request<&'r mut EspHttpConnection<'c>> {
//         self.0
//     }
// }

impl<'a, 'r, 'c> Read for &mut ReadableRequest<'a, 'r, 'c> {
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

// Result<
//         (Request<&'r mut EspHttpConnection<'r>>, DataResponse),
//         CaptiveServerHandlerError<'r>,
//     >

pub enum DataResponseOrValue<T> {
    Response(DataResponse),
    Value(T),
}

// todo: Explore  embedded_svc::http::server::asynch::Request
/// an error would basically be unexpected failure, if it is an expected failure then a DataRequestResponse will be sent
pub fn deserialize_json_from_request<'a: 'a, 'b: 'b, 'c:'c, T>(
    request: &'c mut Request<&'a mut EspHttpConnection<'b>>,
) -> (DataResponseOrValue<T>)
where
    T: DeserializeOwned,
{
    // TODO: Handle error if incorrect data sent in request
    let (data) = {
        let mut rr = ReadableRequest(request);
        let data: anyhow::Result<T> = serde_json::from_reader(&mut rr).map_err(|e| anyhow!(e));
        (data)
    };
    match data {
        Ok(data) => DataResponseOrValue::Value(data),
        Err(_) => {
            let warning = format!(
                "incorrect parameters sent with request!\n for type: {}",
                std::any::type_name::<T>()
            );
            warn!("{warning}");

            DataResponseOrValue::Response(DataResponse::HandledErr(
                StatusCode::BAD_REQUEST,
                json!(warning),
            ))
        }
    }

    // // TODO: Handle error if incorrect data sent in request
}
