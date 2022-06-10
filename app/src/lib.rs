pub mod code_size;
pub mod codestorage;
pub mod documentation;
pub mod game;
pub mod js;
pub mod leaderboard;
pub mod telemetry;
pub mod ui;
pub mod userid;

use chrono::NaiveDateTime;
use rbtag::{BuildDateTime, BuildInfo};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::prelude::*;

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

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/scenario/:name")]
    Scenario { name: String },
}

#[function_component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => switch(&Route::Scenario {
            name: "welcome".to_owned(),
        }),
        Route::Scenario { name } => html! {
            <game::Game scenario={name.clone()} />
        },
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    log::info!("Version {}", &version());
    let userid = userid::get_userid();
    log::info!("userid {}", &userid);
    log::info!("username {}", &userid::get_username(&userid));
    yew::start_app::<Main>();
    Ok(())
}
