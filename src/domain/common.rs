use super::errors::Error;
use serde::{Deserialize, Serialize};

pub type Period = (u64, u64);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct DateRange {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

impl DateRange {
    /// returns true if any field is none.
    pub fn is_empty(&self) -> bool {
        self.start.is_none() || self.end.is_none()
    }

    pub fn to_period(&self) -> Period {
        (self.start.unwrap_or(0), self.end.unwrap_or(0))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Secret {
    pub account: String,
    pub password: String,
}

impl Secret {
    pub fn fake() -> Self {
        let password = format!("{:x}", md5::compute("fake".as_bytes()));

        Self {
            account: "fake".to_string(),
            password,
        }
    }

    pub fn new(account: String, password: String) -> std::result::Result<Self, Error> {
        if password.is_empty() {
            return Err(Error::LogicError("密码不能为空".to_string()));
        }

        let password = format!("{:x}", md5::compute(password.as_bytes()));

        Ok(Self { account, password })
    }

    /// change password to param
    pub fn change_password(&mut self, password: String) {
        self.password = format!("{:x}", md5::compute(password.as_bytes()));
    }

    /// returns a boolean indicating whether the password is matched.
    pub fn is_match(&self, password: &str) -> bool {
        format!("{:x}", md5::compute(password.as_bytes())) == self.password
    }
}
