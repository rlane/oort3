mod user;
mod vec;

pub mod sys {
    use oort_shared::SystemState;

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

pub mod math {
    pub use std::f64::consts::{PI, TAU};

    pub fn normalize_angle(a: f64) -> f64 {
        let mut a = a;
        if a.abs() > TAU {
            a %= TAU;
        }
        if a < 0.0 {
            a += TAU;
        }
        a
    }

    pub fn angle_diff(a: f64, b: f64) -> f64 {
        let c = normalize_angle(b - a);
        if c > PI {
            c - TAU
        } else {
            c
        }
    }
}

pub mod rng {
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

    pub fn rand(low: f64, high: f64) -> f64 {
        rng().rand_float() * (high - low) + low
    }
}

pub mod api {
    use super::sys::{read_system_state, write_system_state};
    use crate::vec::*;
    use oort_shared::{Class, SystemState};

    pub const TICK_LENGTH: f64 = 1.0 / 60.0;

    pub fn class() -> Class {
        Class::from_f64(read_system_state(SystemState::Class))
    }

    pub fn seed() -> u128 {
        read_system_state(oort_shared::SystemState::Seed) as u128
    }

    pub fn orders() -> f64 {
        read_system_state(oort_shared::SystemState::Orders)
    }

    pub fn position() -> Vec2 {
        vec2(
            read_system_state(SystemState::PositionX),
            read_system_state(SystemState::PositionY),
        )
    }

    pub fn velocity() -> Vec2 {
        vec2(
            read_system_state(SystemState::VelocityX),
            read_system_state(SystemState::VelocityY),
        )
    }

    pub fn heading() -> f64 {
        read_system_state(SystemState::Heading)
    }

    pub fn angular_velocity() -> f64 {
        read_system_state(SystemState::AngularVelocity)
    }

    pub fn accelerate(acceleration: Vec2) {
        write_system_state(SystemState::AccelerateX, acceleration.x);
        write_system_state(SystemState::AccelerateY, acceleration.y);
    }

    pub fn torque(angular_acceleration: f64) {
        write_system_state(SystemState::Torque, angular_acceleration);
    }

    pub fn aim_gun(gun_index: usize, heading: f64) {
        let state_index = match gun_index {
            0 => SystemState::Gun0Aim,
            1 => SystemState::Gun1Aim,
            2 => SystemState::Gun2Aim,
            3 => SystemState::Gun3Aim,
            _ => return,
        };
        write_system_state(state_index, heading);
    }

    pub fn fire_gun(gun_index: usize) {
        let state_index = match gun_index {
            0 => SystemState::Gun0Fire,
            1 => SystemState::Gun1Fire,
            2 => SystemState::Gun2Fire,
            3 => SystemState::Gun3Fire,
            _ => return,
        };
        write_system_state(state_index, 1.0);
    }

    pub fn launch_missile(missile_index: usize, orders: f64) {
        let (state_index, orders_index) = match missile_index {
            0 => (SystemState::Missile0Launch, SystemState::Missile0Orders),
            1 => (SystemState::Missile1Launch, SystemState::Missile1Orders),
            2 => (SystemState::Missile2Launch, SystemState::Missile2Orders),
            3 => (SystemState::Missile3Launch, SystemState::Missile3Orders),
            _ => return,
        };
        write_system_state(state_index, 1.0);
        write_system_state(orders_index, orders);
    }

    pub fn explode() {
        write_system_state(SystemState::Explode, 1.0);
    }

    pub fn radar_heading() -> f64 {
        read_system_state(SystemState::RadarHeading)
    }

    pub fn set_radar_heading(heading: f64) {
        write_system_state(SystemState::RadarHeading, heading);
    }

    pub fn radar_width() -> f64 {
        read_system_state(SystemState::RadarWidth)
    }

    pub fn set_radar_width(width: f64) {
        write_system_state(SystemState::RadarWidth, width);
    }

    pub fn radar_min_distance() -> f64 {
        read_system_state(SystemState::RadarMinDistance)
    }

    pub fn set_radar_min_distance(dist: f64) {
        write_system_state(SystemState::RadarMinDistance, dist);
    }

    pub fn radar_max_distance() -> f64 {
        read_system_state(SystemState::RadarMaxDistance)
    }

    pub fn set_radar_max_distance(dist: f64) {
        write_system_state(SystemState::RadarMaxDistance, dist);
    }

    #[derive(Clone, Debug)]
    pub struct ScanResult {
        pub class: Class,
        pub position: Vec2,
        pub velocity: Vec2,
    }

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

    pub fn set_radio_channel(channel: usize) {
        write_system_state(SystemState::RadioChannel, channel as f64);
    }

    pub fn get_radio_channel() -> usize {
        read_system_state(SystemState::RadioChannel) as usize
    }

    pub fn send(data: f64) {
        write_system_state(SystemState::RadioSend, data);
    }

    pub fn receive() -> f64 {
        read_system_state(SystemState::RadioReceive)
    }

    pub fn max_acceleration() -> Vec2 {
        vec2(
            read_system_state(SystemState::MaxAccelerationX),
            read_system_state(SystemState::MaxAccelerationY),
        )
    }

    pub fn max_angular_acceleration() -> f64 {
        read_system_state(SystemState::MaxAngularAcceleration)
    }

    pub fn current_tick() -> u32 {
        read_system_state(SystemState::CurrentTick) as u32
    }

    pub fn current_time() -> f64 {
        read_system_state(SystemState::CurrentTick) * TICK_LENGTH
    }

    // Only used in tutorials.
    pub fn target() -> Vec2 {
        vec2(
            read_system_state(SystemState::RadarContactPositionX),
            read_system_state(SystemState::RadarContactPositionY),
        )
    }
}

#[macro_use]
pub mod debug {
    use crate::sys::write_system_state;
    use crate::vec::*;
    pub use oort_shared::Line;
    pub use std::f64::consts::TAU;

    static mut TEXT_BUFFER: String = String::new();
    static mut LINE_BUFFER: Vec<Line> = Vec::new();

    #[macro_export]
    macro_rules! debug {
        ($($arg:tt)*) => {
            crate::debug::write(std::format_args!($($arg)*))
        };
    }

    #[allow(unused)]
    pub(super) fn write(args: std::fmt::Arguments) {
        use std::fmt::Write;
        unsafe {
            let _ = std::fmt::write(&mut TEXT_BUFFER, args);
            TEXT_BUFFER.push('\n');
        }
    }

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

    pub fn debug_triangle(center: Vec2, radius: f64, color: u32) {
        debug_polygon(center, radius, 3, TAU / 4.0, color);
    }

    pub fn debug_square(center: Vec2, radius: f64, color: u32) {
        debug_polygon(center, radius, 4, 0.0, color);
    }

    pub fn debug_diamond(center: Vec2, radius: f64, color: u32) {
        debug_polygon(center, radius, 4, TAU / 8.0, color);
    }

    pub(super) fn update() {
        {
            let slice = unsafe { TEXT_BUFFER.as_bytes() };
            write_system_state(
                oort_shared::SystemState::DebugTextPointer,
                slice.as_ptr() as u32 as f64,
            );
            write_system_state(
                oort_shared::SystemState::DebugTextLength,
                slice.len() as u32 as f64,
            );
        }
        {
            let slice = unsafe { LINE_BUFFER.as_slice() };
            write_system_state(
                oort_shared::SystemState::DebugLinesPointer,
                slice.as_ptr() as u32 as f64,
            );
            write_system_state(
                oort_shared::SystemState::DebugLinesLength,
                slice.len() as u32 as f64,
            );
        }
    }

    pub(super) fn reset() {
        unsafe {
            TEXT_BUFFER.clear();
            LINE_BUFFER.clear();
        }
    }
}

pub mod prelude {
    pub use super::api::*;
    pub use super::debug::*;
    pub use super::math::*;
    pub use super::rng::*;
    pub use super::vec::*;
    pub use crate::debug;
    pub use oort_shared::*;
}

static mut USER_STATE: Option<user::Ship> = None;

#[no_mangle]
pub fn export_tick() {
    unsafe {
        debug::reset();
        if USER_STATE.is_none() {
            USER_STATE = Some(user::Ship::new());
        }
        USER_STATE.as_mut().unwrap().tick();
        debug::update();
    }
}
