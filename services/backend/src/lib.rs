pub mod discord;
pub mod leaderboard;
pub mod shortcode;
pub mod telemetry;
pub mod tournament;

pub fn project_id() -> &'static str {
    match std::env::var("ENVIRONMENT") {
        Ok(x) if x == "dev" => "oort-dev",
        Ok(x) if x == "prod" => "oort-319301",
        _ => {
            panic!("Invalid ENVIRONMENT")
        }
    }
}
