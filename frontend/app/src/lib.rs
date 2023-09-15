mod analyzer_stub;
pub mod benchmark;
pub mod code_size;
pub mod codestorage;
pub mod compiler_output_window;
pub mod documentation;
pub mod editor_window;
pub mod feedback;
pub mod game;
pub mod gtag;
pub mod js;
pub mod leaderboard;
pub mod leaderboard_window;
pub mod seed_window;
pub mod services;
pub mod simulation_window;
pub mod toolbar;
pub mod tournament;
pub mod ui;
pub mod userid;
pub mod versions_window;
pub mod welcome;

use oort_version::version;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct QueryParams {
    pub seed: Option<u32>,
    pub player0: Option<String>,
    pub player1: Option<String>,
}

#[function_component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[derive(Properties, PartialEq, Eq, Debug)]
struct GameWrapperProps {
    scenario: String,
    #[prop_or_default]
    demo: bool,
}

#[function_component(GameWrapper)]
fn game_wrapper(props: &GameWrapperProps) -> Html {
    let location = use_location().expect("use_location");
    let q = query_params(&location);
    html! {
        <game::Game
            version={version()}
            scenario={props.scenario.clone()}
            seed={q.seed}
            player0={q.player0.clone()}
            player1={q.player1.clone()} />
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {
            <GameWrapper scenario="welcome" />
        },
        Route::Scenario { scenario } => html! {
            <GameWrapper scenario={scenario} />
        },
        Route::Demo { scenario } => html! {
            <GameWrapper scenario={scenario} demo=true />
        },
        Route::Benchmark { scenario } => html! {
            <benchmark::Benchmark scenario={scenario} />
        },
        Route::Tournament { id } => html! {
            <tournament::Tournament id={id} />
        },
    }
}

pub fn query_params(location: &Location) -> QueryParams {
    match location.query::<QueryParams>() {
        Ok(q) => q,
        Err(e) => {
            log::info!("Failed to parse query params: {:?}", e);
            Default::default()
        }
    }
}

fn prevent_drag_and_drop() {
    let closure = Closure::wrap(Box::new(move |e: web_sys::DragEvent| {
        e.prevent_default();
        e.stop_propagation();
    }) as Box<dyn FnMut(_)>);
    for event in &["dragover", "drop"] {
        gloo_utils::document()
            .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
            .unwrap();
    }
    closure.forget();
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
    prevent_drag_and_drop();
    yew::Renderer::<Main>::with_root(
        gloo_utils::document()
            .get_element_by_id("yew")
            .expect("a #yew element"),
    )
    .render();
    Ok(())
}
