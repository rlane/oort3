use yew::{function_component, html};

#[function_component(Documentation)]
pub fn documentation() -> Html {
    html! {
        <>
            <h1>{ "Quick Reference" }</h1>
            { "Press Escape to close. File bugs on " }<a href="http://github.com/rlane/oort3/issues" target="_none">{ "GitHub" }</a>{ "." }<br />

            <h2>{ "Basics" }</h2>
            { "Select a scenario from the list in the top-right of the page." }<br/>
            { "Press Ctrl-Enter in the editor to run the scenario with a new version of your code." }<br/>
            { "The game calls your " }<code>{ "tick()" }</code>{ " function 60 times per second." }
        </>
    }
}
