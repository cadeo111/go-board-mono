use super::auth_token::AuthToken;
use crate::onlinego::status_codes::StatusCode;
use anyhow::{anyhow, Result};
use embedded_svc::http::client::Client;
use embedded_svc::http::{Headers, Method};
use embedded_svc::io::Read;
use esp_idf_svc::http::client::{Configuration as HttpConfiguration, EspHttpConnection};
use esp_idf_svc::io::Write;
use std::str;

pub enum RequestType<'at, S>
where
    S: AsRef<str>,
{
    Get {
        url: S,
    },
    AuthorizedGet {
        url: S,
        auth_token: &'at AuthToken,
    },
    Post {
        url: S,
        data: S,
    },
    AuthorizedPost {
        url: S,
        data: S,
        auth_token: &'at AuthToken,
    },
}

pub fn request(request_type: RequestType<impl AsRef<str>>) -> Result<(StatusCode, String)> {
    const POST_CONTENT_URL_ENCODED: (&str, &str) =
        ("Content-Type", "application/x-www-form-urlencoded");
    // const ACCEPT_CONTENT_HEADER_JSON: (&str, &str) = ("Accept", "application/json");

    // 1. Create a new EspHttpClient.
    let connection = EspHttpConnection::new(&HttpConfiguration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    // wrap with Http client for HTTP methods
    let mut client = Client::wrap(connection);

    // 2. Open a request to `url`
    let url_ref;
    let headers_1;
    let headers_2;

    let request = {
        match request_type {
            RequestType::Get { url } => {
                url_ref = url;
                client.request(Method::Get, url_ref.as_ref(), &[])?
            }
            RequestType::AuthorizedGet { url, auth_token } => {
                url_ref = url;
                headers_1 = [auth_token.auth_header()];
                client.request(Method::Get, url_ref.as_ref(), &headers_1)?
            }
            RequestType::Post { url, data } => {
                url_ref = url;
                let mut request =
                    client.request(Method::Post, url_ref.as_ref(), &[POST_CONTENT_URL_ENCODED])?;
                write!(request, "{}", data.as_ref())?;
                request
            }
            RequestType::AuthorizedPost {
                url,
                auth_token,
                data,
            } => {
                url_ref = url;
                headers_2 = [POST_CONTENT_URL_ENCODED, auth_token.auth_header()];
                let mut request = client.request(Method::Post, url_ref.as_ref(), &headers_2)?;
                write!(request, "{}", data.as_ref())?;
                request
            }
        }
    };

    // 3. Submit write request and check the status code of the response.
    // Successful http status codes are in the 200..=299 range.
    let response = request.submit()?;
    let status = StatusCode::from_u16(response.status())?;

    println!("Response code: {}\n", status);
    // match status {
    // 200..=299 => {
    // 4. if the status is OK, read response data chunk by chunk into a buffer and print it until done
    //
    // NB. see http_client.rs for an explanation of the offset mechanism for handling chunks that are
    // split in the middle of valid UTF-8 sequences. This case is encountered a lot with the given
    // example URL.
    let mut buf = [0_u8; 256];
    let mut offset = 0;
    let mut total = 0;
    let mut response_str = if let Some(len) = response.content_len() {
        String::with_capacity(len as usize)
    } else {
        String::with_capacity(256)
    };
    let mut reader = response;
    loop {
        if let Ok(size) = Read::read(&mut reader, &mut buf[offset..]) {
            if size == 0 {
                break;
            }
            total += size;
            // 5. try converting the bytes into a Rust (UTF-8) string and print it
            let size_plus_offset = size + offset;
            match str::from_utf8(&buf[..size_plus_offset]) {
                Ok(text) => {
                    response_str.push_str(text);
                    offset = 0;
                }
                Err(error) => {
                    let valid_up_to = error.valid_up_to();
                    unsafe {
                        print!("{}", str::from_utf8_unchecked(&buf[..valid_up_to]));
                    }
                    buf.copy_within(valid_up_to.., 0);
                    offset = size_plus_offset - valid_up_to;
                    return Err(anyhow!(error).context("Failed to parse response"));
                }
            }
        }
    }
    println!("Total: {} bytes", total);
    Ok((status, response_str))
}

/*pub fn post(url: impl AsRef<str>, json: impl AsRef<str>) -> Result<String> {
    // 1. Create a new EspHttpClient. (Check documentation)
    // ANCHOR: connection
    let connection = EspHttpConnection::new(&HttpConfiguration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    // ANCHOR_END: connection
    let mut client = Client::wrap(connection);

    // 2. Open a GET request to `url`
    let headers = [("accept", "text/plain")];
    let mut request = client.request(Method::Post, url.as_ref(), &headers)?;
    write!(request, "{}", json.as_ref())?;
    // 3. Submit write request and check the status code of the response.
    // Successful http status codes are in the 200..=299 range.
    let response = request.submit()?;
    let status = response.status();
    println!("Response code: {}\n", status);
    match status {
        200..=299 => {
            // 4. if the status is OK, read response data chunk by chunk into a buffer and print it until done
            //
            // NB. see http_client.rs for an explanation of the offset mechanism for handling chunks that are
            // split in the middle of valid UTF-8 sequences. This case is encountered a lot with the given
            // example URL.
            let mut buf = [0_u8; 256];
            let mut offset = 0;
            let mut total = 0;
            let mut response_str = if let Some(len) = response.content_len() {
                String::with_capacity(len as usize)
            } else {
                String::with_capacity(256)
            };
            let mut reader = response;
            loop {
                if let Ok(size) = Read::read(&mut reader, &mut buf[offset..]) {
                    if size == 0 {
                        break;
                    }
                    total += size;
                    // 5. try converting the bytes into a Rust (UTF-8) string and print it
                    let size_plus_offset = size + offset;
                    match str::from_utf8(&buf[..size_plus_offset]) {
                        Ok(text) => {
                            response_str.push_str(text);
                            offset = 0;
                        }
                        Err(error) => {
                            let valid_up_to = error.valid_up_to();
                            unsafe {
                                print!("{}", str::from_utf8_unchecked(&buf[..valid_up_to]));
                            }
                            return Err(anyhow!("Failed to parse response"));
                            buf.copy_within(valid_up_to.., 0);
                            offset = size_plus_offset - valid_up_to;
                        }
                    }
                }
            }
            println!("Total: {} bytes", total);
            Ok(response_str)
        }
        _ => bail!("Unexpected response code: {}", status),
    }
}*/
