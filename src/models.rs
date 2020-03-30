use chrono::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct GraphDot {
    pub when: DateTime<Local>,
    pub count: u32,
    pub ratio: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StateReport {
    pub state: String,
    pub positive: Option<u32>,
    pub positive_core: Option<u32>,
    pub negative_core: Option<u32>,
    pub negative_regular_core: Option<u32>,
    pub commercial_core: Option<u32>,
    pub grade: Option<char>,	
    pub score: Option<u32>,
    pub negative: Option<u32>,
    pub pending: Option<u32>,
    pub hospitalized: Option<u32>,
    pub death: Option<u32>,
    pub total: Option<u32>,
    pub last_update_et: String,
    pub check_time_et: String,
    pub date_modified: DateTime<Utc>,
    pub date_checked: DateTime<Utc>,
}

impl StateReport {
    pub fn folder(&self) -> String {
        let dt = self.date_checked.naive_local();
        format!("{:04}.{:02}.{:02}", dt.year(), dt.month(), dt.day())
    }
    pub fn file_name(&self) -> String {
        let dt: DateTime<Local> = self.date_checked.into();
        format!("{:02}:{:02}:{:02}.toml", dt.hour(), dt.minute(), dt.second())
    }
    pub fn modified_local(&self) -> DateTime<Local> {
        self.date_checked.into()
    }
    pub fn yesterday_folder(&self) -> String {
        let dt: DateTime<Local> = self.date_checked.into();
        let y = dt - chrono::Duration::days(1);
        format!("{}.{:02}.{02}", y.year(), y.month(), y.day())
    }
}