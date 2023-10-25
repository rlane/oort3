use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DocumentationProps {
    pub host: web_sys::Element,
    pub show_feedback_cb: Callback<MouseEvent>,
}

#[function_component(Documentation)]
pub fn documentation(props: &DocumentationProps) -> Html {
    let htm = html! {
        <div class="documentation">
            <h1>{ "Quick Reference" }</h1>
            { "Please file bugs at " }<a href="http://github.com/rlane/oort3/issues" target="_blank">{ "GitHub" }</a>
            { " and give feedback on " } <a href="https://discord.gg/vYyu9EhkKH" target="_blank">{ "Discord" }</a>
            { " or " }<a href="#" onclick={props.show_feedback_cb.clone()}>{ "in-game" }</a>{ ". " }
            { "Also take a look at the " }<a href="https://github.com/rlane/oort3/wiki" target="_blank">{ "wiki" }</a>{ "." }<br/>
            { "The " }<a href="https://docs.rs/oort_api">{ "API reference" }</a>{ " contains more detailed information." }

            <h2>{ "Basics" }</h2>
            { "Select a scenario from the list in the top-right of the page." }<br/>
            { "Click the run button in the editor to start the scenario with a new version of your code." }<br/>

            <h2>{ "Controls" }</h2>
            <ul>
                <li>{ "W/A/S/D: Pan the camera." }</li>
                <li>{ "Space: Pause/resume." }</li>
                <li>{ "N: Single-step (advance time by one tick and then pause)." }</li>
                <li>{ "F: Fast-forward." }</li>
                <li>{ "M: Slow motion." }</li>
                <li>{ "G: Show debug lines for all ships." }</li>
                <li>{ "C: Chase, or follow the selected ship." }</li>
                <li>{ "V: Toggle NLIPS, which makes smaller ships more visible when zoomed out." }</li>
                <li>{ "B: Toggle postprocessing (blur)." }</li>
                <li>{ "Mouse wheel: Zoom." }</li>
                <li>{ "Mouse click: Select a ship to show debugging info." }</li>
            </ul>

            <h2>{ "Language" }</h2>
            <p>
                { "Oort AIs are written in " }<a href="https://www.rust-lang.org/" target="_blank">{ "Rust" }</a>{ ". " }
                { "For an introduction to the language check out " }<a href="https://doc.rust-lang.org/stable/rust-by-example/" target="_blank">{ "Rust By Example" }</a>{ ". " }
            </p>

            <p>
                { "The starter code for each scenario includes a Ship struct with a "}<code>{ "tick" }</code>{ " method that the game will call 60 times per second. "}
                { "You can also store state in this struct which can be initialized in "}<code>{ "new" }</code>{ " and accessed with " }<code>{ "self.field_name" }</code>{ ". "}
            </p>

            <p>
                { "All interactions between your AI and the game are done using the functions listed below. " }
                { "Many of these functions take or return "}<code>{ "Vec2" }</code>{ ", which is a 2-dimensional double-precision vector type." }
            </p>

            <h2>{ "Coordinate System" }</h2>
            <p>
                { "The game world is a 2D plane with the origin at the center. " }
                { "The X axis points to the right and the Y axis points up. " }
                { "The Wikipedia article on the " }<a href="https://en.wikipedia.org/wiki/Cartesian_coordinate_system" target="_blank">{ "Cartesian coordinate system" }</a>{ " has a picture." }
            </p>

            <p>
                { "The API uses units of meters, radians, and seconds." }
            </p>

            <h2>{ "Ship Status and Control" }</h2>
            <ul>
              <li><code>{ "class() → Class" }</code>{ ": Returns the ship class." }</li>
              <li><code>{ "position() → Vec2" }</code>{ ": Get the current position in meters." }</li>
              <li><code>{ "velocity() → Vec2" }</code>{ ": Get the current velocity in m/s." }</li>
              <li><code>{ "heading() → f64" }</code>{ ": Get the current heading in radians." }</li>
              <li><code>{ "angular_velocity() → f64" }</code>{ ": Get the current angular velocity in radians/s." }</li>
              <li><code>{ "health() → f64" }</code>{ ": Current health." }</li>
              <li><code>{ "fuel() → f64" }</code>{ ": Current fuel (delta-v)." }</li>
              <li><code>{ "accelerate(acceleration: Vec2)" }</code>{ ": Accelerate the ship. Units are m/s²." }</li>
              <li><code>{ "turn(speed: f64)" }</code>{ ": Rotate the ship. Unit is radians/s." }</li>
              <li><code>{ "torque(acceleration: f64)" }</code>{ ": Angular acceleration. Unit is radians/s²." }</li>
              <li><code>{ "max_forward_acceleration() -> f64" }</code>{ ": Maximum forward acceleration." }</li>
              <li><code>{ "max_backward_acceleration() -> f64" }</code>{ ": Maximum backward acceleration." }</li>
              <li><code>{ "max_lateral_acceleration() -> f64" }</code>{ ": Maximum lateral acceleration." }</li>
              <li><code>{ "max_angular_acceleration() -> f64" }</code>{ ": Maximum angular acceleration." }</li>
            </ul>

            <h2>{ "Weapons" }</h2>
            <ul>
              <li><code>{ "fire(index: usize)" }</code>{ ": Fire a weapon (gun or missile launcher)." }</li>
              <li><code>{ "aim(index: usize, angle: f64)" }</code>{ ": Aim a weapon (for weapons on a turret)." }</li>
              <li><code>{ "explode()" }</code>{ ": Self-destruct." }</li>
            </ul>

            <h2>{ "Radar" }</h2>
            <ul>
              <li><code>{ "set_radar_heading(angle: f64)" }</code>{ ": Point the radar at the given heading." }</li>
              <li><code>{ "radar_heading() -> f64" }</code>{ ": Get current radar heading." }</li>
              <li><code>{ "set_radar_width(width: f64)" }</code>{ ": Adjust the width of the radar beam (in radians)." }</li>
              <li><code>{ "radar_width() -> f64" }</code>{ ": Get current radar width." }</li>
              <li><code>{ "scan() → Option<ScanResult>" }</code>{ ": Find an enemy ship illuminated by the radar." }</li>
              <li><code>{ "struct ScanResult { position: Vec2, velocity: Vec2 }" }</code></li>
            </ul>

            <h2>{ "Advanced Radar" }</h2>
            <ul>
              <li><code>{ "set_radar_min_distance(dist: f64)" }</code>{ ": Set the minimum distance filter." }</li>
              <li><code>{ "radar_min_distance() -> f64" }</code>{ ": Get current minimum distance filter." }</li>
              <li><code>{ "set_radar_max_distance(dist: f64)" }</code>{ ": Set the maximum distance filter." }</li>
              <li><code>{ "radar_max_distance() -> f64" }</code>{ ": Get current maximum distance filter." }</li>
              <li><code>{ "set_radar_ecm_mode(mode: EcmMode)" }</code>{ ": Set the Electronic Counter Measures (ECM) mode." }</li>
              <li><code>{ "EcmMode::None" }</code>{ ": No ECM, radar will operate normally." }</li>
              <li><code>{ "EcmMode::Noise" }</code>{ ": Decrease the enemy radar's signal to noise ratio, making it more difficult to detect targets and reducing accuracy of returned contacts." }</li>
            </ul>

            <h2>{ "Radio" }</h2>
            <ul>
              <li><code>{ "set_radio_channel(channel: usize)" }</code>{ ": Change the radio channel (0 to 9). Takes effect next tick." }</li>
              <li><code>{ "get_radio_channel() -> usize" }</code>{ ": Get the radio channel." }</li>
              <li><code>{ "send(data: [f64; 4])" }</code>{ ": Send a message on a channel." }</li>
              <li><code>{ "receive() -> Option<[f64; 4]>" }</code>{ ": Receive a message from the channel. The message with the strongest signal is returned." }</li>
              <li><code>{ "send_bytes(data: &[u8])" }</code>{ ": Send a message on a channel as bytes, the data will be zero-filled or truncated to a length of 32 bytes." }</li>
              <li><code>{ "receive_bytes() -> Option<[u8; 32]>" }</code>{ ": Just like receive, but instead the message will be returned as a byte array." }</li>
              <li><code>{ "select_radio(index: usize)" }</code>{ ": Select the radio to control with subsequent API calls. Frigates have 4 radios and cruisers have 8." }</li>
            </ul>

            <h2>{ "Special Abilities" }</h2>
            <ul>
              <li><code>{ "activate_ability(ability: Ability)" }</code>{ ": Activates a ship's special ability." }</li>
              <li><code>{ "deactivate_ability(ability: Ability)" }</code>{ ": Deactivates a ship's special ability." }</li>
              <li><code>{ "active_abilities() → ActiveAbilities" }</code>{ ": Returns the ship's active abilities." }</li>
              <li>{ "Available abilities:" }
                <ul>
                  <li><code>{ "Ability::Boost" }</code>{ ": Fighter and missile only. Applies a 100 m/s² forward acceleration for 2s. Reloads in 10s." }</li>
                  <li><code>{ "Ability::Decoy" }</code>{ ": Torpedo only. Mimics the radar signature of a Cruiser for 0.5s. Reloads in 10s." }</li>
                  <li><code>{ "Ability::Shield" }</code>{ ": Cruiser only. Deflects damage for 1s. Reloads in 5s." }</li>
                </ul>
              </li>
            </ul>

            <h2>{ "Scalar Math" }</h2>
            <ul>
              <li><code>{ "PI, TAU" }</code>{ ": Constants."}</li>
              <li><code>{ "x.abs()" }</code>{ ": Absolute value."}</li>
              <li><code>{ "x.sqrt()" }</code>{ ": Square root."}</li>
              <li><code>{ "x.sin(), x.cos(), x.tan()" }</code>{": Trignometry."}</li>
              <li>{ "See the " }<a href="https://doc.rust-lang.org/std/primitive.f64.html" target="_blank">{ "Rust documentation" }</a>{ " for the full list of f64 methods." }</li>
            </ul>

            <h2>{ "Vector Math" }</h2>
            <span>
                { "For a refresher on vectors check out this " }
                <a href="https://phys.libretexts.org/Bookshelves/University_Physics/Radically_Modern_Introductory_Physics_Text_I_(Raymond)/02%3A_Waves_in_Two_and_Three_Dimensions/2.01%3A_Math_Tutorial__Vectors"
                   target="_blank">{ "tutorial" }</a>
                { "." }
            </span>
            <ul>
              <li><code>{ "vec2(x, y) → Vec2" }</code>{ ": Create a vector." }</li>
              <li><code>{ "v.x, v.y → f64" }</code>{ ": Get a component of a vector." }</li>
              <li><code>{ "v1 +- v2 → Vec2" }</code>{ ": Basic arithmetic between vectors." }</li>
              <li><code>{ "v */ f64 → Vec2" }</code>{ ": Basic arithmetic between vectors and scalars." }</li>
              <li><code>{ "-v → Vec2" }</code>{ ": Negate a vector." }</li>
              <li><code>{ "v.length() → f64" }</code>{ ": Length." }</li>
              <li><code>{ "v.normalize() → Vec2" }</code>{ ": Normalize to a unit vector." }</li>
              <li><code>{ "v.rotate(f64) → Vec2" }</code>{ ": Rotate counter-clockwise." }</li>
              <li><code>{ "v.angle() → f64" }</code>{ ": Angle of a vector." }</li>
              <li><code>{ "v1.dot(v2: Vec2) → f64" }</code>{ ": Dot product." }</li>
              <li><code>{ "v1.distance(v2: Vec2) → f64" }</code>{ ": Distance between two points." }</li>
            </ul>

            <h2>{ "Debugging" }</h2>
            <ul>
              <li><code>{ "debug!(...)" }</code>{ ": Add text to be displayed when the ship is selected by clicking on it. Works just like " }<code>{ "println!" }</code>{ "." }</li>
              <li><code>{ "draw_line(v0: Vec2, v1: Vec2, color: u32)" }</code>{ ": Draw a line visible when the ship is selected. Color is 24-bit RGB." }</li>
              <li><code>{ "draw_triangle(center: Vec2, radius: f64, color: u32)" }</code>{ ": Draw a triangle visible when the ship is selected." }</li>
              <li><code>{ "draw_square(center: Vec2, radius: f64, color: u32)" }</code>{ ": Draw a square visible when the ship is selected." }</li>
              <li><code>{ "draw_diamond(center: Vec2, radius: f64, color: u32)" }</code>{ ": Draw a diamond visible when the ship is selected." }</li>
              <li><code>{ "draw_polygon(center: Vec2, radius: f64, sides: i32, angle: f64, color: u32)" }</code>{ ": Draw a regular polygon visible when the ship is selected." }</li>
              <li><code>{ "draw_text!(topleft: Vec2, color: u32, ...)" }</code>{ ": Draw text. Works like " }<code>{ "println!" }</code>{ "." }</li>
            </ul>

            <h2>{ "Miscellaneous" }</h2>
            <ul>
              <li><code>{ "current_tick() → u32" }</code>{ ": Returns the number of ticks elapsed since the simulation started." }</li>
              <li><code>{ "current_time() → f64" }</code>{ ": Returns the number of seconds elapsed since the simulation started." }</li>
              <li><code>{ "angle_diff(a: f64, b: f64) → f64" }</code>{ ": Returns the shortest (possibly negative) distance between two angles." }</li>
              <li><code>{ "rand(low: f64, high: f64) → f64" }</code>{ ": Get a random number." }</li>
              <li><code>{ "target() → Vec2" }</code>{ ": Used in some scenarios, returns the position of the target." }</li>
              <li><code>{ "target_velocity() → Vec2" }</code>{ ": Used in some scenarios, returns the velocity of the target." }</li>
              <li><code>{ "seed() → u128" }</code>{ ": Returns a seed useful for initializing a random number generator." }</li>
            </ul>

            <h2>{ "Extra Crates" }</h2>
            <p>{ "The following crates are available for use in your code:" }</p>
            <ul>
                <li><a href="https://docs.rs/byteorder/1.4.3/byteorder/index.html" target="_blank">{ "byteorder" }</a>{ ": Utilities to read and write binary data, useful for radio." }</li>
                <li><a href="https://docs.rs/maths-rs/0.2.4/maths_rs/index.html" target="_blank">{ "maths_rs" }</a>{ ": A linear algebra library." }</li>
                <li><a href="https://docs.rs/oorandom/11.1.3/oorandom/index.html" target="_blank">{ "oorandom" }</a>{ ": A random number generation library." }</li>
            </ul>

            <h2>{ "Ship Classes" }</h2>
            <ul>
              <li>{ "Fighter: Small, fast, and lightly armored. One forward-facing gun and one missile launcher. "}</li>
              <li>{ "Frigate: Medium size with heavy armor. One forward-facing high-velocity gun, two turreted guns, and one missile launcher. "}</li>
              <li>{ "Cruiser: Large, slow, and heavily armored. One turreted flak gun, two missile launchers, and one torpedo launcher. "}</li>
              <li>{ "Missile: Highly maneuverable but unarmored. Explodes on contact or after an " }<code>{ "explode()" }</code>{ " call." }</li>
              <li>{ "Torpedo: Better armor, larger warhead, but less maneuverable than a missile. Explodes on contact or after an " }<code>{ "explode()" }</code>{ " call." }</li>
            </ul>
        </div>
    };

    create_portal(htm, props.host.clone())
}
