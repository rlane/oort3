use crate::services;
use oort_proto::TournamentResults;
use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {
    SendRequest,
    ReceiveResponse(Result<TournamentResults, anyhow::Error>),
}

#[derive(Properties, Clone, PartialEq, Eq)]
pub struct TournamentProps {
    pub id: String,
}

pub struct Tournament {
    data: Option<TournamentResults>,
    error: Option<String>,
    fetching: bool,
}

impl Component for Tournament {
    type Message = Msg;
    type Properties = TournamentProps;

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
                let id = context.props().id.clone();
                let link = context.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    link.send_message(Msg::ReceiveResponse(
                        services::get_tournament_results(&id).await,
                    ));
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
        if self.fetching {
            html! {
                <div id="tournament_results">{ "Fetching tournament results..." }</div>
            }
        } else if let Some(e) = self.error.as_ref() {
            html! {
                <div id="tournament_results">{ "Failed to fetch tournament results: " }{ format!("{e:?}") }</div>
            }
        } else if let Some(data) = self.data.as_ref() {
            html! {
                <div id="tournament_results">
                    <p>{ "Scenario: " }{ data.scenario_name.clone() }</p>
                    { make_ratings_table(data) }
                    <br />
                    { make_win_matrix_table(data) }
                </div>
            }
        } else {
            html! {}
        }
    }
}

fn make_ratings_table(data: &TournamentResults) -> Html {
    html! {
        <table>
            <tr>
                <th>{ "Username" }</th>
                <th>{ "Rating" }</th>
            </tr>
            { data.competitors.iter().map(|x| html! {
                <tr><td>{ x.username.clone() }</td><td>{ x.rating.round() }</td></tr>
            }).collect::<Html>() }
        </table>
    }
}

fn make_win_matrix_table(data: &TournamentResults) -> Html {
    let username: Vec<_> = data
        .competitors
        .iter()
        .map(|x| x.username.clone())
        .collect();
    let mut index = 0;
    html! {
        <table>
            <tr>
                <th>{ "Winner / Loser" }</th>
                { username.iter().map(|x| html! { <td>{ x.clone() }</td> }).collect::<Html>() }
            </tr>
            {
                username.iter().map(|x| html! { <tr><td>{ &x }</td>
                    { username.iter().map(|y| {
                        let v = data.win_matrix[index];
                        index += 1;
                        if x == y{
                            html! { <td></td> }
                        } else {
                            html! { <td>{ (v * 100.0).round() }</td> }
                        }
                    }).collect::<Html>() }
                </tr> }).collect::<Html>()
            }
        </table>
    }
}
