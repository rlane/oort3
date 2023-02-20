use crate::services;
use crate::userid;
use oort_proto::LeaderboardSubmission;
use oort_proto::{LeaderboardData, TimeLeaderboardRow};
use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {
    SendRequest,
    ReceiveResponse(Result<LeaderboardData, anyhow::Error>),
}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct LeaderboardProps {
    pub scenario_name: String,
    pub submission: Option<LeaderboardSubmission>,
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
                let callback =
                    context
                        .link()
                        .callback(|response: Result<LeaderboardData, anyhow::Error>| {
                            Msg::ReceiveResponse(response)
                        });
                if let Some(submission) = context.props().submission.as_ref() {
                    services::post_leaderboard(submission.clone(), callback);
                } else {
                    services::get_leaderboard(&context.props().scenario_name, callback);
                }
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
                let class = (row.userid == userid).then_some("own-leaderboard-entry");
                let copy_encrypted_code_cb = {
                    let text = row.encrypted_code.clone();
                    move |_| {
                        crate::js::clipboard::write(&text);
                    }
                };
                html! {
                    <tr class={classes!(class)}>
                        <td>{ row.username.clone().unwrap_or_else(|| userid::generate_username(&row.userid)) }</td>
                        <td>{ &row.time }</td>
                        <td><a class="material-symbols-outlined" onclick={copy_encrypted_code_cb}>{ "content_copy" }</a></td>
                    </tr>
                }
            };

            html! {
                <div class="leaderboard">
                    <table>
                        <tr><th colspan=3>{ "Leaderboard" }</th></tr>
                        <tr><th>{ "User" }</th><th>{ "Time" }</th><th>{ "Encrypted Code" }</th></tr>
                        <tbody>{ for data.lowest_time.iter().map(render_time_row) }</tbody>
                    </table>
                </div>
            }
        } else {
            html! { <pre>{ "Unknown" }</pre> }
        }
    }

    fn changed(&mut self, context: &Context<Self>, old_props: &LeaderboardProps) -> bool {
        if old_props.scenario_name != context.props().scenario_name {
            self.data = None;
            self.error = None;
            context.link().send_message(Msg::SendRequest);
            true
        } else {
            false
        }
    }
}
