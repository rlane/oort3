use oort_simulator::scenario;
use oort_version_control::Version;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug)]
pub enum Msg {
    StartFetch(String),
    FetchFinished(Vec<Version>),
    FetchFailed,
}

#[derive(Properties, Clone, PartialEq)]
pub struct VersionsWindowProps {
    pub host: web_sys::Element,
    pub scenario_name: String,
    pub load_cb: Callback<String>,
    pub save_cb: Callback<String>,
    pub update_timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct VersionsWindow {
    versions: Vec<Version>,
}

impl Component for VersionsWindow {
    type Message = Msg;
    type Properties = VersionsWindowProps;

    fn create(context: &yew::Context<Self>) -> Self {
        context
            .link()
            .send_message(Msg::StartFetch(context.props().scenario_name.clone()));
        Self {
            versions: Vec::new(),
        }
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StartFetch(scenario_name) => {
                context.link().send_future(async move {
                    let version_control =
                        oort_version_control::VersionControl::new().await.unwrap();
                    match version_control.list_versions(&scenario_name).await {
                        Ok(versions) => Msg::FetchFinished(versions),
                        Err(e) => {
                            log::error!("Error fetching versions: {:?}", e);
                            Msg::FetchFailed
                        }
                    }
                });
                false
            }
            Msg::FetchFinished(versions) => {
                self.versions = versions;
                true
            }
            Msg::FetchFailed => false,
        }
    }

    fn changed(&mut self, context: &Context<Self>, _old_props: &Self::Properties) -> bool {
        context
            .link()
            .send_message(Msg::StartFetch(context.props().scenario_name.clone()));
        false
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        let input_ref = NodeRef::default();
        let submit_cb = {
            let input_ref = input_ref.clone();
            let save_cb = context.props().save_cb.clone();
            Callback::from(move |event: yew::events::SubmitEvent| {
                event.prevent_default();
                save_cb.emit(input_ref.cast::<HtmlInputElement>().unwrap().value())
            })
        };

        let versions_html = if self.versions.is_empty() {
            html! {
                <li>{ "No previous versions found." }</li>
            }
        } else {
            self.versions
                .iter()
                .map(|version| {
                    let onclick = {
                        let version_id = version.id.clone();
                        context.props().load_cb.reform(move |_| version_id.clone())
                    };
                    let ts = version
                        .timestamp
                        .with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string();
                    let text = if let Some(label) = &version.label {
                        format!("{ts} ({label})")
                    } else {
                        ts
                    };
                    html! { <li><a href="#" {onclick}>{ text }</a></li> }
                })
                .collect::<Html>()
        };

        let scenario_names = scenario::list()
            .iter()
            .flat_map(|x| x.1.clone())
            .collect::<Vec<_>>();
        let scenario_options = scenario_names
            .iter()
            .map(|scenario_name| {
                let selected = scenario_name == &context.props().scenario_name;
                html! { <option value={scenario_name.clone()} {selected}>{ scenario_name }</option> }
            })
            .collect::<Html>();

        let scenario_select_cb = context.link().callback(move |e: Event| {
            let target: web_sys::EventTarget = e.target().unwrap();
            let scenario_name = target.unchecked_into::<HtmlInputElement>().value();
            Msg::StartFetch(scenario_name)
        });

        create_portal(
            html! {
                <div class="versions">
                    <h1>{ "Versions" }</h1>
                    <form onsubmit={submit_cb}>
                        <input type="text" ref={input_ref} />
                        <button type="submit">{ "Save" }</button>
                    </form>
                    <p>{ "This list shows previous versions of your code for this scenario. Click on a version to load it." }</p>
                    <p><select onchange={scenario_select_cb}>{scenario_options}</select></p>
                    <ul>
                    { versions_html }
                    </ul>
                </div>
            },
            context.props().host.clone(),
        )
    }
}
