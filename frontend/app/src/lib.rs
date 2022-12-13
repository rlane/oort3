pub mod benchmark;
pub mod code_size;
pub mod codestorage;
pub mod compiler_output_window;
pub mod documentation;
pub mod editor_window;
pub mod feedback;
pub mod game;
pub mod js;
pub mod leaderboard;
pub mod services;
pub mod simulation_window;
pub mod toolbar;
pub mod ui;
pub mod userid;
pub mod welcome;

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

#[derive(Clone, Routable, PartialEq, Eq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/scenario/:scenario")]
    Scenario { scenario: String },
    #[at("/demo/:scenario")]
    Demo { scenario: String },
    #[at("/benchmark/:scenario")]
    Benchmark { scenario: String },
}

#[function_component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {
            <game::Game scenario="welcome" version={version()} />
        },
        Route::Scenario { scenario } => html! {
            <game::Game scenario={scenario} version={version()} />
        },
        Route::Demo { scenario } => html! {
            <game::Game scenario={scenario} demo=true version={version()} />
        },
        Route::Benchmark { scenario } => html! {
            <benchmark::Benchmark scenario={scenario} />
        },
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    log::info!("Version {}", &version());
    let userid = userid::get_userid();
    log::info!("userid {}", &userid);
    log::info!("username {}", &userid::get_username());
    js::golden_layout::init();
    js::completion::init();
    yew::Renderer::<Main>::with_root(
        gloo_utils::document()
            .get_element_by_id("yew")
            .expect("a #yew element"),
    )
    .render();
    Ok(())
}
