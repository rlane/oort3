pub mod discord;
pub mod leaderboard;
pub mod shortcode;
pub mod telemetry;
pub mod tournament;

pub fn project_id() -> String {
    std::env::var("PROJECT_ID").expect("missing PROJECT_ID environment variable")
}
