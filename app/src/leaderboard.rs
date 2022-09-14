use crate::userid;
use reqwasm::http::Request;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Debug)]
pub struct LeaderboardData {
    lowest_time: Vec<TimeLeaderboardRow>,
    lowest_code_size: Vec<CodeSizeLeaderboardRow>,
}

#[derive(Deserialize, Debug)]
pub struct TimeLeaderboardRow {
    userid: String,
    username: Option<String>,
    time: String,
}

#[derive(Deserialize, Debug)]
pub struct CodeSizeLeaderboardRow {
    userid: String,
    username: Option<String>,
    code_size: i64,
}

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
                let url = format!("https://us-central1-oort-319301.cloudfunctions.net/leaderboard?scenario_name={}", &context.props().scenario_name);
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
            fn render_time_row(row: &TimeLeaderboardRow) -> Html {
                html! { <tr><td>{ row.username.clone().unwrap_or_else(|| userid::get_username(&row.userid)) }</td><td>{ &row.time }</td></tr> }
            }

            fn render_code_size_row(row: &CodeSizeLeaderboardRow) -> Html {
                html! { <tr><td>{ row.username.clone().unwrap_or_else(|| userid::get_username(&row.userid)) }</td><td>{ row.code_size }</td></tr> }
            }

            html! {
                <div id="leaderboards">
                    <div class="leaderboard" id="time-leaderboard">
                        <table>
                            <tr><th colspan=2>{ "Top By Time" }</th></tr>
                            <tr><th>{ "User" }</th><th>{ "Time (seconds)" }</th></tr>
                            <tbody>{ for data.lowest_time.iter().map(render_time_row) }</tbody>
                        </table>
                    </div>
                    <div class="leaderboard" id="code-size-leaderboard">
                        <table>
                            <tr><th colspan=2>{ "Top By Size" }</th></tr>
                            <tr><th>{ "User" }</th><th>{ "Size (bytes)" }</th></tr>
                            <tbody>{ for data.lowest_code_size.iter().map(render_code_size_row) }</tbody>
                        </table>
                    </div>
                </div>
            }
        } else {
            html! { <pre>{ "Unknown" }</pre> }
        }
    }
}
