pub mod benchmark;
pub mod code_size;
pub mod codestorage;
pub mod documentation;
pub mod filesystem;
pub mod game;
pub mod js;
pub mod leaderboard;
pub mod telemetry;
pub mod ui;
pub mod userid;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::prelude::*;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn version() -> String {
    let mut fragments = vec![built_info::GIT_VERSION.unwrap_or("unknown")];
    if built_info::GIT_DIRTY == Some(true) {
        fragments.push("dirty");
    }
    fragments.join("-")
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/scenario/:name")]
    Scenario { name: String },
    #[at("/demo/:name")]
    Demo { name: String },
    #[at("/benchmark/:name")]
    Benchmark { name: String },
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
            <game::Game scenario={name.clone()} demo=false version={version()} />
        },
        Route::Demo { name } => html! {
            <game::Game scenario={name.clone()} demo=true version={version()} />
        },
        Route::Benchmark { name } => html! {
            <benchmark::Benchmark scenario={name.clone()} />
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
