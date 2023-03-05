use crate::services;
use oort_proto::{TournamentCompetitor, TournamentResults};
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
                    <h1>{ "Tournament Results" }</h1>
                    <p>{ "Scenario: " }{ data.scenario_name.clone() }</p>
                    <p>
                        { "Ratings are calculated with " }
                        <a href="https://en.wikipedia.org/wiki/Glicko_rating_system">{ "Glicko-2" }</a>
                        {". Click a username to play against their AI." }
                    </p>
                    { make_ratings_table(data) }
                    <br />
                    <p>
                        { "This table shows the win percentage of the user in the row vs the user in the column. " }
                        { "Click a cell to run those AIs against each other." }
                    </p>
                    { make_win_matrix_table(data) }
                </div>
            }
        } else {
            html! {}
        }
    }
}

fn make_ratings_table(data: &TournamentResults) -> Html {
    let make_link =
        |shortcode: &str| format!("/scenario/{}?player1={}", data.scenario_name, shortcode);
    html! {
        <table>
            <tr>
                <th>{ "Username" }</th>
                <th>{ "Rating" }</th>
            </tr>
            { data.competitors.iter().map(|x| html! {
                <tr>
                    <td><a href={make_link(&x.shortcode)}>{ x.username.clone() }</a></td>
                    <td>{ x.rating.round() }</td>
                </tr>
            }).collect::<Html>() }
        </table>
    }
}

fn make_win_matrix_table(data: &TournamentResults) -> Html {
    let make_link = |c0: &TournamentCompetitor, c1: &TournamentCompetitor| {
        format!(
            "/scenario/{}?player0={}&player1={}",
            data.scenario_name, c0.shortcode, c1.shortcode
        )
    };
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
                data.competitors.iter().map(|c0| html! { <tr><td>{ &c0.username }</td>
                    { data.competitors.iter().map(|c1| {
                        let v = data.win_matrix[index];
                        index += 1;
                        if c0 == c1 {
                            html! { <td><a href={make_link(c0, c1)}>{ "-" }</a></td> }
                        } else {
                            html! { <td><a href={make_link(c0, c1)}>{ (v * 100.0).round() }</a></td> }
                        }
                    }).collect::<Html>() }
                </tr> }).collect::<Html>()
            }
        </table>
    }
}
