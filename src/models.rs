use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Option<String>,
    pub email: String,
    pub name: String,
    pub password: String,

    pub opt_enabled: Option<bool>,
    pub opt_verified: Option<bool>,
    pub opt_base32: Option<String>,
    pub opt_auth_url: Option<String>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct AppState {
    pub db: Arc<Mutex<Vec<User>>>,
}

impl AppState {
    pub fn init() -> AppState {
        AppState {
            db: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct UserRegisterSchema {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct UserLoginSchema {
    pub email: String,
    pub password: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct GenerateOPTSchema {
    pub email: String,
    pub user_id: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct VerifyOPTSschema {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct DisableOPTSchema {
    pub user_id: String,
}
