mod user;
mod shared;

pub mod sys {
    use shared::SystemState;

    #[no_mangle]
    pub static mut SYSTEM_STATE: [f64; SystemState::Size as usize] = [0.0; SystemState::Size as usize];

    pub fn read_system_state(index: SystemState) -> f64 {
        unsafe {
            SYSTEM_STATE[index as usize]
        }
    }

    pub fn write_system_state(index: SystemState, value: f64) {
        unsafe {
            SYSTEM_STATE[index as usize] = value;
        }
    }
}

pub mod vec {
    pub struct Vec2 {
        pub x: f64,
        pub y: f64,
    }

    pub fn vec2(x: f64, y: f64) -> Vec2 {
        Vec2 { x, y }
    }
}

pub mod api {
    use super::sys::{read_system_state, write_system_state};
    use super::vec::*;
    use crate::shared::{SystemState, Class};

    pub fn class() -> Class {
        Class::from_f64( read_system_state(SystemState::Class))
    }

    pub fn position() -> Vec2 {
        Vec2 {
            x: read_system_state(SystemState::PositionX),
            y: read_system_state(SystemState::PositionY),
        }
    }

    pub fn velocity() -> Vec2 {
        Vec2 {
            x: read_system_state(SystemState::VelocityX),
            y: read_system_state(SystemState::VelocityY),
        }
    }

    pub fn heading() -> f64 {
        read_system_state(SystemState::Heading)
    }

    pub fn angular_velocity() -> f64 {
        read_system_state(SystemState::AngularVelocity)
    }

    pub fn accelerate(acceleration: Vec2) {
        write_system_state(SystemState::AccelerateX, acceleration.x );
        write_system_state(SystemState::AccelerateY, acceleration.y );
    }

    pub fn torque(angular_acceleration: f64) {
        write_system_state(SystemState::Torque, angular_acceleration );
    }

    pub fn aim_gun(gun_index: usize, heading: f64) {
        let state_index = match gun_index {
            0 => SystemState::Gun0Heading,
            1 => SystemState::Gun1Heading,
            2 => SystemState::Gun2Heading,
            3 => SystemState::Gun3Heading,
            _ => return,
        };
        write_system_state(state_index, heading);
    }

    pub fn fire_gun(gun_index: usize) {
        let state_index = match gun_index {
            0 => SystemState::Gun0Fired,
            1 => SystemState::Gun1Fired,
            2 => SystemState::Gun2Fired,
            3 => SystemState::Gun3Fired,
            _ => return,
        };
        write_system_state(state_index, 1.0);
    }

    pub fn launch_missile(missile_index: usize) {
        let state_index = match missile_index {
            0 => SystemState::Missile0Launched,
            1 => SystemState::Missile1Launched,
            2 => SystemState::Missile2Launched,
            3 => SystemState::Missile3Launched,
            _ => return,
        };
        write_system_state(state_index, 1.0);
    }

    pub fn explode() {
        write_system_state(SystemState::Explode, 1.0);
    }

    pub fn set_radar_heading(heading: f64) {
        write_system_state(SystemState::RadarHeading, heading);
    }

    pub fn set_radar_width(width: f64) {
        write_system_state(SystemState::RadarWidth, width);
    }

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
}

pub mod prelude {
    pub use super::api::*;
    pub use super::vec::*;
    pub use crate::shared::*;
}

#[no_mangle]
pub fn export_tick() {
    user::tick();
}
