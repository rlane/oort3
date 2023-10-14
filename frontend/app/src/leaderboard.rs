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

#[derive(Properties, Clone, PartialEq)]
pub struct LeaderboardProps {
    pub scenario_name: String,
    pub submission: Option<LeaderboardSubmission>,
    pub play_cb: Callback<String>,
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

    fn view(&self, context: &yew::Context<Self>) -> Html {
        if let Some(ref error) = self.error {
            html! { <p>{ error.clone() }</p> }
        } else if self.fetching {
            html! { <p>{ "Fetching leaderboard..." }</p> }
        } else if let Some(ref data) = self.data {
            let userid = userid::get_userid();
            let render_time_row = |rank: usize, row: &TimeLeaderboardRow| -> Html {
                let class = (row.userid == userid).then_some("own-leaderboard-entry");
                let copy_encrypted_code_cb = {
                    let text = row.encrypted_code.clone();
                    move |_| {
                        crate::js::clipboard::write(&text);
                    }
                };
                let play_cb = {
                    let text = row.encrypted_code.clone();
                    context.props().play_cb.reform(move |_| text.clone())
                };
                html! {
                    <tr class={classes!(class)}>
                        <td class="centered"><b>{ rank }</b></td>
                        <td>{ row.username.clone().unwrap_or_else(|| userid::generate_username(&row.userid)) }</td>
                        <td>{ &row.time }</td>
                        <td>
                            <a class="material-symbols-outlined" onclick={copy_encrypted_code_cb}>{ "content_copy" }</a>
                            <a class="material-symbols-outlined" onclick={play_cb}>{ "play_arrow" }</a>
                        </td>
                    </tr>
                }
            };

            let own_row_index = data
                .lowest_time
                .iter()
                .position(|row| row.userid == userid)
                .unwrap_or(std::usize::MAX - 1);

            let mut table_rows = vec![];
            let mut last_index = None;
            for (i, row) in data.lowest_time.iter().enumerate() {
                let rank = i + 1;
                let add_entry = i < 10
                    || i + 1 == own_row_index
                    || i == own_row_index
                    || i == own_row_index + 1;
                if add_entry {
                    if let Some(last_index) = last_index {
                        if last_index + 1 != i {
                            let skipped = i - (last_index + 1);
                            table_rows.push(html! { <tr><td colspan=4 class="skip">{ "skipped " }{ skipped }{ " rows" }</td></tr> });
                        }
                    }
                    table_rows.push(render_time_row(rank, row));
                    last_index = Some(i);
                }
            }
            if let Some(last_index) = last_index {
                if last_index + 1 != data.lowest_time.len() {
                    let skipped = data.lowest_time.len() - (last_index + 1);
                    table_rows.push(html! { <tr><td colspan=4 class="skip">{ "skipped " }{ skipped }{ " rows" }</td></tr> });
                }
            }

            html! {
                <div class="leaderboard">
                    <table>
                        <tr><th colspan=3>{ "Leaderboard" }</th></tr>
                        <tr><th>{ "Rank" }</th><th>{ "User" }</th><th>{ "Time" }</th><th>{ "Encrypted Code" }</th></tr>
                        <tbody>{ for table_rows }</tbody>
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
