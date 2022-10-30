use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WelcomeProps {
    pub host: web_sys::Element,
    pub show_feedback_cb: Callback<MouseEvent>,
    pub select_scenario_cb: Callback<String>,
}

#[function_component(Welcome)]
pub fn welcome(props: &WelcomeProps) -> Html {
    let scenario = |name: &'static str| {
        let cb = props.select_scenario_cb.clone();
        move |_: MouseEvent| cb.emit(name.to_string())
    };
    let htm = html! {
        <div class="welcome">
            <h1 class="centered">{ "Welcome to Oort!" }</h1>
            <p>
                { "Oort is a \"programming game\" where you write Rust code to control a fleet of spaceships. " }
                { "Your code is responsible for the engines, weapons, radar, and communications of ships ranging from tiny missiles to massive cruisers." }
            </p>

            <h2>{ "Getting Started" }</h2>
            <p>
                { "Oort includes a series of tutorials you can access from the select box in the top-right of the screen, or you can jump straight to the " }<a href="#" onclick={scenario("tutorial01")}>{ "first tutorial" }</a>{ ". " }
            </p>
            <p>
                { "The built-in editor will be populated with starter code that has a comment describing the objective. " }
                { "Right-clicking in the editor brings up a menu with commands to execute the current code, reload the starter code, load a sample solution (for tutorials), and more. " }
            </p>
            <p>
                { "The first couple of tutorials are very simple (uncomment the provided code) but you should expect the difficulty to ramp up quickly. " }
                { "At the end of the tutorials you should have an AI that is becoming competitive in the \"tournament\" scenarios including Furball, Fleet, and Belt. " }
                { "The endgame is to submit your AI to the tournament system where it will compete against other players' creations." }
            </p>

            <p>
                { "For an introduction to the language check out " }<a href="https://doc.rust-lang.org/stable/rust-by-example/" target="_none">{ "Rust By Example" }</a>{ ". " }
                { "Only basic Rust knowledge is required to play. " }
                { "The \"Documentation\" link in the top-right of the game UI has a quick reference to the API, or you can check out the " }<a href="https://docs.rs/oort_api" target="_none">{ "API Reference" }</a>{ ". " }
                { "You also have large parts of the " }<a href="https://doc.rust-lang.org/std/" target="_none">{ "Rust Standard Library" }</a>{ " available." }
            </p>

            <h2>{ "Next Steps" }</h2>
            <ul>
                <li>{ "Complete a few tutorials starting with " }<a href="#" onclick={scenario("tutorial01")}>{ "tutorial01" }</a> { ". " }<a href="#" onclick={scenario("tutorial05")}>{ "tutorial05" }</a>{ " is where it can get challenging!" }</li>
                <li>{ "Read up on the " }<a href="https://docs.rs/oort_api" target="_none">{ "API" }</a>{ " and the " }<a href="https://github.com/rlane/oort3/wiki" target="_none">{ "wiki" }</a>{ "." }</li>
                <li>{ "Join the " }<a href="https://discord.gg/vYyu9EhkKH" target="_none">{ "Discord" }</a>{ "." }</li>
                <li>{ "Send in your feedback via Discord, a " }
                    <a href="http://github.com/rlane/oort3/issues" target="_none">{ "GitHub issue" }</a>{ ", or " }
                    <a href="#" onclick={props.show_feedback_cb.clone()}>{ "in-game" }</a>{ "." }</li>
            </ul>
        </div>
    };

    create_portal(htm, props.host.clone())
}
