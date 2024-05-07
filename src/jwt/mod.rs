use std::{collections::BTreeMap, ops::Add};

use chrono::{Duration, Utc};
use hmac::{digest::InvalidLength, Hmac, Mac};
use jwt::{Claims, RegisteredClaims, SignWithKey, VerifyWithKey};
use serde_json::Value;
use sha2::Sha256;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("jwt error: {0}")]
    Jwt(#[from] jwt::Error),

    #[error("hmac error: {0}")]
    Hmac(#[from] InvalidLength),

    #[error("token creation failed")]
    TokenCreationFailed,
}

#[derive(Debug, Clone)]
pub struct Engine {
    key: Hmac<Sha256>,
}

pub struct TokenPayload {
    pub id: String,
    pub account: String,
    pub role: String,
}

impl TokenPayload {
    pub fn new(id: String, account: String, role: String) -> Self {
        Self { id, account, role }
    }
}

impl Into<BTreeMap<String, Value>> for TokenPayload {
    fn into(self) -> BTreeMap<String, Value> {
        let mut out: BTreeMap<String, Value> = BTreeMap::new();
        out.insert("id".to_string(), self.id.into());
        out.insert("account".to_string(), self.account.into());
        out.insert("role".to_string(), self.role.into());

        out
    }
}

impl From<BTreeMap<String, Value>> for TokenPayload {
    fn from(payload_map: BTreeMap<String, Value>) -> Self {
        let id = match payload_map.get("id") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };

        let account = match payload_map.get("account") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };

        let role = match payload_map.get("role") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };

        TokenPayload { id, account, role }
    }
}

impl Engine {
    pub fn new(secret: String) -> Result<Self, Error> {
        let out = Self {
            key: Hmac::new_from_slice(secret.as_bytes())?,
        };

        Ok(out)
    }

    /// create a token str from a user id
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    /// * the token can not be created (sign failed)
    pub fn create_token(&self, payload: TokenPayload) -> Result<String, Error> {
        let expiration = Utc::now().add(Duration::days(30)).timestamp();

        let mut claims = Claims::new(RegisteredClaims {
            subject: Some(payload.id.clone()),
            expiration: Some(expiration as u64),
            ..Default::default()
        });

        claims.private = payload.into();

        let token = claims.sign_with_key(&self.key)?;

        Ok(token)
    }

    /// returns a user id if the token is valid
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    /// * the token is invalid
    pub fn verify_token(&self, token: &str) -> Result<TokenPayload, Error> {
        let claims: Claims = token.verify_with_key(&self.key)?;
        let uid = claims
            .registered
            .subject
            .ok_or(Error::TokenCreationFailed)?;

        Ok(TokenPayload::from(claims.private))
    }
}
