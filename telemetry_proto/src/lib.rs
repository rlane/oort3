use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TelemetryMsg {
    #[serde(flatten)]
    pub payload: Telemetry,
    pub build: String,
    pub userid: String,
    pub username: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        success: Option<bool>,
    },
    Crash {
        msg: String,
    },
    SubmitToTournament {
        scenario_name: String,
        code: String,
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
}
