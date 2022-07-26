#[derive(Copy, Clone)]
pub enum SystemState {
    Class,
    Seed,
    Orders,
    PositionX,
    PositionY,
    VelocityX,
    VelocityY,
    Heading,
    AngularVelocity,

    AccelerateX,
    AccelerateY,
    Torque,

    Gun0Aim,
    Gun0Fire,
    Gun1Aim,
    Gun1Fire,
    Gun2Aim,
    Gun2Fire,
    Gun3Aim,
    Gun3Fire,

    Missile0Launch,
    Missile0Orders,
    Missile1Launch,
    Missile1Orders,
    Missile2Launch,
    Missile2Orders,
    Missile3Launch,
    Missile3Orders,

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

    Size,
    MaxSize = 128,
}

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

#[derive(Default, Clone)]
pub struct Line {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub color: u32,
}
