use crate::services;
use oort_proto::Telemetry;
use web_sys::{HtmlTextAreaElement, MouseEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FeedbackProps {
    pub close_overlay_cb: Callback<()>,
}

#[function_component(Feedback)]
pub fn feedback(props: &FeedbackProps) -> Html {
    let text_ref = NodeRef::default();

    let submit_cb = {
        let text_ref = text_ref.clone();
        let close_overlay_cb = props.close_overlay_cb.clone();
        move |_: MouseEvent| {
            services::send_telemetry(Telemetry::Feedback {
                text: text_ref.cast::<HtmlTextAreaElement>().unwrap().value(),
            });
            close_overlay_cb.emit(());
        }
    };

    html! {
        <>
            <h1>{ "Feedback" }</h1>
            <p>
                { "I'm glad to get your feedback! Consider posting on " }
                <a href="https://discord.gg/vYyu9EhkKH" target="_blank">{ "Discord" }</a>
                {" as well." }
            </p>
            <textarea ref={text_ref} rows="10" cols="80"></textarea>
            <br />
            <input type="submit" onclick={submit_cb} />
        </>
    }
}
