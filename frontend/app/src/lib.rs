mod analyzer_stub;
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
pub mod leaderboard_window;
pub mod services;
pub mod simulation_window;
pub mod toolbar;
pub mod tournament;
pub mod ui;
pub mod userid;
pub mod versions_window;
pub mod welcome;

use oort_version::version;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::prelude::*;

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
    #[at("/tournament/:id")]
    Tournament { id: String },
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
        Route::Tournament { id } => html! {
            <tournament::Tournament id={id} />
        },
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    log::info!("Version {}", &version());
    let userid = userid::get_userid();
    log::info!("userid {}", &userid);
    log::info!("username {}", &userid::get_username());
    log::info!(
        "hashed envelope secret: {:?}",
        &oort_envelope::hashed_secret()
    );
    js::completion::init();
    yew::Renderer::<Main>::with_root(
        gloo_utils::document()
            .get_element_by_id("yew")
            .expect("a #yew element"),
    )
    .render();
    Ok(())
}
