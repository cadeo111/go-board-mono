use super::auth_token::AuthToken;
use super::https::{request, RequestType};
use super::status_codes::StatusCode;
use anyhow::{anyhow, Result};
use esp_idf_svc::sys::const_format::formatcp;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

const ONLINE_GO_CLIENT_ID: &'static str = env!("ONLINE_GO_CLIENT_ID");
const ONLINE_GO_USERNAME: &'static str = env!("ONLINE_GO_USERNAME");
const ONLINE_GO_PASSWORD: &'static str = env!("ONLINE_GO_PASSWORD");

const BASE_URL: &str = "https://online-go.com";
const API_URL: &str = formatcp!("{}/api/v1/", BASE_URL);
const TERMINATION_API_URL: &str = formatcp!("{}/termination-api/", BASE_URL);
const OAUTH_URL: &str = formatcp!("{}/oauth2/", BASE_URL);

/// PASSWORD AUTH
#[derive(Serialize, Deserialize, Debug)]
struct OauthResponseError {
    error: String, // will be empty string if all went well
    error_description: String,
}
impl OauthResponseError {
    fn to_anyhow(&self, status: StatusCode) -> anyhow::Error {
        let OauthResponseError {
            error,
            error_description,
        } = self;
        anyhow!("{error} -> {error_description} ({status})")
    }
}

#[derive(Deserialize, Debug)]
struct OauthResponseValid {
    access_token: AuthToken,
    expires_in: i32,
    token_type: String,
    refresh_token: String,
    // raw_scope: String, // must still be parsed, not necessary at the moment
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthPasswordData<'s> {
    client_id: &'s str,
    username: &'s str,
    grant_type: &'static str,
    password: &'s str,
}
impl<'s> AuthPasswordData<'s> {
    fn qs(client_id: &'s str, username: &'s str, password: &'s str) -> anyhow::Result<String> {
        let obj = Self {
            client_id,
            username,
            grant_type: "password",
            password,
        };
        serde_qs::to_string(&obj).map_err(|e| anyhow!(e))
    }
}

fn auth_with_password(
    client_id: impl AsRef<str>,
    username: impl AsRef<str>,
    password: impl AsRef<str>,
) -> Result<OauthResponseValid> {
    const REFRESH_TOKEN_URL: &str = formatcp!("{}token/", OAUTH_URL);

    let (status_code, s) = request(RequestType::Post {
        url: REFRESH_TOKEN_URL,
        data: AuthPasswordData::qs(client_id.as_ref(), username.as_ref(), password.as_ref())?
            .as_ref(),
    })?;

    let possible_valid_oauth = serde_json::from_str::<OauthResponseValid>(&s);

    match possible_valid_oauth {
        Err(_) => {
            let possible_error = serde_json::from_str::<OauthResponseError>(&s);
            match possible_error {
                Ok(data) => Err(data.to_anyhow(status_code)),
                Err(e) => Err(anyhow!(e).context(
                    "Failed to parse valid oauth json and failed to parse valid error json",
                )),
            }
        }
        Ok(valid_oauth) => Ok(valid_oauth),
    }
}

/// END PASSWORD AUTH

/// PLAYER INFO
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Player {
    id: i64,
    username: String,
    #[serde(rename = "ranking")]
    raw_ranking: f32,
}

impl Player {
    pub fn ranking(&self) -> String {
        if self.raw_ranking < 30.0 {
            format!("{} kyu", (30.0 - self.raw_ranking + 0.5).round() as i32)
        } else {
            format!(
                "{} dan",
                ((self.raw_ranking - 30.0 + 0.5).round() + 1.0) as i32
            )
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ({})", self.username, self.ranking())
    }
}

pub fn get_current_player(auth_token: &AuthToken) -> Result<Player> {
    const CURRENT_PLAYER_URL: &str = formatcp!("{}me", API_URL);
    let (status_code, value) = request(RequestType::AuthorizedGet {
        url: CURRENT_PLAYER_URL,
        auth_token,
    })?;

    if status_code.is_success() {
        serde_json::from_str::<Player>(&value).map_err(|e| {
            anyhow!(e).context(format!("Failed to get current player! ({status_code}"))
        })
    } else {
        Err(anyhow!("Failed to get current player! ({status_code})"))
    }
}

/// END PLAYER

pub fn test_connection() -> Result<()> {
    // let url = Url::parse_with_params("https://httpbun.org/post",
    //                                  &[("lang", "rust"), ("browser", "servo")])?;//"

    let s = auth_with_password(ONLINE_GO_CLIENT_ID, ONLINE_GO_USERNAME, ONLINE_GO_PASSWORD)?;
    println!("{s:?}");

    Ok(())
}
