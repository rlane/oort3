This is the API reference for [Oort](oort.rs). For more general information see
the [wiki](https://github.com/rlane/oort3/wiki).

# Starter Code

Oort expects your code to have a `Ship` type with a `tick` method. Each
tutorial provides some starter code which includes this:

```rust
use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
    }
}
```

The game will call your `new` function when a ship is created and then call
`tick` 60 times per second during the simulation.

`struct Ship` is useful for storing any state that needs to persist between
ticks. `enum Ship` works too and can be helpful when this state differs between
ship classes.

The statement `use oort_api::prelude::*` imports all the APIs so that you can use
them simply as e.g. `position()`. See the [prelude] module documentation for
the details on everything this imports. The important APIs are covered below.

# Subsystems

## Ship Status and Control

- [`class() → Class`](prelude::class): Returns the ship class.
- [`position() → Vec2`](prelude::position): Get the current position in meters.
- [`velocity() → Vec2`](prelude::velocity): Get the current velocity in m/s.
- [`heading() → f64`](prelude::heading): Get the current heading in radians.
- [`angular_velocity() → f64`](prelude::angular_velocity): Get the current angular velocity in radians/s.
- [`accelerate(acceleration: Vec2)`](prelude::accelerate): Linear acceleration. X axis is forward/back, Y axis is left/right. Units are m/s².
- [`max_acceleration() -> Vec2`](prelude::max_acceleration): Maximum linear acceleration.
- [`torque(acceleration: f64)`](prelude::torque): Angular acceleration. Unit is radians/s².
- [`max_angular_acceleration() -> f64`](prelude::max_angular_acceleration): Maximum angular acceleration.
- [`energy() -> f64`](prelude::energy): Available energy in Joules.

## Weapons

- [`fire_gun(index: usize)`](prelude::fire_gun): Fire a gun.
- [`aim_gun(index: usize, angle: f64)`](prelude::aim_gun): Aim a gun (for guns on a turret).
- [`launch_missile(index: usize, orders: f64)`](prelude::launch_missile): Launch a missile.
- [`explode()`](prelude::explode): Self-destruct.

## Radar

- [`set_radar_heading(angle: f64)`](prelude::set_radar_heading): Point the radar at the given heading.
- [`radar_heading() -> f64`](prelude::radar_heading): Get current radar heading.
- [`set_radar_width(width: f64)`](prelude::set_radar_width): Adjust the width of the radar beam (in radians).
- [`radar_width() -> f64`](prelude::radar_width): Get current radar width.
- [`set_radar_min_distance(dist: f64)`](prelude::set_radar_min_distance): Set the minimum distance filter.
- [`radar_min_distance() -> f64`](prelude::radar_min_distance): Get current minimum distance filter.
- [`set_radar_max_distance(dist: f64)`](prelude::set_radar_max_distance): Set the maximum distance filter.
- [`radar_max_distance() -> f64`](prelude::radar_max_distance): Get current maximum distance filter.
- [`scan() → Option<ScanResult>`](prelude::scan): Find an enemy ship illuminated by the radar.
- [`struct ScanResult { position: Vec2, velocity: Vec2, class: Class }`](prelude::ScanResult): Structure returned by `scan()`.

## Radio

- [`set_radio_channel(channel: usize)`](prelude::set_radio_channel): Change the radio channel (0 to 9). Takes effect next tick.
- [`get_radio_channel() -> usize`](prelude::get_radio_channel): Get the radio channel.
- [`send(data: f64)`](prelude::send): Send a message on a channel.
- [`receive() -> f64`](prelude::receive): Receive a message from the channel. The message with the strongest signal is returned.

## Scalar Math

- [`PI`](prelude::PI), [`TAU`](prelude::TAU): Constants.
- [`x.abs()`](f64::abs): Absolute value.
- [`x.sqrt()`](f64::sqrt): Square root.
- [`x.sin()`](f64::sin), [`x.cos()`](f64::cos), [`x.tan()`](f64::tan): Trignometry.

See the [Rust documentation](https://doc.rust-lang.org/std/primitive.f64.html) for the full list of f64 methods.

## Vector Math

Two-dimensional floating point vectors ([Vec2](prelude::Vec2)) are ubiquitous
in Oort and are used to represent positions, velocities, accelerations, etc.

- [`vec2(x: f64, y: f64) → Vec2`](prelude::vec2): Create a vector.
- `v.x, v.y → f64`: Get a component of a vector.
- `v1 +- v2 → Vec2`: Basic arithmetic between vectors.
- `v */ f64 → Vec2`: Basic arithmetic between vectors and scalars.
- `-v → Vec2`: Negate a vector.
- [`v.length() → f64`](prelude::Vec2::length): Length.
- [`v.normalize() → Vec2`](prelude::Vec2::normalize): Normalize to a unit vector.
- [`v.rotate(angle: f64) → Vec2`](prelude::Vec2::rotate): Rotate counter-clockwise.
- [`v.angle() → f64`](prelude::Vec2::angle): Angle of a vector.
- [`v1.dot(v2: Vec2) → f64`](prelude::Vec2::dot): Dot product.
- [`v1.distance(v2: Vec2) → f64`](prelude::Vec2::distance): Distance between two points.

## Debugging

- [`debug!(...)`](prelude::debug!): Add text to be displayed when the ship is selected by clicking on it. Works just like [`println!`].
- [`debug_line(v0: Vec2, v1: Vec2, color: u32)`](prelude::debug_line): Draw a line visible when the ship is selected. Color is 24-bit RGB.
- [`debug_triangle(center: Vec2, radius: f64, color: u32)`](prelude::debug_triangle): Draw a triangle visible when the ship is selected.
- [`debug_square(center: Vec2, radius: f64, color: u32)`](prelude::debug_square): Draw a square visible when the ship is selected.
- [`debug_diamond(center: Vec2, radius: f64, color: u32)`](prelude::debug_diamond): Draw a diamond visible when the ship is selected.
- [`debug_polygon(center: Vec2, radius: f64, sides: i32, angle: f64, color: u32)`](prelude::debug_polygon): Draw a regular polygon visible when the ship is selected.

## Miscellaneous

- [`current_tick() → f64`](prelude::current_tick): Returns the number of ticks elapsed since the simulation started.
- [`current_time() → f64`](prelude::current_time): Returns the number of seconds elapsed since the simulation started.
- [`angle_diff(a: f64, b: f64) → f64`](prelude::angle_diff): Returns the shortest (possibly negative) distance between two angles.
- [`rand(low: f64, high: f64) → f64`](prelude::rand): Get a random number.
- [`orders() → f64`](prelude::orders): Returns the orders passed to launch_missile.
- [`seed() → u128`](prelude::seed): Returns a seed useful for initializing a random number generator.

## Ship Classes

- [`Fighter`](prelude::Class::Fighter): Small, fast, and lightly armored. One forward-facing gun and one missile launcher.
- [`Frigate`](prelude::Class::Frigate): Medium size with heavy armor. One forward-facing high-velocity gun, two turreted guns, and one missile launcher.
- [`Cruiser`](prelude::Class::Cruiser): Large, slow, and heavily armored. One turreted flak gun, two missile launchers, and one torpedo launcher.
- [`Missile`](prelude::Class::Missile): Highly maneuverable but unarmored. Explodes on contact or after an explode() call.
- [`Torpedo`](prelude::Class::Torpedo): Better armor, larger warhead, but less maneuverable than a missile. Explodes on contact or after an explode() call.
