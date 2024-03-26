use chrono::{DateTime, Utc};

#[derive(serde::Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub name: String,

    pub opt_enabled: bool,
    pub opt_verified: bool,
    pub opt_base32: Option<String>,
    pub opt_auth_url: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserResponse {
    pub status: String,
    pub user: UserData,
}
