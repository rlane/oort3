use crate::userid;
use serde::Deserialize;
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::fetch::{FetchService, FetchTask, Request, Response},
};

#[derive(Deserialize, Debug)]
pub struct LeaderboardData {
    lowest_time: Vec<TimeLeaderboardRow>,
    lowest_code_size: Vec<CodeSizeLeaderboardRow>,
}

#[derive(Deserialize, Debug)]
pub struct TimeLeaderboardRow {
    userid: String,
    time: String,
}

#[derive(Deserialize, Debug)]
pub struct CodeSizeLeaderboardRow {
    userid: String,
    code_size: i64,
}

#[derive(Debug)]
pub enum Msg {
    SendRequest,
    ReceiveResponse(Result<LeaderboardData, anyhow::Error>),
}

#[derive(Properties, Clone, PartialEq)]
pub struct LeaderboardProps {
    pub scenario_name: String,
}

pub struct Leaderboard {
    fetch_task: Option<FetchTask>,
    data: Option<LeaderboardData>,
    link: ComponentLink<Self>,
    error: Option<String>,
    props: LeaderboardProps,
}

impl Component for Leaderboard {
    type Message = Msg;
    type Properties = LeaderboardProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_message(Msg::SendRequest);
        Self {
            fetch_task: None,
            data: None,
            link,
            error: None,
            props,
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        props != self.props
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        use Msg::*;

        match msg {
            SendRequest => {
                let url = &format!("https://us-central1-oort-319301.cloudfunctions.net/leaderboard?scenario_name={}", &self.props.scenario_name);
                // 1. build the request
                let request = Request::get(url)
                    .body(Nothing)
                    .expect("Could not build request.");
                // 2. construct a callback
                let callback = self.link.callback(
                    |response: Response<Json<Result<LeaderboardData, anyhow::Error>>>| {
                        let Json(data) = response.into_body();
                        Msg::ReceiveResponse(data)
                    },
                );
                // 3. pass the request and callback to the fetch service
                let task = FetchService::fetch(request, callback).expect("failed to start request");
                // 4. store the task so it isn't canceled immediately
                self.fetch_task = Some(task);
                true
            }
            ReceiveResponse(response) => {
                match response {
                    Ok(data) => {
                        self.data = Some(data);
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
                self.fetch_task = None;
                true
            }
        }
    }

    fn view(&self) -> Html {
        if let Some(ref error) = self.error {
            html! { <p>{ error.clone() }</p> }
        } else if self.fetch_task.is_some() {
            html! { <p>{ "Fetching leaderboard..." }</p> }
        } else if let Some(ref data) = self.data {
            fn render_time_row(row: &TimeLeaderboardRow) -> Html {
                html! { <tr><td>{ userid::get_username(&row.userid) }</td><td>{ &row.time }</td></tr> }
            }

            fn render_code_size_row(row: &CodeSizeLeaderboardRow) -> Html {
                html! { <tr><td>{ userid::get_username(&row.userid) }</td><td>{ row.code_size }</td></tr> }
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
