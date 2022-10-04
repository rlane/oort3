This is the API reference for [Oort](https://oort.rs). For more general information see
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

All actions performed by a ship (such as firing weapons or scanning the radar)
occur between ticks. In particular, setting the radar heading or the radio
channel will affect the scan results or messages received on the next tick.

## Ship Status and Control

Basic status:

- [`class() → Class`](prelude::class): Get the ship class ([Fighter](prelude::Class::Fighter), [Cruiser](prelude::Class::Cruiser), etc).
- [`position() → Vec2`](prelude::position): Get the current position in meters.
- [`velocity() → Vec2`](prelude::velocity): Get the current velocity in m/s.
- [`heading() → f64`](prelude::heading): Get the current heading in radians.
- [`angular_velocity() → f64`](prelude::angular_velocity): Get the current angular velocity in radians/s.

Engine control:

- [`accelerate(acceleration: Vec2)`](prelude::accelerate): Linear acceleration. X axis is forward/back, Y axis is left/right. Units are m/s².
- [`torque(acceleration: f64)`](prelude::torque): Angular acceleration. Unit is radians/s².

Engine limits:

- [`max_acceleration() -> Vec2`](prelude::max_acceleration): Maximum linear acceleration.
- [`max_angular_acceleration() -> f64`](prelude::max_angular_acceleration): Maximum angular acceleration.

Energy is required for nearly everything including accelerating, firing
weapons, and scanning with radar. It's replenished steadily by the ship's
reactor.

- [`energy() -> f64`](prelude::energy): Available energy in Joules.

## Weapons

Guns:

- [`fire_gun(index: usize)`](prelude::fire_gun): Fire a gun.
- [`aim_gun(index: usize, angle: f64)`](prelude::aim_gun): Aim a gun (for guns on a turret).

Missiles:

- [`launch_missile(index: usize)`](prelude::launch_missile): Launch a missile.
- [`explode()`](prelude::explode): Self-destruct.

## Radar

Radar in Oort is modeled as a beam that can be pointed in any direction and
which has a width between 1/360 of a circle to a full circle. Enemy ships
illuminated by this beam reflect an amount of energy proportional to their
radar cross section (larger for larger ships). The radar can return one
contact per tick. Any changes to heading/width/filtering take effect on the
next tick.

The position and velocity returned for a contact will have error inversely
related to the signal strength.

Basic operation:

- [`set_radar_heading(angle: f64)`](prelude::set_radar_heading): Point the radar at the given heading, relative to the ship heading.
- [`set_radar_width(width: f64)`](prelude::set_radar_width): Adjust the beam width (in radians).
- [`scan() → Option<ScanResult>`](prelude::scan): Get the radar contact with the highest signal strength.
- [`struct ScanResult { position: Vec2, velocity: Vec2, class: Class }`](prelude::ScanResult): Structure returned by [`scan`](prelude::scan).

Advanced filtering:

- [`set_radar_min_distance(dist: f64)`](prelude::set_radar_min_distance): Set the minimum distance filter.
- [`set_radar_max_distance(dist: f64)`](prelude::set_radar_max_distance): Set the maximum distance filter.

Retrieving current state:

- [`radar_heading() -> f64`](prelude::radar_heading): Get current radar heading.
- [`radar_width() -> f64`](prelude::radar_width): Get current radar width.
- [`radar_min_distance() -> f64`](prelude::radar_min_distance): Get current minimum distance filter.
- [`radar_max_distance() -> f64`](prelude::radar_max_distance): Get current maximum distance filter.

## Radio

The radio can be used to send or receive a single value per tick. There are 10
channels available (0 to 9), shared between all teams.

- [`set_radio_channel(channel: usize)`](prelude::set_radio_channel): Change the radio channel. Takes effect next tick.
- [`get_radio_channel() -> usize`](prelude::get_radio_channel): Get the radio channel.
- [`send(data: f64)`](prelude::send): Send a message on a channel.
- [`receive() -> f64`](prelude::receive): Receive a message from the channel. The message with the strongest signal is returned.

## Scalar Math

- [`PI`](prelude::PI), [`TAU`](prelude::TAU): Constants.
- [`x.abs()`](f64::abs): Absolute value.
- [`x.sqrt()`](f64::sqrt): Square root.
- [`x.sin()`](f64::sin), [`x.cos()`](f64::cos), [`x.tan()`](f64::tan): Trigonometry.

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

Clicking on a ship in the UI displays status information and graphics
indicating its acceleration, radar cone, etc. You can add to this with the
functions below.

- [`debug!(...)`](prelude::debug!): Add status text.
- [`debug_line(v0: Vec2, v1: Vec2, color: u32)`](prelude::debug_line): Draw a line.
- [`debug_triangle(center: Vec2, radius: f64, color: u32)`](prelude::debug_triangle): Draw a triangle.
- [`debug_square(center: Vec2, radius: f64, color: u32)`](prelude::debug_square): Draw a square.
- [`debug_diamond(center: Vec2, radius: f64, color: u32)`](prelude::debug_diamond): Draw a diamond.
- [`debug_polygon(center: Vec2, radius: f64, sides: i32, angle: f64, color: u32)`](prelude::debug_polygon): Draw a regular polygon.

Entering debug mode by pressing the 'g' key also displays debug graphics from all ships.

## Miscellaneous

- [`current_tick() → f64`](prelude::current_tick): Returns the number of ticks elapsed since the simulation started.
- [`current_time() → f64`](prelude::current_time): Returns the number of seconds elapsed since the simulation started.
- [`angle_diff(a: f64, b: f64) → f64`](prelude::angle_diff): Returns the shortest (possibly negative) distance between two angles.
- [`rand(low: f64, high: f64) → f64`](prelude::rand): Get a random number.
- [`seed() → u128`](prelude::seed): Returns a seed useful for initializing a random number generator.

## Ship Classes

- [`Fighter`](prelude::Class::Fighter): Small, fast, and lightly armored.
  - Health: 100
  - Acceleration: Forward/Reverse: 60 m/s², Lateral: 30 m/s², Angular: 2π rad/s²
  - Gun 0: Damage: 7, Speed: 1000 m/s, Reload: 66ms
  - Missile 0: Reload: 5s
- [`Frigate`](prelude::Class::Frigate): Medium size with heavy armor and an extremely powerful main gun.
  - Health: 10000
  - Acceleration: Forward/Reverse: 10 m/s², Lateral: 5 m/s², Angular: π/4 rad/s²
  - Gun 0: Damage: 1000, Speed: 4000 m/s, Reload: 1 second
  - Gun 1: Damage: 7, Speed: 1000 m/s, Reload: 66ms, Turreted
  - Gun 2: Damage: 7, Speed: 1000 m/s, Reload: 66ms, Turreted
  - Missile 0: Reload: 2s
- [`Cruiser`](prelude::Class::Cruiser): Large, slow, and heavily armored. Rapid fire missile launchers and devastating torpedos.
  - Health: 20000
  - Acceleration: Forward/Reverse: 5 m/s², Lateral: 2.5 m/s², Angular: π/8 rad/s²
  - Gun 0: Damage: 3×7, Speed: 1000 m/s, Reload: 0.4s, Turreted
  - Missile 0: Reload: 1.2s
  - Missile 1: Reload: 1.2s
  - Torpedo 2: Reload: 3s
- [`Missile`](prelude::Class::Missile): Highly maneuverable but unarmored. Explodes on contact or after an [`explode`](prelude::explode) call.
  - Health: 20
  - Acceleration: Forward/Reverse: 200 m/s², Lateral: 50 m/s², Angular: 4π rad/s²
  - Warhead: 20×15
- [`Torpedo`](prelude::Class::Torpedo): Better armor, larger warhead, but less maneuverable than a missile. Explodes on contact or after an [`explode`](prelude::explode) call.
  - Health: 100
  - Acceleration: Forward/Reverse: 70 m/s², Lateral: 20 m/s², Angular: 4π rad/s²
  - Warhead: 100×50
