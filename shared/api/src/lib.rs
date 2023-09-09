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

    MaxForwardAcceleration,
    MaxLateralAcceleration,
    MaxAngularAcceleration,

    DebugLinesPointer,
    DebugLinesLength,

    RadarMinDistance,
    RadarMaxDistance,

    CurrentTick,
    MaxBackwardAcceleration,

    ActivateAbility,

    Radio0Channel, // TODO collapse into command word
    Radio0Send,
    Radio0Receive,
    Radio0Data0,
    Radio0Data1,
    Radio0Data2,
    Radio0Data3,

    Radio1Channel,
    Radio1Send,
    Radio1Receive,
    Radio1Data0,
    Radio1Data1,
    Radio1Data2,
    Radio1Data3,

    Radio2Channel,
    Radio2Send,
    Radio2Receive,
    Radio2Data0,
    Radio2Data1,
    Radio2Data2,
    Radio2Data3,

    Radio3Channel,
    Radio3Send,
    Radio3Receive,
    Radio3Data0,
    Radio3Data1,
    Radio3Data2,
    Radio3Data3,

    Radio4Channel,
    Radio4Send,
    Radio4Receive,
    Radio4Data0,
    Radio4Data1,
    Radio4Data2,
    Radio4Data3,

    Radio5Channel,
    Radio5Send,
    Radio5Receive,
    Radio5Data0,
    Radio5Data1,
    Radio5Data2,
    Radio5Data3,

    Radio6Channel,
    Radio6Send,
    Radio6Receive,
    Radio6Data0,
    Radio6Data1,
    Radio6Data2,
    Radio6Data3,

    Radio7Channel,
    Radio7Send,
    Radio7Receive,
    Radio7Data0,
    Radio7Data1,
    Radio7Data2,
    Radio7Data3,

    // TODO not part of interface
    SelectedRadio,

    DrawnTextPointer,
    DrawnTextLength,

    RadarEcmMode,

    Health,
    Fuel,

    RadarContactRssi,
    RadarContactSnr,

    ReloadTicks0,
    ReloadTicks1,
    ReloadTicks2,
    ReloadTicks3,

    Id,

    Size,
    MaxSize = 128,
}

#[allow(missing_docs)]
pub const MAX_ENVIRONMENT_SIZE: usize = 1024;

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

/// List of active abilities for an entity.
#[repr(transparent)]
pub struct ActiveAbilities(pub u64);

impl ActiveAbilities {
    /// Activate an ability.
    pub fn set_ability(&mut self, ability: Ability) {
        let mut mask = 0u64;
        mask ^= 1u64 << (ability as u64);
        self.0 |= mask;
    }
    /// Deactivate an ability.
    pub fn unset_ability(&mut self, ability: Ability) {
        let mut mask = 0u64;
        mask ^= 1u64 << (ability as u64);
        self.0 &= !mask;
    }
    /// Get whether an ability is active.
    pub fn get_ability(&self, ability: Ability) -> bool {
        (self.0 & 1 << (ability as u64)) >> (ability as u64) != 0
    }
    /// Unset all abilities
    pub fn clear(&mut self) {
        self.0 = 0
    }
    /// Invert set and unset abilities.
    pub fn invert(&mut self) {
        self.0 = !self.0
    }
    /// Iterator over active abilities
    pub fn active_iter(&self) -> impl Iterator<Item = Ability> + core::fmt::Debug + Clone + '_ {
        ABILITIES
            .iter()
            .flat_map(|&ability| self.get_ability(ability).then_some(ability))
    }
}

/// Special abilities available to different ship classes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Ability {
    /// No-op.
    #[doc(hidden)]
    None,
    /// Fighter and missile only. Applies a 100 m/s² forward acceleration for 2s. Reloads in 10s.
    Boost,
    /// Deprecated
    #[doc(hidden)]
    ShapedCharge,
    /// Torpedo only. Mimics the radar signature of a Cruiser for 0.5s. Reloads in 10s.
    Decoy,
    /// Cruiser only. Deflects projectiles for 1s. Reloads in 5s.
    Shield,
}

/// Array of all ability types.
pub const ABILITIES: &[Ability] = &[Ability::Boost, Ability::Decoy, Ability::Shield];

/// Electronic Counter Measures (ECM) modes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EcmMode {
    /// No ECM, radar will work normally.
    None,
    /// Affected enemy radars will have a lower signal-to-noise ratio, making
    /// it harder to detect and track targets.
    Noise,
}

impl From<f64> for EcmMode {
    fn from(x: f64) -> Self {
        match x as u32 {
            0 => EcmMode::None,
            1 => EcmMode::Noise,
            _ => EcmMode::None,
        }
    }
}

#[doc(hidden)]
#[derive(Default, Clone)]
pub struct Line {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub color: u32,
}

#[doc(hidden)]
#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Text {
    pub x: f64,
    pub y: f64,
    pub color: u32,
    pub length: u8,
    pub text: [u8; 11],
}

/// Message sent and received on the radio.
pub type Message = [f64; 4];

// Public for fuzzer.
#[doc(hidden)]
pub mod sys {
    use crate::MAX_ENVIRONMENT_SIZE;

    use super::SystemState;

    // TODO crashes rust-analyzer
    #[no_mangle]
    pub static mut SYSTEM_STATE: [u64; SystemState::MaxSize as usize] =
        [0; SystemState::MaxSize as usize];

    pub fn read_system_state_u64(index: SystemState) -> u64 {
        let system_state = unsafe { &SYSTEM_STATE };
        system_state[index as usize]
    }

    pub fn write_system_state_u64(index: SystemState, value: u64) {
        let system_state = unsafe { &mut SYSTEM_STATE };
        system_state[index as usize] = value;
    }

    pub fn read_system_state(index: SystemState) -> f64 {
        f64::from_bits(read_system_state_u64(index))
    }

    pub fn write_system_state(index: SystemState, value: f64) {
        write_system_state_u64(index, value.to_bits())
    }

    #[no_mangle]
    pub static mut ENVIRONMENT: [u8; MAX_ENVIRONMENT_SIZE] = [0; MAX_ENVIRONMENT_SIZE];

    pub fn read_environment() -> &'static str {
        // Format is key=value\nkey=value\n... ending with a null byte.
        let environment = unsafe { &ENVIRONMENT };
        let n = environment
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(environment.len());
        std::str::from_utf8(&environment[..n]).expect("Failed to convert environment to string")
    }

    pub fn getenv(key: &str) -> Option<&'static str> {
        let environment = read_environment();
        for line in environment.lines() {
            let mut parts = line.splitn(2, '=');
            if let Some(k) = parts.next() {
                if k == key {
                    return parts.next();
                }
            }
        }
        None
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
    fn rng() -> &'static mut oorandom::Rand64 {
        let rng_state = unsafe { super::rng_state::get() };
        &mut rng_state.rng
    }

    /// Returns a random number between `low` and `high`.
    pub fn rand(low: f64, high: f64) -> f64 {
        rng().rand_float() * (high - low) + low
    }
}

#[doc(hidden)]
pub mod rng_state {
    #[derive(Clone)]
    pub struct RngState {
        pub rng: oorandom::Rand64,
    }

    static mut RNG_STATE: Option<RngState> = None;

    impl RngState {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {
                rng: oorandom::Rand64::new(super::api::seed()),
            }
        }
    }

    pub unsafe fn get() -> &'static mut RngState {
        RNG_STATE.as_mut().unwrap()
    }

    pub unsafe fn set(s: RngState) {
        RNG_STATE = Some(s)
    }
}

mod api {
    use super::sys::{read_system_state, write_system_state};
    use super::{Ability, Class, EcmMode, SystemState};
    use crate::sys::{read_system_state_u64, write_system_state_u64};
    use crate::{vec::*, ActiveAbilities, Message};

    /// The time between each simulation tick.
    pub const TICK_LENGTH: f64 = 1.0 / 60.0;

    /// Returns a per-ship ID that is unique within a team.
    pub fn id() -> u32 {
        read_system_state(SystemState::Id) as u32
    }

    /// Returns the ship [`Class`] (Fighter, Cruiser, etc).
    pub fn class() -> Class {
        Class::from_f64(read_system_state(SystemState::Class))
    }

    /// Returns a random number useful for initializing a random number generator.
    pub fn seed() -> u128 {
        read_system_state(super::SystemState::Seed) as u128
    }

    /// Returns the scenario name.
    pub fn scenario_name() -> &'static str {
        super::sys::getenv("SCENARIO_NAME").unwrap_or("unknown")
    }

    /// Returns the world size in meters.
    pub fn world_size() -> f64 {
        super::sys::getenv("WORLD_SIZE")
            .unwrap_or("0.0")
            .parse()
            .unwrap_or(0.0)
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
        if acceleration.x > max_forward_acceleration() {
            acceleration *= max_forward_acceleration() / acceleration.x;
        }
        if acceleration.x < -max_backward_acceleration() {
            acceleration *= max_backward_acceleration() / -acceleration.x;
        }
        if acceleration.y.abs() > max_lateral_acceleration() {
            acceleration *= max_lateral_acceleration() / acceleration.y.abs();
        }
        write_system_state(SystemState::AccelerateX, acceleration.x);
        write_system_state(SystemState::AccelerateY, acceleration.y);
    }

    /// Rotates the ship at the given speed (in radians/s).
    ///
    /// Internally this uses `torque()`. Reaching the commanded speed takes time.
    pub fn turn(speed: f64) {
        let max = max_angular_acceleration() * 0.2;
        torque((speed.clamp(-max, max) - angular_velocity()).signum() * max_angular_acceleration());
    }

    /// Sets the angular acceleration for the next tick (in radians/s²).
    ///
    /// This is lower-level than turn() and can be used to turn faster.
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
        write_system_state(state_index, heading);
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

    /// Returns the number of ticks until a weapon is ready to fire.
    ///
    /// `index` selects the weapon. Returns 0 if the weapon is ready.
    pub fn reload_ticks(index: usize) -> u32 {
        let state_index = match index {
            0 => SystemState::ReloadTicks0,
            1 => SystemState::ReloadTicks1,
            2 => SystemState::ReloadTicks2,
            3 => SystemState::ReloadTicks3,
            _ => return 0,
        };
        read_system_state(state_index) as u32
    }

    /// Self-destructs, producing a damaging explosion.
    ///
    /// This is commonly used by missiles.
    pub fn explode() {
        write_system_state(SystemState::Explode, 1.0);
    }

    /// Returns the current health.
    pub fn health() -> f64 {
        read_system_state(SystemState::Health)
    }

    /// Returns the current fuel (delta-v).
    pub fn fuel() -> f64 {
        read_system_state(SystemState::Fuel)
    }

    /// Returns the heading the radar is pointed at.
    pub fn radar_heading() -> f64 {
        read_system_state(SystemState::RadarHeading)
    }

    /// Sets the heading to point the radar at.
    ///
    /// It takes effect next tick.
    pub fn set_radar_heading(heading: f64) {
        write_system_state(SystemState::RadarHeading, heading);
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

    /// Gets the Electronic Counter Measures (ECM) mode.
    pub fn radar_ecm_mode() -> EcmMode {
        read_system_state(SystemState::RadarEcmMode).into()
    }

    /// Sets the Electronic Counter Measures (ECM) mode.
    pub fn set_radar_ecm_mode(mode: EcmMode) {
        write_system_state(SystemState::RadarEcmMode, mode as u32 as f64);
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
        /// The received signal strength measured in dBm.
        pub rssi: f64,
        /// The signal-to-noise ratio measured in dB.
        pub snr: f64,
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
            rssi: read_system_state(SystemState::RadarContactRssi),
            snr: read_system_state(SystemState::RadarContactSnr),
        })
    }

    #[doc(hidden)]
    pub mod radio_internal {
        use super::SystemState;

        pub const MAX_RADIOS: usize = 8;

        pub struct RadioIndices {
            pub channel: SystemState,
            pub send: SystemState,
            pub receive: SystemState,
            pub data: [SystemState; 4],
        }

        pub fn radio_indices(sel: usize) -> RadioIndices {
            assert!(sel < MAX_RADIOS);
            let stride = 7;
            let offset = stride * sel;
            let add_offset =
                |x| unsafe { ::std::mem::transmute::<u8, SystemState>((x as u8) + offset as u8) };
            RadioIndices {
                channel: add_offset(SystemState::Radio0Channel),
                send: add_offset(SystemState::Radio0Send),
                receive: add_offset(SystemState::Radio0Receive),
                data: [
                    add_offset(SystemState::Radio0Data0),
                    add_offset(SystemState::Radio0Data1),
                    add_offset(SystemState::Radio0Data2),
                    add_offset(SystemState::Radio0Data3),
                ],
            }
        }
    }

    /// Select the radio to control with subsequent API calls.
    pub fn select_radio(index: usize) {
        let index = index.clamp(0, radio_internal::MAX_RADIOS - 1);
        write_system_state(SystemState::SelectedRadio, index as f64);
    }

    /// Sets the channel to send and receive radio transmissions on.
    ///
    /// Takes effect next tick.
    pub fn set_radio_channel(channel: usize) {
        write_system_state(
            radio_internal::radio_indices(read_system_state(SystemState::SelectedRadio) as usize)
                .channel,
            channel as f64,
        );
    }

    /// Gets the current radio channel.
    pub fn get_radio_channel() -> usize {
        read_system_state(
            radio_internal::radio_indices(read_system_state(SystemState::SelectedRadio) as usize)
                .channel,
        ) as usize
    }

    /// Sends a radio message.
    ///
    /// The message will be received on the next tick.
    ///
    /// If you want to send arbitrary data, consider using [`send_bytes`] instead.
    pub fn send(msg: Message) {
        let idxs =
            radio_internal::radio_indices(read_system_state(SystemState::SelectedRadio) as usize);
        write_system_state(idxs.send, 1.0);
        write_system_state(idxs.data[0], msg[0]);
        write_system_state(idxs.data[1], msg[1]);
        write_system_state(idxs.data[2], msg[2]);
        write_system_state(idxs.data[3], msg[3]);
    }

    /// Returns the received radio message.
    pub fn receive() -> Option<Message> {
        let idxs =
            radio_internal::radio_indices(read_system_state(SystemState::SelectedRadio) as usize);
        if read_system_state(idxs.receive) != 0.0 {
            Some([
                read_system_state(idxs.data[0]),
                read_system_state(idxs.data[1]),
                read_system_state(idxs.data[2]),
                read_system_state(idxs.data[3]),
            ])
        } else {
            None
        }
    }

    /// Sends a radio message.
    /// The message will be zero-filled or truncated to be 32 bytes long.
    ///
    /// The message will be received on the next tick.
    ///
    /// If you only want to send [`f64`]s consider using [`send`] instead.
    pub fn send_bytes(msg: &[u8]) {
        let mut bytes = [[0; 8]; 4];
        bytes
            .iter_mut()
            .flatten()
            .zip(msg)
            .for_each(|(b, m)| *b = *m);

        let idxs = radio_internal::radio_indices(
            read_system_state_u64(SystemState::SelectedRadio) as usize
        );
        write_system_state(idxs.send, 1.0);
        write_system_state_u64(idxs.data[0], u64::from_ne_bytes(bytes[0]));
        write_system_state_u64(idxs.data[1], u64::from_ne_bytes(bytes[1]));
        write_system_state_u64(idxs.data[2], u64::from_ne_bytes(bytes[2]));
        write_system_state_u64(idxs.data[3], u64::from_ne_bytes(bytes[3]));
    }

    /// Returns the received radio message.
    pub fn receive_bytes() -> Option<[u8; 32]> {
        let idxs = radio_internal::radio_indices(
            read_system_state_u64(SystemState::SelectedRadio) as usize
        );
        if read_system_state(idxs.receive) != 0.0 {
            let mut bytes = [0; 32];
            bytes[0..8].copy_from_slice(&read_system_state_u64(idxs.data[0]).to_ne_bytes());
            bytes[8..16].copy_from_slice(&read_system_state_u64(idxs.data[1]).to_ne_bytes());
            bytes[16..24].copy_from_slice(&read_system_state_u64(idxs.data[2]).to_ne_bytes());
            bytes[24..32].copy_from_slice(&read_system_state_u64(idxs.data[3]).to_ne_bytes());
            Some(bytes)
        } else {
            None
        }
    }

    /// Returns the maximum linear acceleration (in m/s²).
    #[deprecated]
    pub fn max_acceleration() -> Vec2 {
        vec2(
            read_system_state(SystemState::MaxForwardAcceleration),
            read_system_state(SystemState::MaxBackwardAcceleration),
        )
    }

    /// Returns the maximum forward acceleration (in m/s²).
    pub fn max_forward_acceleration() -> f64 {
        read_system_state(SystemState::MaxForwardAcceleration)
    }

    /// Returns the maximum backward acceleration (in m/s²).
    pub fn max_backward_acceleration() -> f64 {
        read_system_state(SystemState::MaxBackwardAcceleration)
    }

    /// Returns the maximum lateral acceleration (in m/s²).
    pub fn max_lateral_acceleration() -> f64 {
        read_system_state(SystemState::MaxLateralAcceleration)
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

    /// Activates a special ability.
    pub fn activate_ability(ability: Ability) {
        let mut active_abilities =
            ActiveAbilities(read_system_state_u64(SystemState::ActivateAbility));
        active_abilities.set_ability(ability);
        write_system_state_u64(SystemState::ActivateAbility, active_abilities.0);
    }

    /// Deactivates a special ability.
    pub fn deactivate_ability(ability: Ability) {
        let mut active_abilities =
            ActiveAbilities(read_system_state_u64(SystemState::ActivateAbility));
        active_abilities.unset_ability(ability);
        write_system_state_u64(SystemState::ActivateAbility, active_abilities.0);
    }

    /// Get a copy of the active abilities. Useful for querying which abilities are currently active.
    pub fn active_abilities() -> ActiveAbilities {
        ActiveAbilities(read_system_state_u64(SystemState::ActivateAbility))
    }

    /// Returns the position of the target set by the scenario.
    /// Only used in tutorials.
    pub fn target() -> Vec2 {
        vec2(
            read_system_state(SystemState::RadarContactPositionX),
            read_system_state(SystemState::RadarContactPositionY),
        )
    }

    /// Returns the velocity of the target set by the scenario.
    /// Only used in tutorials.
    pub fn target_velocity() -> Vec2 {
        vec2(
            read_system_state(SystemState::RadarContactVelocityX),
            read_system_state(SystemState::RadarContactVelocityY),
        )
    }
}

#[doc(hidden)]
#[macro_use]
pub mod dbg {
    use super::{Line, Text};
    use crate::sys::write_system_state;
    use crate::vec::*;
    use std::f64::consts::TAU;

    static mut TEXT_BUFFER: String = String::new();
    static mut LINE_BUFFER: Vec<Line> = Vec::new();
    static mut DRAWN_TEXT_BUFFER: Vec<Text> = Vec::new();

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
        let buf = unsafe { &mut TEXT_BUFFER };
        let _ = std::fmt::write(buf, args);
        buf.push('\n');
    }

    /// Creates a 24-bit RGB color from the arguments.
    pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
        let r = r as u32;
        let g = g as u32;
        let b = b as u32;
        r << 16 | g << 8 | b
    }

    /// Draws a line visible in debug mode.
    ///
    /// `a` and `b` are positions in world coordinates.
    /// `color` is 24-bit RGB.
    ///
    /// Up to 1024 lines can be drawn per ship, per tick. This quota is also consumed
    /// by the various shape drawing functions.
    pub fn draw_line(a: Vec2, b: Vec2, color: u32) {
        let buf = unsafe { &mut LINE_BUFFER };
        buf.push(Line {
            x0: a.x,
            y0: a.y,
            x1: b.x,
            y1: b.y,
            color,
        });
    }

    #[deprecated]
    #[doc(hidden)]
    pub fn debug_line(a: Vec2, b: Vec2, color: u32) {
        draw_line(a, b, color)
    }

    /// Draws a regular polygon visible in debug mode.
    ///
    /// `center` is a position in world coordinates.
    /// `color` is 24-bit RGB.
    pub fn draw_polygon(center: Vec2, radius: f64, sides: i32, angle: f64, color: u32) {
        let mut angle = angle;
        let delta_angle = TAU / sides as f64;
        let p = vec2(radius, 0.0);
        for _ in 0..sides {
            draw_line(
                center + p.rotate(angle),
                center + p.rotate(angle + delta_angle),
                color,
            );
            angle += delta_angle;
        }
    }

    #[deprecated]
    #[doc(hidden)]
    pub fn debug_polygon(center: Vec2, radius: f64, sides: i32, angle: f64, color: u32) {
        draw_polygon(center, radius, sides, angle, color)
    }

    /// Draws a triangle visible in debug mode.
    ///
    /// `center` is a position in world coordinates.
    /// `color` is 24-bit RGB.
    pub fn draw_triangle(center: Vec2, radius: f64, color: u32) {
        draw_polygon(center, radius, 3, TAU / 4.0, color);
    }

    #[deprecated]
    #[doc(hidden)]
    pub fn debug_triangle(center: Vec2, radius: f64, color: u32) {
        draw_triangle(center, radius, color)
    }

    /// Draws a triangle visible in debug mode.
    ///
    /// `center` is a position in world coordinates.
    /// `color` is 24-bit RGB.
    pub fn draw_square(center: Vec2, radius: f64, color: u32) {
        draw_polygon(center, radius, 4, TAU / 8.0, color);
    }

    #[deprecated]
    #[doc(hidden)]
    pub fn debug_square(center: Vec2, radius: f64, color: u32) {
        draw_square(center, radius, color)
    }

    /// Draws a triangle visible in debug mode.
    ///
    /// `center` is a position in world coordinates.
    /// `color` is 24-bit RGB.
    pub fn draw_diamond(center: Vec2, radius: f64, color: u32) {
        draw_polygon(center, radius, 4, 0.0, color);
    }

    #[deprecated]
    #[doc(hidden)]
    pub fn debug_diamond(center: Vec2, radius: f64, color: u32) {
        draw_diamond(center, radius, color)
    }

    /// Adds text to be drawn in the world, visible in debug mode.
    ///
    /// Works like [println!]. Up to 128 strings can be drawn per ship, per tick.
    #[macro_export]
    macro_rules! draw_text {
        ($topleft:expr, $color:expr, $($arg:tt)*) => {
            $crate::dbg::draw_text_internal($topleft, $color, std::format_args!($($arg)*))
        };
    }

    #[allow(unused)]
    #[doc(hidden)]
    pub fn draw_text_internal(topleft: Vec2, color: u32, args: std::fmt::Arguments) {
        use std::fmt::Write;
        let mut text = String::new();
        let _ = std::fmt::write(&mut text, args);
        let buf = unsafe { &mut DRAWN_TEXT_BUFFER };
        // TODO handle longer text
        let mut text_buf = [0u8; 11];
        text_buf
            .iter_mut()
            .zip(text.bytes())
            .for_each(|(d, s)| *d = s);
        buf.push(Text {
            x: topleft.x,
            y: topleft.y,
            color,
            length: text.len().min(text_buf.len()) as u8,
            text: text_buf,
        });
    }

    #[doc(hidden)]
    pub fn update() {
        {
            let slice = unsafe { &mut TEXT_BUFFER }.as_bytes();
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
            let slice = unsafe { &mut LINE_BUFFER }.as_slice();
            write_system_state(
                super::SystemState::DebugLinesPointer,
                slice.as_ptr() as u32 as f64,
            );
            write_system_state(
                super::SystemState::DebugLinesLength,
                slice.len() as u32 as f64,
            );
        }
        {
            let slice = unsafe { &mut DRAWN_TEXT_BUFFER }.as_slice();
            write_system_state(
                super::SystemState::DrawnTextPointer,
                slice.as_ptr() as u32 as f64,
            );
            write_system_state(
                super::SystemState::DrawnTextLength,
                slice.len() as u32 as f64,
            );
        }
    }

    #[doc(hidden)]
    pub fn reset() {
        unsafe {
            TEXT_BUFFER.clear();
            LINE_BUFFER.clear();
            DRAWN_TEXT_BUFFER.clear();
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
    pub use super::{Ability, Class, EcmMode, Message};
    #[doc(inline)]
    pub use crate::{debug, draw_text};

    pub use byteorder;
    pub use maths_rs;
}
