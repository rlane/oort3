pub mod analyzer;

use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TelemetryMsg {
    #[serde(flatten)]
    pub payload: Telemetry,
    pub build: String,
    pub userid: String,
    pub username: String,
    #[serde(default)]
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Telemetry {
    StartScenario {
        scenario_name: String,
        code: String,
    },
    FinishScenario {
        scenario_name: String,
        code: String,
        ticks: u32,
        code_size: usize,
        success: bool,
        time: Option<f64>,
    },
    Crash {
        msg: String,
    },
    SubmitToTournament {
        scenario_name: String,
        code: String,
    },
    Feedback {
        text: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LeaderboardData {
    pub lowest_time: Vec<TimeLeaderboardRow>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TimeLeaderboardRow {
    pub userid: String,
    pub username: Option<String>,
    pub time: String,
    pub encrypted_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LeaderboardSubmission {
    pub scenario_name: String,
    pub userid: String,
    pub username: String,
    #[serde(default)]
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub time: f64,
    pub code_size: usize,
    pub code: String,
}

impl Eq for LeaderboardSubmission {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ShortcodeUpload {
    pub userid: String,
    pub username: String,
    #[serde(default)]
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub code: String,
}
