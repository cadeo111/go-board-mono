use std::io::{ErrorKind, Read};
use anyhow::{anyhow, Error};
use embedded_svc::http::server::Request;
use esp_idf_svc::http::server::EspHttpConnection;
use esp_idf_svc::io::{EspIOError, Write};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

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

// todo: Explore  embedded_svc::http::server::asynch::Request
/// an error would basically be unexpected failure, if it is an expected failure then a DataRequestResponse will be sent
pub fn deserialize_json_from_request<'a: 'a, 'b: 'b, T>(
    request: Request<&'a mut EspHttpConnection<'b>>,
) -> anyhow::Result<Option<(Request<&'a mut EspHttpConnection<'b>>, T)>>
where
    T: DeserializeOwned,
{
    // TODO: Handle error if incorrect data sent in request
    let (v, data) = {
        let mut rr = ReadableRequest(request);
        let data: anyhow::Result<T> = serde_json::from_reader(&mut rr).map_err(|e| anyhow!(e));
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
