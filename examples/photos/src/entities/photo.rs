use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpsCoordinates {
    pub lat: f64,
    pub lng: f64,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Photo {
    pub id: String,
    pub user_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub taken_at: DateTime<Utc>,
    pub uploaded_at: DateTime<Utc>,
    pub rating: Option<u8>,
    pub labels: Vec<String>,
    pub album_ids: Vec<String>,
    pub gps: Option<GpsCoordinates>,
    pub thumbnail_url: String,
    pub preview_url: String,
}
