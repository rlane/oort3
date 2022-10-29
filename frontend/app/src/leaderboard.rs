use crate::services;
use crate::userid;
use oort_proto::{LeaderboardData, TimeLeaderboardRow};
use reqwasm::http::Request;
use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {
    SendRequest,
    ReceiveResponse(Result<LeaderboardData, reqwasm::Error>),
}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct LeaderboardProps {
    pub scenario_name: String,
}

pub struct Leaderboard {
    data: Option<LeaderboardData>,
    error: Option<String>,
    fetching: bool,
}

impl Component for Leaderboard {
    type Message = Msg;
    type Properties = LeaderboardProps;

    fn create(context: &yew::Context<Self>) -> Self {
        context.link().send_message(Msg::SendRequest);
        Self {
            data: None,
            error: None,
            fetching: false,
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        use Msg::*;

        match msg {
            SendRequest => {
                let url = format!(
                    "{}/leaderboard?scenario_name={}",
                    services::leaderboard_url(),
                    &context.props().scenario_name
                );
                let callback =
                    context
                        .link()
                        .callback(|response: Result<LeaderboardData, reqwasm::Error>| {
                            Msg::ReceiveResponse(response)
                        });
                wasm_bindgen_futures::spawn_local(async move {
                    let result = Request::get(&url).send().await.unwrap().json().await;
                    callback.emit(result);
                });
                self.fetching = true;
                true
            }
            ReceiveResponse(response) => {
                match response {
                    Ok(data) => {
                        self.data = Some(data);
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
                self.fetching = false;
                true
            }
        }
    }

    fn view(&self, _context: &yew::Context<Self>) -> Html {
        if let Some(ref error) = self.error {
            html! { <p>{ error.clone() }</p> }
        } else if self.fetching {
            html! { <p>{ "Fetching leaderboard..." }</p> }
        } else if let Some(ref data) = self.data {
            let userid = userid::get_userid();
            let render_time_row = |row: &TimeLeaderboardRow| -> Html {
                let class = (row.userid == userid).then(|| "own-leaderboard-entry");
                html! { <tr class={classes!(class)}><td>{ row.username.clone().unwrap_or_else(|| userid::generate_username(&row.userid)) }</td><td>{ &row.time }</td></tr> }
            };

            html! {
                <div class="leaderboard">
                    <table>
                        <tr><th colspan=2>{ "Leaderboard" }</th></tr>
                        <tr><th>{ "User" }</th><th>{ "Time" }</th></tr>
                        <tbody>{ for data.lowest_time.iter().map(render_time_row) }</tbody>
                    </table>
                </div>
            }
        } else {
            html! { <pre>{ "Unknown" }</pre> }
        }
    }
}
