pub enum SystemState {
    Class,
    PositionX,
    PositionY,
    VelocityX,
    VelocityY,
    Heading,
    AngularVelocity,

    AccelerateX,
    AccelerateY,
    Torque,

    Gun0Heading,
    Gun0Fired,
    Gun1Heading,
    Gun1Fired,
    Gun2Heading,
    Gun2Fired,
    Gun3Heading,
    Gun3Fired,

    Missile0Launched,
    Missile1Launched,
    Missile2Launched,
    Missile3Launched,

    Explode,

    RadarHeading,
    RadarWidth,
    RadarContactFound,
    RadarContactClass,
    RadarContactPositionX,
    RadarContactPositionY,
    RadarContactVelocityX,
    RadarContactVelocityY,

    Size
}

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
