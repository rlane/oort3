use yew::{function_component, html, Html};

#[function_component(Documentation)]
pub fn documentation() -> Html {
    html! {
        <>
            <h1>{ "Quick Reference" }</h1>
            { "Press Escape to close. Please file bugs at " }<a href="http://github.com/rlane/oort3/issues" target="_none">{ "GitHub" }</a>
            { " and give feedback on " } <a href="https://discord.gg/vYyu9EhkKH" target="_none">{ "Discord" }</a>{ ". " }
            { "Also take a look at the " }<a href="https://github.com/rlane/oort3/wiki">{ "wiki" }</a>{ "." }<br/>

            <h2>{ "Basics" }</h2>
            { "Select a scenario from the list in the top-right of the page (after closing the documentation overlay)." }<br/>
            { "Press Ctrl-Enter in the editor (Cmd-Enter on Mac) to run the scenario with a new version of your code." }<br/>

            <h2>{ "Controls" }</h2>
            <ul>
                <li>{ "W/A/S/D: Pan the camera." }</li>
                <li>{ "Space: Pause/resume." }</li>
                <li>{ "N: Single-step." }</li>
                <li>{ "F: Fast-forward." }</li>
                <li>{ "G: Show debug lines for all ships." }</li>
                <li>{ "Mouse wheel: Zoom." }</li>
                <li>{ "Mouse click: Select a ship to show debugging info." }</li>
            </ul>

            <h2>{ "Language" }</h2>
            <p>
                { "Oort AIs are written in " }<a href="https://www.rust-lang.org/">{ "Rust" }</a>{ ". " }
                { "For an introduction to the language check out " }<a href="https://doc.rust-lang.org/stable/rust-by-example/">{ "Rust By Example" }</a>{ ". " }
            </p>

            <p>
                { "The starter code for each scenario includes a Ship struct with a "}<code>{ "tick" }</code>{ " method that the game will call 60 times per second. "}
                { "You can also store state in this struct which can be initialized in "}<code>{ "new" }</code>{ " and accessed with " }<code>{ "self.field_name" }</code>{ ". "}
            </p>

            <p>
                { "All interactions between your AI and the game are done using the functions listed below. " }
                { "Many of these functions take or return "}<code>{ "Vec2" }</code>{ ", which is a 2-dimensional double-precision vector type." }
            </p>


            <h2>{ "Ship Status and Control" }</h2>
            <ul>
              <li><code>{ "class() → Class" }</code>{ ": Returns the ship class." }</li>
              <li><code>{ "position() → Vec2" }</code>{ ": Get the current position in meters." }</li>
              <li><code>{ "velocity() → Vec2" }</code>{ ": Get the current velocity in m/s." }</li>
              <li><code>{ "heading() → f64" }</code>{ ": Get the current heading in radians." }</li>
              <li><code>{ "angular_velocity() → f64" }</code>{ ": Get the current angular velocity in radians/s." }</li>
              <li><code>{ "accelerate(acceleration: Vec2)" }</code>{ ": Linear acceleration. X axis is forward/back, Y axis is left/right. Units are m/s²." }</li>
              <li><code>{ "max_acceleration() -> Vec2" }</code>{ ": Maximum linear acceleration." }</li>
              <li><code>{ "torque(acceleration: f64)" }</code>{ ": Angular acceleration. Unit is radians/s²." }</li>
              <li><code>{ "max_angular_acceleration() -> f64" }</code>{ ": Maximum angular acceleration." }</li>
              <li><code>{ "energy() -> f64" }</code>{ ": Available energy in Joules." }</li>
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
              <li><code>{ "set_radar_min_distance(dist: f64)" }</code>{ ": Set the minimum distance filter." }</li>
              <li><code>{ "radar_min_distance() -> f64" }</code>{ ": Get current minimum distance filter." }</li>
              <li><code>{ "set_radar_max_distance(dist: f64)" }</code>{ ": Set the maximum distance filter." }</li>
              <li><code>{ "radar_max_distance() -> f64" }</code>{ ": Get current maximum distance filter." }</li>
              <li><code>{ "scan() → Option<ScanResult>" }</code>{ ": Find an enemy ship illuminated by the radar." }</li>
              <li><code>{ "struct ScanResult { position: Vec2, velocity: Vec2 }" }</code></li>
            </ul>

            <h2>{ "Radio" }</h2>
            <ul>
              <li><code>{ "set_radio_channel(channel: usize)" }</code>{ ": Change the radio channel (0 to 9). Takes effect next tick." }</li>
              <li><code>{ "get_radio_channel() -> usize" }</code>{ ": Get the radio channel." }</li>
              <li><code>{ "send(data: f64)" }</code>{ ": Send a message on a channel." }</li>
              <li><code>{ "receive() -> f64" }</code>{ ": Receive a message from the channel. The message with the strongest signal is returned." }</li>
            </ul>

            <h2>{ "Special Abilities" }</h2>
            <ul>
              <li><code>{ "activate_ability(ability: Ability)" }</code>{ ": Activates a ship's special ability." }</li>
              <li>{ "Available abilities:" }
                <ul>
                  <li><code>{ "Ability::Boost" }</code>{ ": Fighter only. Applies a 100 m/s² forward acceleration for 2s. Reloads in 10s." }</li>
                  <li><code>{ "Ability::ShapedCharge" }</code>{ ": Missile only. " }<code>{ "explode()" }</code>{ " will create a jet of shrapnel instead of a circle." }</li>
                </ul>
              </li>
            </ul>

            <h2>{ "Scalar Math" }</h2>
            <ul>
              <li><code>{ "PI, TAU" }</code>{ ": Constants."}</li>
              <li><code>{ "x.abs()" }</code>{ ": Absolute value."}</li>
              <li><code>{ "x.sqrt()" }</code>{ ": Square root."}</li>
              <li><code>{ "x.sin(), x.cos(), x.tan()" }</code>{": Trignometry."}</li>
              <li>{ "See the " }<a href="https://doc.rust-lang.org/std/primitive.f64.html">{ "Rust documentation" }</a>{ " for the full list of f64 methods." }</li>
            </ul>

            <h2>{ "Vector Math" }</h2>
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
              <li><code>{ "debug_line(v0: Vec2, v1: Vec2, color: u32)" }</code>{ ": Draw a line visible when the ship is selected. Color is 24-bit RGB." }</li>
              <li><code>{ "debug_triangle(center: Vec2, radius: f64, color: u32)" }</code>{ ": Draw a triangle visible when the ship is selected." }</li>
              <li><code>{ "debug_square(center: Vec2, radius: f64, color: u32)" }</code>{ ": Draw a square visible when the ship is selected." }</li>
              <li><code>{ "debug_diamond(center: Vec2, radius: f64, color: u32)" }</code>{ ": Draw a diamond visible when the ship is selected." }</li>
              <li><code>{ "debug_polygon(center: Vec2, radius: f64, sides: i32, angle: f64, color: u32)" }</code>{ ": Draw a regular polygon visible when the ship is selected." }</li>
            </ul>

            <h2>{ "Miscellaneous" }</h2>
            <ul>
              <li><code>{ "current_tick() → f64" }</code>{ ": Returns the number of ticks elapsed since the simulation started." }</li>
              <li><code>{ "current_time() → f64" }</code>{ ": Returns the number of seconds elapsed since the simulation started." }</li>
              <li><code>{ "angle_diff(a: f64, b: f64) → f64" }</code>{ ": Returns the shortest (possibly negative) distance between two angles." }</li>
              <li><code>{ "rand(low: f64, high: f64) → f64" }</code>{ ": Get a random number." }</li>
              <li><code>{ "target() → Vec2" }</code>{ ": Used in some scenarios, returns the location of the target." }</li>
              <li><code>{ "seed() → u128" }</code>{ ": Returns a seed useful for initializing a random number generator." }</li>
            </ul>

            <h2>{ "Ship Classes" }</h2>
            <ul>
              <li>{ "Fighter: Small, fast, and lightly armored. One forward-facing gun and one missile launcher. "}</li>
              <li>{ "Frigate: Medium size with heavy armor. One forward-facing high-velocity gun, two turreted guns, and one missile launcher. "}</li>
              <li>{ "Cruiser: Large, slow, and heavily armored. One turreted flak gun, two missile launchers, and one torpedo launcher. "}</li>
              <li>{ "Missile: Highly maneuverable but unarmored. Explodes on contact or after an " }<code>{ "explode()" }</code>{ " call." }</li>
              <li>{ "Torpedo: Better armor, larger warhead, but less maneuverable than a missile. Explodes on contact or after an " }<code>{ "explode()" }</code>{ " call." }</li>
            </ul>
        </>
    }
}
