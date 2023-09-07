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
- [`health() → f64`](prelude::health): Get the current health.
- [`fuel() → f64`](prelude::fuel): Get the current fuel (delta-v).

Engine control:

- [`accelerate(acceleration: Vec2)`](prelude::accelerate): Accelerate the ship. Units are m/s².
- [`turn(speed: f64)`](prelude::turn): Rotate the ship. Unit is radians/s.
- [`torque(acceleration: f64)`](prelude::torque): Angular acceleration. Unit is radians/s².

Engine limits:

- [`max_forward_acceleration() -> f64`](prelude::max_forward_acceleration): Maximum forward acceleration.
- [`max_backward_acceleration() -> f64`](prelude::max_backward_acceleration): Maximum backward acceleration.
- [`max_lateral_acceleration() -> f64`](prelude::max_lateral_acceleration): Maximum lateral acceleration.
- [`max_angular_acceleration() -> f64`](prelude::max_angular_acceleration): Maximum angular acceleration.

## Weapons

- [`fire(index: usize)`](prelude::fire): Fire a weapon (gun or missile).
- [`aim(index: usize, angle: f64)`](prelude::aim): Aim a weapon (for weapons on a turret).
- [`reload_ticks(index: usize) -> u32`](prelude::reload_ticks): Number of ticks until the weapon is ready to fire.
- [`explode()`](prelude::explode): Self-destruct.

## Radar

Radar in Oort is modeled as a beam that can be pointed in any direction and
which has a beam width between 1/3600 to 1/16 of a circle. Enemy ships
illuminated by this beam reflect an amount of energy proportional to their
radar cross section (larger for larger ships). The radar can return one
contact per tick. Any changes to radar heading/width/filtering take effect on
the next tick.

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

Electronic Counter Measures (ECM):

The goal of ECM is to make enemy radar less effective. For ECM to work, the enemy radar must be
pointed towards your ship, and your ship's radar must be pointed at the enemy. Your radar will not
return contacts while ECM is enabled.

- [`EcmMode`](prelude::EcmMode):
  - [`EcmMode::None`](prelude::EcmMode::None): No ECM, radar will operate normally.
  - [`EcmMode::Noise`](prelude::EcmMode::Noise): Decrease the enemy radar's signal to noise ratio,
    making it more difficult to detect targets and reducing accuracy of returned contacts.
- [`radar_set_ecm_mode(mode: EcmMode)`](prelude::set_radar_ecm_mode): Set the ECM mode.

Retrieving current state:

- [`radar_heading() -> f64`](prelude::radar_heading): Get current radar heading.
- [`radar_width() -> f64`](prelude::radar_width): Get current radar width.
- [`radar_min_distance() -> f64`](prelude::radar_min_distance): Get current minimum distance filter.
- [`radar_max_distance() -> f64`](prelude::radar_max_distance): Get current maximum distance filter.

## Radio

The radio can be used to send or receive a `[f64; 4]` message per tick. There are 10
channels available (0 to 9), shared between all teams.

- [`set_radio_channel(channel: usize)`](prelude::set_radio_channel): Change the radio channel. Takes effect next tick.
- [`get_radio_channel() -> usize`](prelude::get_radio_channel): Get the radio channel.
- [`send(data: [f64; 4])`](prelude::send): Send a message on a channel.
- [`receive() -> Option<[f64; 4]>`](prelude::receive): Receive a message from the channel. The message with the strongest signal is returned.
- [`send_bytes(data: &[u8])`](prelude::send_bytes): Send a message on a channel as bytes, the data will be zero-filled or truncated to a length of 32 bytes.
- [`receive_bytes() -> Option<[u8; 32]>`](prelude::receive_bytes): Just like receive, but insted the message will be returned as a byte array.
- [`select_radio(index: usize)`](prelude::select_radio): Select the radio to control with subsequent API calls. Frigates have 4 radios and cruisers have 8.

## Special Abilities

Some ship classes have a unique special ability. These abilities are activated for a certain time and then need to reload.

- [`activate_ability(ability: Ability)`](prelude::activate_ability): Activates a special ability.
- Available abilities:
  - [`Ability::Boost`](prelude::Ability::Boost): Fighter and missile only. Applies a 100 m/s² forward acceleration for 2s. Reloads in 10s.
  - [`Ability::ShapedCharge`](prelude::Ability::ShapedCharge): Missile only. [`explode()`][prelude::explode] will create a jet of shrapnel instead of a circle.
  - [`Ability::Decoy`](prelude::Ability::Decoy): Torpedo only. Mimics the radar signature of a Cruiser for 0.5s. Reloads in 10s.
  - [`Ability::Shield`](prelude::Ability::Shield): Cruiser only. Deflects damage for 1s. Reloads in 5s.

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
- [`v.length() → f64`](prelude::Vec2Extras::length): Length.
- [`v.normalize() → Vec2`](prelude::Vec2Extras::normalize): Normalize to a unit vector.
- [`v.rotate(angle: f64) → Vec2`](prelude::Vec2Extras::rotate): Rotate counter-clockwise.
- [`v.angle() → f64`](prelude::Vec2Extras::angle): Angle of a vector.
- [`v1.dot(v2: Vec2) → f64`](prelude::Vec2Extras::dot): Dot product.
- [`v1.distance(v2: Vec2) → f64`](prelude::Vec2Extras::distance): Distance between two points.

The entire [maths_rs](https://docs.rs/maths-rs/0.2.4/maths_rs/index.html) crate is also available.

## Debugging

Clicking on a ship in the UI displays status information and graphics
indicating its acceleration, radar cone, etc. You can add to this with the
functions below.

- [`debug!(...)`](prelude::debug!): Add status text.
- [`draw_line(v0: Vec2, v1: Vec2, color: u32)`](prelude::draw_line): Draw a line.
- [`draw_triangle(center: Vec2, radius: f64, color: u32)`](prelude::draw_triangle): Draw a triangle.
- [`draw_square(center: Vec2, radius: f64, color: u32)`](prelude::draw_square): Draw a square.
- [`draw_diamond(center: Vec2, radius: f64, color: u32)`](prelude::draw_diamond): Draw a diamond.
- [`draw_polygon(center: Vec2, radius: f64, sides: i32, angle: f64, color: u32)`](prelude::draw_polygon): Draw a regular polygon.
- [`draw_text!(topleft: Vec2, color: u32, ...)`](prelude::draw_text!): Draw text.

Entering debug mode by pressing the 'g' key also displays debug graphics from all ships.

## Miscellaneous

- [`current_tick() → f64`](prelude::current_tick): Returns the number of ticks elapsed since the simulation started.
- [`current_time() → f64`](prelude::current_time): Returns the number of seconds elapsed since the simulation started.
- [`angle_diff(a: f64, b: f64) → f64`](prelude::angle_diff): Returns the shortest (possibly negative) distance between two angles.
- [`rand(low: f64, high: f64) → f64`](prelude::rand): Get a random number.
- [`seed() → u128`](prelude::seed): Returns a seed useful for initializing a random number generator.
- [`scenario_name() → &str`](prelude::scenario_name): Returns the name of the current scenario.
- [`world_size() → f64`](prelude::world_size): Returns the width of the world in meters.
- [`id() → u32`](prelude::id): Returns a per-ship ID that is unique within a team.
- [`TICK_LENGTH`](prelude::TICK_LENGTH): Length of a single game tick in seconds. There are 60 ticks per second.

## Ship Classes

- [`Fighter`](prelude::Class::Fighter): Small, fast, and lightly armored.
  - Health: 100
  - Acceleration: Forward: 60 m/s², Lateral: 30 m/s², Reverse: 30 m/s², Angular: 2π rad/s²
  - Weapon 0: Gun, Speed: 1000 m/s, Reload: 66ms
  - Weapon 1: Missile, Reload: 5s
- [`Frigate`](prelude::Class::Frigate): Medium size with heavy armor and an extremely powerful main gun.
  - Health: 10000
  - Acceleration: Forward: 10 m/s², Lateral: 5 m/s², Reverse: 5 m/s², Angular: π/4 rad/s²
  - Weapon 0: Gun, Speed: 4000 m/s, Reload: 2 seconds
  - Weapon 1: Gun, Speed: 1000 m/s, Reload: 66ms, Turreted
  - Weapon 2: Gun, Speed: 1000 m/s, Reload: 66ms, Turreted
  - Weapon 3: Missile, Reload: 2s
- [`Cruiser`](prelude::Class::Cruiser): Large, slow, and heavily armored. Rapid fire missile launchers and devastating torpedos.
  - Health: 20000
  - Acceleration: Forward: 5 m/s², Lateral: 2.5 m/s², Reverse: 2.5 m/s², Angular: π/8 rad/s²
  - Weapon 0: Gun, Speed: 1000 m/s, Burst size: 6, Reload: 0.4s, Turreted
  - Weapon 1: Missile, Reload: 1.2s
  - Weapon 2: Missile, Reload: 1.2s
  - Weapon 3: Torpedo, Reload: 3s
- [`Missile`](prelude::Class::Missile): Highly maneuverable but unarmored. Explodes on contact or after an [`explode`](prelude::explode) call.
  - Health: 20
  - Fuel: 3600 m/s
  - Acceleration: Forward: 180 m/s², Reverse: 0 m/s², Lateral: 50 m/s², Angular: 4π rad/s²
- [`Torpedo`](prelude::Class::Torpedo): Better armor, larger warhead, but less maneuverable than a missile. Explodes on contact or after an [`explode`](prelude::explode) call.
  - Health: 100
  - Fuel: 4000 m/s
  - Acceleration: Forward: 70 m/s², Reverse: 0 m/s², Lateral: 20 m/s², Angular: 4π rad/s²
