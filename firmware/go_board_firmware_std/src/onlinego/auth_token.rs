use crate::storage::SaveInNvs;
use postcard::experimental::max_size::MaxSize;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone)]
pub struct AuthToken {
    token: String,
    header_val: String,
}

impl AuthToken {
    pub fn new(token: String) -> Self {
        let header_val = format!("Bearer {token}");
        Self {
            token: token.as_str().into(),
            header_val,
        }
    }
    pub fn auth_header(&self) -> (&'static str, &str) {
        ("Authorization", self.header_val.as_str())
    }
}

impl Serialize for AuthToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.token)
    }
}

impl<'de> Deserialize<'de> for AuthToken {
    fn deserialize<D>(deserializer: D) -> Result<AuthToken, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(AuthTokenVisitor)
    }
}

struct AuthTokenVisitor;

impl<'de> Visitor<'de> for AuthTokenVisitor {
    type Value = AuthToken;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string that is a token (no verification)")
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(AuthToken::new(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(AuthToken::new(v))
    }
}
