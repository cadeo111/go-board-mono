use super::auth_token::AuthToken;
use super::https::{request, RequestType};
use super::status_codes::StatusCode;
use crate::storage::SaveInNvs;
use anyhow::{anyhow, Result};
use esp_idf_svc::sys::const_format::formatcp;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

const ONLINE_GO_CLIENT_ID: &'static str = env!("ONLINE_GO_CLIENT_ID");
const ONLINE_GO_USERNAME: &'static str = env!("ONLINE_GO_USERNAME");
const ONLINE_GO_PASSWORD: &'static str = env!("ONLINE_GO_PASSWORD");

const BASE_URL: &str = "https://online-go.com";
const API_URL: &str = formatcp!("{}/api/v1/", BASE_URL);
const TERMINATION_API_URL: &str = formatcp!("{}/termination-api/", BASE_URL);
const OAUTH_URL: &str = formatcp!("{}/oauth2/", BASE_URL);

/// PASSWORD AUTH
#[derive(Serialize, Deserialize, Debug)]
pub struct OauthResponseError {
    pub error: String, // will be empty string if all went well
    pub error_description: String,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct OauthResponseErrorWithStatusCode {
    pub response: OauthResponseError,
    pub status_code: StatusCode,
}

impl Display for OauthResponseErrorWithStatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Error for OauthResponseErrorWithStatusCode {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OauthResponseValid {
    pub access_token: AuthToken, //DQTDfpBbE7pBh2E5GqwzxYSkb4AT1u
    pub expires_in: i32,
    pub token_type: String,
    pub refresh_token: String,
    // raw_scope: String, // must still be parsed, not necessary at the moment
}

#[derive(Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash, MaxSize)]
pub struct OnlineGoLoginInfo {
    pub username: heapless::String<100>,
    pub password: heapless::String<100>,
}
impl OnlineGoLoginInfo {
    pub fn auth_with_password(
        &self,
    ) -> Result<Result<OauthResponseValid, OauthResponseErrorWithStatusCode>> {
        auth_with_password(ONLINE_GO_CLIENT_ID, &self.username, &self.password)
    }
}

impl SaveInNvs for OnlineGoLoginInfo {
    fn namespace() -> &'static str {
        "og"
    }

    fn key() -> &'static str {
        "login"
    }
    fn get_struct_buffer<'a>() -> impl AsMut<[u8]> {
        [0; Self::POSTCARD_MAX_SIZE]
    }
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
) -> Result<Result<OauthResponseValid, OauthResponseErrorWithStatusCode>> {
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
                Ok(response) => Ok(Err(OauthResponseErrorWithStatusCode {
                    response,
                    status_code,
                })),
                Err(e) => Err(anyhow!(e).context(
                    "Failed to parse valid oauth json and failed to parse valid error json",
                )),
            }
        }
        Ok(valid_oauth) => Ok(Ok(valid_oauth)),
    }
}

/// END PASSWORD AUTH

/// PLAYER INFO
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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

// END PLAYER

// START GAME

/// List of games from api
#[derive(Serialize, Deserialize, Debug)]
pub struct GameList {
    #[serde(rename = "results")]
    pub games: Vec<GameListData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameListData {
    pub id: i64,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub players: PlayerList,
    /// when the game was started
    pub started: String,
    pub black_lost: bool,
    pub white_lost: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerList {
    pub white: Player,
    pub black: Player,
}

impl GameListData {
    // Description returns a formatted description of the game.
    pub fn description(&self) -> String {
        let ended = if self.is_game_over() { " (ended)" } else { "" };
        format!(
            "{} (B) vs {} (W) ({}x{}){}",
            self.players.black, self.players.white, self.width, self.height, ended
        )
    }

    // GameOver returns true if the game has ended, otherwise false.
    pub fn is_game_over(&self) -> bool {
        // If a game is over, one of these will be false
        !self.black_lost || !self.white_lost
    }

    pub fn get_detail(&self, auth_token: &AuthToken) -> Result<BoardState> {
        get_game_data(self.id, auth_token)
    }
}

pub fn get_current_player_games(auth_token: &AuthToken) -> Result<GameList> {
    const CURRENT_GAMES_URL: &str = formatcp!("{}me/games?ended__isnull=true", API_URL);

    let (status_code, value) = request(RequestType::AuthorizedGet {
        url: CURRENT_GAMES_URL,
        auth_token,
    })?;
    if status_code.is_success() {
        serde_json::from_str::<GameList>(&value).map_err(|e| {
            anyhow!(e).context(format!(
                "Failed to get current players games! ({status_code}"
            ))
        })
    } else {
        Err(anyhow!(
            "Failed to get current players games! ({status_code})"
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardState {
    pub move_number: i32,
    pub player_to_move: i64,
    pub phase: String,
    pub board: Vec<Vec<i32>>,
    pub outcome: String,
    pub removal: Vec<Vec<i32>>,
    pub last_move: LastMove,
}

pub enum BoardColor {
    Empty = 0,
    Black = 1,
    White = 2,
}

impl TryFrom<i32> for BoardColor {
    type Error = anyhow::Error;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == BoardColor::Black as i32 => Ok(BoardColor::Black),
            x if x == BoardColor::White as i32 => Ok(BoardColor::White),
            x if x == BoardColor::Empty as i32 => Ok(BoardColor::Empty),
            _ => Err(anyhow!("Failed to convert board color! {}", v)),
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct LastMove {
    x: i32,
    y: i32,
}

impl BoardState {
    pub fn finished(&self) -> bool {
        self.phase == "finished"
    }

    pub fn height(&self) -> usize {
        self.board.len()
    }

    pub fn width(&self) -> usize {
        if self.height() == 0 {
            0
        } else {
            self.board[0].len()
        }
    }

    pub fn board_iter(&self) -> impl Iterator<Item = (u8, u8, &i32)> {
        self.board
            .iter()
            .enumerate()
            .map(|(x, v)| (x, v.iter().enumerate()))
            .flat_map(|(x, iter)| iter.map(move |(y, v)| (x, y, v)))
    }
}

fn get_game_data(game_id: i64, auth_token: &AuthToken) -> Result<BoardState> {
    let url = format!("{TERMINATION_API_URL}game/{game_id}/state");

    let (status_code, value) = request(RequestType::AuthorizedGet { url, auth_token })?;

    if status_code.is_success() {
        print!("\n\n{value}\n\n");
        serde_json::from_str::<BoardState>(&value).map_err(|e| {
            anyhow!(e).context(format!(
                "Failed to get game data for {game_id}! ({status_code}"
            ))
        })
    } else {
        Err(anyhow!(
            "Failed to get game data for {game_id}! ({status_code})"
        ))
    }
}

/// END GAME

pub fn test_connection() -> Result<BoardState> {
    // let url = Url::parse_with_params("https://httpbun.org/post",
    //                                  &[("lang", "rust"), ("browser", "servo")])?;//"

    let oauth = auth_with_password(ONLINE_GO_CLIENT_ID, ONLINE_GO_USERNAME, ONLINE_GO_PASSWORD)??;
    let games = get_current_player_games(&oauth.access_token)?;
    let game_data = get_game_data(games.games[0].id, &oauth.access_token)?;
    println!("{game_data:?}");
    Ok(game_data)
    // let player = get_current_player(&s.access_token)?;
    // println!("{player}");

    // Ok(())
}
