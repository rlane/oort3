#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
mod vec;

#[allow(missing_docs)]
#[derive(Copy, Clone)]
pub enum SystemState {
    Class,
    Seed,
    PositionX,
    PositionY,
    VelocityX,
    VelocityY,
    Heading,
    AngularVelocity,

    AccelerateX,
    AccelerateY,
    Torque,

    Aim0,
    Aim1,
    Aim2,
    Aim3,

    Fire0,
    Fire1,
    Fire2,
    Fire3,

    Explode,

    RadarHeading,
    RadarWidth,
    RadarContactFound,
    RadarContactClass,
    RadarContactPositionX,
    RadarContactPositionY,
    RadarContactVelocityX,
    RadarContactVelocityY,

    DebugTextPointer,
    DebugTextLength,

    MaxAccelerationX,
    MaxAccelerationY,
    MaxAngularAcceleration,

    RadioChannel,
    RadioSend,
    RadioReceive,

    DebugLinesPointer,
    DebugLinesLength,

    RadarMinDistance,
    RadarMaxDistance,

    CurrentTick,
    Energy,

    ActivateAbility,

    RadioData0,
    RadioData1,
    RadioData2,
    RadioData3,

    Size,
    MaxSize = 128,
}

/// Identifiers for each class of ship.
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Class {
    Fighter,
    Frigate,
    Cruiser,
    Asteroid,
    Target,
    Missile,
    Torpedo,
    Unknown,
}

impl Class {
    #[allow(missing_docs)]
    pub fn from_f64(v: f64) -> Class {
        match v as u32 {
            0 => Class::Fighter,
            1 => Class::Frigate,
            2 => Class::Cruiser,
            3 => Class::Asteroid,
            4 => Class::Target,
            5 => Class::Missile,
            6 => Class::Torpedo,
            _ => Class::Unknown,
        }
    }
}

/// Special abilities available to different ship classes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Ability {
    /// No-op.
    None,
    /// Fighter only. Applies a 100 m/s² forward acceleration for 2s. Reloads in 10s.
    Boost,
    /// Missile only. `explode()` will create a jet of shrapnel instead of a circle.
    ShapedCharge,
}

#[allow(missing_docs)]
#[derive(Default, Clone)]
pub struct Line {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub color: u32,
}

/// Message sent and received on the radio.
pub type Message = [f64; 4];

// Public for fuzzer.
#[doc(hidden)]
pub mod sys {
    use super::SystemState;

    #[no_mangle]
    pub static mut SYSTEM_STATE: [f64; SystemState::MaxSize as usize] =
        [0.0; SystemState::MaxSize as usize];

    pub fn read_system_state(index: SystemState) -> f64 {
        unsafe { SYSTEM_STATE[index as usize] }
    }

    pub fn write_system_state(index: SystemState, value: f64) {
        unsafe {
            SYSTEM_STATE[index as usize] = value;
        }
    }
}

mod math {
    pub use std::f64::consts::{PI, TAU};

    /// Returns the smallest rotation between angles `a` and `b`.
    ///
    /// A positive result is a counter-clockwise rotation and negative is clockwise.
    pub fn angle_diff(a: f64, b: f64) -> f64 {
        let c = (b - a).rem_euclid(TAU);
        if c > PI {
            c - TAU
        } else {
            c
        }
    }
}

mod rng {
    use super::api::seed;

    static mut RNG: Option<oorandom::Rand64> = None;

    fn rng() -> &'static mut oorandom::Rand64 {
        unsafe {
            if RNG.is_none() {
                RNG = Some(oorandom::Rand64::new(seed()));
            }
            RNG.as_mut().unwrap()
        }
    }

    /// Returns a random number between `low` and `high`.
    pub fn rand(low: f64, high: f64) -> f64 {
        rng().rand_float() * (high - low) + low
    }
}

mod api {
    use super::sys::{read_system_state, write_system_state};
    use super::{Ability, Class, SystemState};
    use crate::{vec::*, Message};

    /// The time between each simulation tick.
    pub const TICK_LENGTH: f64 = 1.0 / 60.0;

    /// Returns the ship [`Class`] (Fighter, Cruiser, etc).
    pub fn class() -> Class {
        Class::from_f64(read_system_state(SystemState::Class))
    }

    /// Returns a random number useful for initializing a random number generator.
    pub fn seed() -> u128 {
        read_system_state(super::SystemState::Seed) as u128
    }

    /// Returns the current position (in meters).
    pub fn position() -> Vec2 {
        vec2(
            read_system_state(SystemState::PositionX),
            read_system_state(SystemState::PositionY),
        )
    }

    /// Returns the current velocity (in m/s).
    pub fn velocity() -> Vec2 {
        vec2(
            read_system_state(SystemState::VelocityX),
            read_system_state(SystemState::VelocityY),
        )
    }

    /// Returns the current heading (in radians).
    pub fn heading() -> f64 {
        read_system_state(SystemState::Heading)
    }

    /// Returns the current angular velocity (in radians/s).
    pub fn angular_velocity() -> f64 {
        read_system_state(SystemState::AngularVelocity)
    }

    /// Sets the linear acceleration for the next tick (in m/s²).
    pub fn accelerate(mut acceleration: Vec2) {
        acceleration = acceleration.rotate(-heading());
        let max = max_acceleration();
        if acceleration.x.abs() > max.x.abs() {
            acceleration *= max.x.abs() / acceleration.x.abs();
        }
        if acceleration.y.abs() > max.y.abs() {
            acceleration *= max.y.abs() / acceleration.y.abs();
        }
        write_system_state(SystemState::AccelerateX, acceleration.x);
        write_system_state(SystemState::AccelerateY, acceleration.y);
    }

    /// Sets the angular acceleration for the next tick (in radians/s²).
    pub fn torque(angular_acceleration: f64) {
        write_system_state(SystemState::Torque, angular_acceleration);
    }

    /// Aims a turreted weapon.
    ///
    /// `index` selects the weapon.
    /// `heading` is in radians.
    pub fn aim(index: usize, heading: f64) {
        let state_index = match index {
            0 => SystemState::Aim0,
            1 => SystemState::Aim1,
            2 => SystemState::Aim2,
            3 => SystemState::Aim3,
            _ => return,
        };
        write_system_state(state_index, heading - crate::api::heading());
    }

    /// Fires a weapon.
    ///
    /// `index` selects the weapon.
    pub fn fire(index: usize) {
        let state_index = match index {
            0 => SystemState::Fire0,
            1 => SystemState::Fire1,
            2 => SystemState::Fire2,
            3 => SystemState::Fire3,
            _ => return,
        };
        write_system_state(state_index, 1.0);
    }

    /// Self-destructs, producing a damaging explosion.
    ///
    /// This is commonly used by missiles.
    pub fn explode() {
        write_system_state(SystemState::Explode, 1.0);
    }

    /// Returns the heading the radar is pointed at.
    ///
    /// This is relative to the ship's heading.
    pub fn radar_heading() -> f64 {
        read_system_state(SystemState::RadarHeading) + heading()
    }

    /// Sets the heading to point the radar at.
    ///
    /// This is relative to the ship's heading.
    /// It takes effect next tick.
    pub fn set_radar_heading(heading: f64) {
        write_system_state(SystemState::RadarHeading, heading - crate::api::heading());
    }

    /// Returns the current radar width (in radians).
    ///
    /// This is the field of view of the radar.
    pub fn radar_width() -> f64 {
        read_system_state(SystemState::RadarWidth)
    }

    /// Sets the radar width (in radians).
    ///
    /// This is the field of view of the radar.
    /// It takes effect next tick.
    pub fn set_radar_width(width: f64) {
        write_system_state(SystemState::RadarWidth, width);
    }

    /// Sets the minimum distance filter of the radar (in meters).
    ///
    /// It takes effect next tick.
    pub fn radar_min_distance() -> f64 {
        read_system_state(SystemState::RadarMinDistance)
    }

    /// Gets the current minimum distance filter of the radar (in meters).
    pub fn set_radar_min_distance(dist: f64) {
        write_system_state(SystemState::RadarMinDistance, dist);
    }

    /// Sets the maximum distance filter of the radar (in meters).
    ///
    /// It takes effect next tick.
    pub fn radar_max_distance() -> f64 {
        read_system_state(SystemState::RadarMaxDistance)
    }

    /// Gets the current maximum distance filter of the radar (in meters).
    pub fn set_radar_max_distance(dist: f64) {
        write_system_state(SystemState::RadarMaxDistance, dist);
    }

    /// A radar contact.
    #[derive(Clone, Debug)]
    pub struct ScanResult {
        /// The contact's class.
        pub class: Class,
        /// The contact's approximate position.
        pub position: Vec2,
        /// The contact's approximate velocity.
        pub velocity: Vec2,
    }

    /// Returns the radar contact with the highest signal strength.
    pub fn scan() -> Option<ScanResult> {
        if read_system_state(SystemState::RadarContactFound) == 0.0 {
            return None;
        }
        Some(ScanResult {
            class: Class::from_f64(read_system_state(SystemState::RadarContactClass)),
            position: vec2(
                read_system_state(SystemState::RadarContactPositionX),
                read_system_state(SystemState::RadarContactPositionY),
            ),
            velocity: vec2(
                read_system_state(SystemState::RadarContactVelocityX),
                read_system_state(SystemState::RadarContactVelocityY),
            ),
        })
    }

    /// Sets the channel to send and receive radio transmissions on.
    ///
    /// Takes effect next tick.
    pub fn set_radio_channel(channel: usize) {
        write_system_state(SystemState::RadioChannel, channel as f64);
    }

    /// Gets the current radio channel.
    pub fn get_radio_channel() -> usize {
        read_system_state(SystemState::RadioChannel) as usize
    }

    /// Sends a radio message.
    ///
    /// The message will be received on the next tick.
    pub fn send(msg: Message) {
        write_system_state(SystemState::RadioSend, 1.0);
        write_system_state(SystemState::RadioData0, msg[0]);
        write_system_state(SystemState::RadioData1, msg[1]);
        write_system_state(SystemState::RadioData2, msg[2]);
        write_system_state(SystemState::RadioData3, msg[3]);
    }

    /// Returns the received radio message.
    pub fn receive() -> Option<Message> {
        if read_system_state(SystemState::RadioReceive) != 0.0 {
            Some([
                read_system_state(SystemState::RadioData0),
                read_system_state(SystemState::RadioData1),
                read_system_state(SystemState::RadioData2),
                read_system_state(SystemState::RadioData3),
            ])
        } else {
            None
        }
    }

    /// Returns the maximum linear acceleration (in m/s²).
    pub fn max_acceleration() -> Vec2 {
        vec2(
            read_system_state(SystemState::MaxAccelerationX),
            read_system_state(SystemState::MaxAccelerationY),
        )
    }

    /// Returns the maximum angular acceleration (in radians/s²).
    pub fn max_angular_acceleration() -> f64 {
        read_system_state(SystemState::MaxAngularAcceleration)
    }

    /// Returns the number of ticks elapsed since the simulation began.
    pub fn current_tick() -> u32 {
        read_system_state(SystemState::CurrentTick) as u32
    }

    /// Returns the number of seconds elapsed since the simulation began.
    pub fn current_time() -> f64 {
        read_system_state(SystemState::CurrentTick) * TICK_LENGTH
    }

    /// Returns the energy available to the ship (in Joules).
    pub fn energy() -> f64 {
        read_system_state(SystemState::Energy)
    }

    /// Activates a special ability.
    pub fn activate_ability(ability: Ability) {
        write_system_state(SystemState::ActivateAbility, ability as u32 as f64);
    }

    /// Returns the position of the target set by the scenario.
    /// Only used in tutorials.
    pub fn target() -> Vec2 {
        vec2(
            read_system_state(SystemState::RadarContactPositionX),
            read_system_state(SystemState::RadarContactPositionY),
        )
    }
}

#[doc(hidden)]
#[macro_use]
pub mod dbg {
    use super::Line;
    use crate::sys::write_system_state;
    use crate::vec::*;
    use std::f64::consts::TAU;

    static mut TEXT_BUFFER: String = String::new();
    static mut LINE_BUFFER: Vec<Line> = Vec::new();

    /// Adds text to be displayed when the ship is selected by clicking on it.
    ///
    /// Works just like [println!].
    #[macro_export]
    macro_rules! debug {
        ($($arg:tt)*) => {
            $crate::dbg::write(std::format_args!($($arg)*))
        };
    }

    #[allow(unused)]
    #[doc(hidden)]
    pub fn write(args: std::fmt::Arguments) {
        use std::fmt::Write;
        unsafe {
            let _ = std::fmt::write(&mut TEXT_BUFFER, args);
            TEXT_BUFFER.push('\n');
        }
    }

    /// Draws a line visible in debug mode.
    ///
    /// `a` and `b` are positions in world coordinates.
    /// `color` is 24-bit RGB.
    pub fn debug_line(a: Vec2, b: Vec2, color: u32) {
        unsafe {
            LINE_BUFFER.push(Line {
                x0: a.x,
                y0: a.y,
                x1: b.x,
                y1: b.y,
                color,
            });
        }
    }

    /// Draws a regular polygon visible in debug mode.
    ///
    /// `center` is a position in world coordinates.
    /// `color` is 24-bit RGB.
    pub fn debug_polygon(center: Vec2, radius: f64, sides: i32, angle: f64, color: u32) {
        let mut angle = angle;
        let delta_angle = TAU / sides as f64;
        let p = vec2(radius, 0.0);
        for _ in 0..sides {
            debug_line(
                center + p.rotate(angle),
                center + p.rotate(angle + delta_angle),
                color,
            );
            angle += delta_angle;
        }
    }

    /// Draws a triangle visible in debug mode.
    ///
    /// `center` is a position in world coordinates.
    /// `color` is 24-bit RGB.
    pub fn debug_triangle(center: Vec2, radius: f64, color: u32) {
        debug_polygon(center, radius, 3, TAU / 4.0, color);
    }

    /// Draws a triangle visible in debug mode.
    ///
    /// `center` is a position in world coordinates.
    /// `color` is 24-bit RGB.
    pub fn debug_square(center: Vec2, radius: f64, color: u32) {
        debug_polygon(center, radius, 4, TAU / 8.0, color);
    }

    /// Draws a triangle visible in debug mode.
    ///
    /// `center` is a position in world coordinates.
    /// `color` is 24-bit RGB.
    pub fn debug_diamond(center: Vec2, radius: f64, color: u32) {
        debug_polygon(center, radius, 4, 0.0, color);
    }

    #[doc(hidden)]
    pub fn update() {
        {
            let slice = unsafe { TEXT_BUFFER.as_bytes() };
            write_system_state(
                super::SystemState::DebugTextPointer,
                slice.as_ptr() as u32 as f64,
            );
            write_system_state(
                super::SystemState::DebugTextLength,
                slice.len() as u32 as f64,
            );
        }
        {
            let slice = unsafe { LINE_BUFFER.as_slice() };
            write_system_state(
                super::SystemState::DebugLinesPointer,
                slice.as_ptr() as u32 as f64,
            );
            write_system_state(
                super::SystemState::DebugLinesLength,
                slice.len() as u32 as f64,
            );
        }
    }

    #[doc(hidden)]
    pub fn reset() {
        unsafe {
            TEXT_BUFFER.clear();
            LINE_BUFFER.clear();
        }
    }
}

mod deprecated {
    use super::api::*;
    use super::sys::write_system_state;
    use super::SystemState;

    /// TODO Remove this.
    #[deprecated]
    pub fn aim_gun(index: usize, heading: f64) {
        aim(index, heading);
    }

    /// TODO Remove this.
    #[deprecated]
    pub fn fire_gun(index: usize) {
        fire(index);
    }

    /// TODO Remove this.
    #[deprecated]
    pub fn launch_missile(index: usize, _unused: f64) {
        use super::Class::*;
        let state_index = match (class(), index) {
            (Fighter, 0) => SystemState::Fire1,

            (Frigate, 0) => SystemState::Fire3,

            (Cruiser, 0) => SystemState::Fire1,
            (Cruiser, 1) => SystemState::Fire2,
            (Cruiser, 2) => SystemState::Fire3,

            _ => return,
        };
        write_system_state(state_index, 1.0);
    }

    /// TODO Remove this.
    #[deprecated]
    pub fn orders() -> f64 {
        0.0
    }
}

/// All APIs.
pub mod prelude {
    #[doc(inline)]
    pub use super::api::*;
    #[doc(inline)]
    pub use super::dbg::*;
    #[doc(hidden)]
    pub use super::deprecated::*;
    #[doc(inline)]
    pub use super::math::*;
    #[doc(inline)]
    pub use super::rng::*;
    #[doc(inline)]
    pub use super::vec::*;
    #[doc(inline)]
    pub use super::{Ability, Class, Message};
    #[doc(inline)]
    pub use crate::debug;
}
