pub mod api;
pub mod game;
pub mod renderer;
pub mod script;
pub mod simulation;
pub mod ui;
pub mod worker_api;

#[macro_use]
extern crate memoffset;

use chrono::NaiveDateTime;
use rbtag::{BuildDateTime, BuildInfo};

#[derive(BuildDateTime, BuildInfo)]
struct BuildTag;

pub fn version() -> String {
    let build_time = NaiveDateTime::from_timestamp(
        BuildTag {}
            .get_build_timestamp()
            .parse::<i64>()
            .unwrap_or(0),
        0,
    );

    let commit = BuildTag {}.get_build_commit();

    if commit.contains("dirty") {
        commit.to_string()
    } else {
        format!("{} {}", build_time.format("%Y%m%d.%H%M%S"), commit)
    }
}
