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

            <h2>{ "Ship Control" }</h2>
            <ul>
              <li><code>{ "ship.position() → vec2" }</code>{ ": Get the current position in meters." }</li>
              <li><code>{ "ship.velocity() → vec2" }</code>{ ": Get the current velocity in m/s." }</li>
              <li><code>{ "ship.heading() → float" }</code>{ ": Get the current heading in radians." }</li>
              <li><code>{ "ship.angular_velocity() → float" }</code>{ ": Get the current angular velocity in radians/s." }</li>
              <li><code>{ "ship.accelerate(acceleration: vec2)" }</code>{ ": Linear acceleration. X axis is forward/back, Y axis is left/right. Units are m/s<sup>2</sup>" }</li>
              <li><code>{ "ship.torque(acceleration: float)" }</code>{ ": Angular acceleration. Unit is radians/s<sup>2</sup>." }</li>
              <li><code>{ "ship.fire_weapon()" }</code>{ ": Fire the ship's main weapon." }</li>
              <li><code>{ "ship.launch_missile()" }</code>{ ": Launch a missile." }</li>
              <li><code>{ "ship.class() → string" }</code>{ ": Returns the ship class as a string." }</li>
              <li><code>{ "ship.explode()" }</code>{ ": Self-destruct." }</li>
            </ul>

            <h2>{ "Radar" }</h2>
            <ul>
              <li><code>{ "radar.set_heading(x)" }</code>{ ": Point the radar at the given heading." }</li>
              <li><code>{ "radar.set_width(x)" }</code>{ ": Adjust the width of the radar beam (in radians)." }</li>
              <li><code>{ "radar.scan() → ScanResult" }</code>{ ": Find the closest enemy ship illuminated by the radar." }</li>
              <li><code>{ "ScanResult { position: vec2, velocity: vec2, found: bool }" }</code></li>
            </ul>

            <h2>{ "Scalar Math" }</h2>
            <ul>
              <li><code>{ "x +-*/% y → float" }</code>{ ": Basic arithmetic." }</li>
              <li><code>{ "-x → float" }</code>{ ": Negation." }</li>
              <li><code>{ "x ** y → float" }</code>{ ": Exponentiation." }</li>
              <li><code>{ "abs(x) → float" }</code>{ ": Absolute value." }</li>
              <li><code>{ "sin(x), cos(x), tan(x) → float" }</code>{ ": Trignometry." }</li>
              <li><code>{ "sqrt(x) → float" }</code>{ ": Square root." }</li>
              <li><code>{ "log(x, base) → float" }</code>{ ": Logarithm." }</li>
              <li><code>{ "min(a, b), max(a, b) → float" }</code>{ ": Minimum and maximum." }</li>
              <li><code>{ "PI(), E()" }</code>{ ": Constants." }</li>
            </ul>

            <h2>{ "Vector Math" }</h2>
            <ul>
              <li><code>{ "vec2(x, y) → vec2" }</code>{ ": Create a vector." }</li>
              <li><code>{ "v.x, v.y → float" }</code>{ ": Get a component of a vector." }</li>
              <li><code>{ "v1 +- v2 → vec2" }</code>{ ": Basic arithmetic between vectors." }</li>
              <li><code>{ "v */ float → vec2" }</code>{ ": Basic arithmetic between vectors and scalars." }</li>
              <li><code>{ "-v → vec2" }</code>{ ": Negate a vector." }</li>
              <li><code>{ "v.magnitude() → float" }</code>{ ": Magnitude (length)." }</li>
              <li><code>{ "v.normalize() → vec2" }</code>{ ": Normalize to a unit vector." }</li>
              <li><code>{ "v.rotate(float) → vec2" }</code>{ ": Rotate counter-clockwise." }</li>
              <li><code>{ "v.angle() → float" }</code>{ ": Angle of a vector." }</li>
              <li><code>{ "v1.dot(v2) → float" }</code>{ ": Dot product." }</li>
              <li><code>{ "v1.distance(v2) → float" }</code>{ ": Distance between two points." }</li>
            </ul>

            <h2>{ "Miscellaneous" }</h2>
            <ul>
              <li><code>{ "print(string)" }</code>{ ": Log a message to the browser console." }</li>
              <li><code>{ "rng.next(low, high) → float" }</code>{ ": Get a random number." }</li>
              <li><code>{ "angle_diff(a, b) → float" }</code>{ ": Returns the shortest (possibly negative) distance between two angles." }</li>
              <li><code>{ "dbg.line(v0, v1, color: int)" }</code>{ ": Draw a line. Color is 24-bit RGB." }</li>
              <li><a href="https://rhai.rs/book/language/index.html" target="_blank">{ "Rhai Language Reference" }</a></li>
            </ul>
            <h2>{ "Credits" }</h2>
            { "Rich Lane" }<br/>
            <br/>
        </>
    }
}
