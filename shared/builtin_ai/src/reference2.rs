use oort_api::prelude::*;

const BULLET_SPEED: f64 = 1000.0; // m/s

pub enum Ship {
    Fighter(Fighter),
    Frigate(Frigate),
    Cruiser(Cruiser),
    Missile(Missile),
}

impl Ship {
    pub fn new() -> Ship {
        match class() {
            Class::Fighter => Ship::Fighter(Fighter::new()),
            Class::Frigate => Ship::Frigate(Frigate::new()),
            Class::Cruiser => Ship::Cruiser(Cruiser::new()),
            Class::Missile => Ship::Missile(Missile::new()),
            Class::Torpedo => Ship::Missile(Missile::new()),
            _ => unreachable!(),
        }
    }

    pub fn tick(&mut self) {
        match self {
            Ship::Fighter(fighter) => fighter.tick(),
            Ship::Frigate(frigate) => frigate.tick(),
            Ship::Cruiser(cruiser) => cruiser.tick(),
            Ship::Missile(missile) => missile.tick(),
        }
    }
}

// Fighters
pub struct Fighter {
    pub move_target: Vec2,
}

impl Fighter {
    pub fn new() -> Self {
        Self {
            move_target: vec2(0.0, 0.0),
        }
    }

    pub fn tick(&mut self) {
        if let Some(contact) = scan().filter(|c| {
            [
                Class::Fighter,
                Class::Frigate,
                Class::Cruiser,
                Class::Torpedo,
            ]
            .contains(&c.class)
        }) {
            let dp = contact.position - position();
            let dv = contact.velocity - velocity();
            set_radar_heading(dp.angle());
            set_radar_width(radar_width() * 0.5);

            seek(
                contact.position + dv.normalize().rotate(TAU / 4.0) * 5e3,
                vec2(0.0, 0.0),
                5.0,
                true,
                1e3,
            );

            // Guns
            if let Some(angle) = lead_target(contact.position, contact.velocity) {
                let angle = angle + rand(-1.0, 1.0) * TAU / 120.0;
                turn_to(angle);
                if angle_diff(angle, heading()).abs() < TAU / 60.0 {
                    fire(0);
                }
            }

            // Missiles
            if reload_ticks(1) == 0 {
                send(make_orders(contact.position, contact.velocity));
                fire(1);
            }
        } else {
            set_radar_heading(radar_heading() + radar_width());
            set_radar_width(TAU / 120.0);
            seek(self.move_target, vec2(0.0, 0.0), 5.0, true, 1e3);
        }
    }
}

// Frigates
pub struct Frigate {}

impl Frigate {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self) {}
}

// Cruisers
pub struct Cruiser {}

impl Cruiser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self) {}
}

// Missiles and Torpedos
pub struct Missile {
    target_position: Vec2,
    target_velocity: Vec2,
}

impl Missile {
    pub fn new() -> Self {
        let (target_position, target_velocity) = parse_orders(receive());
        Self {
            target_position,
            target_velocity,
        }
    }

    pub fn tick(&mut self) {
        self.target_position += self.target_velocity * TICK_LENGTH;

        if let Some(contact) = scan().filter(|c| {
            [
                Class::Fighter,
                Class::Frigate,
                Class::Cruiser,
                Class::Torpedo,
            ]
            .contains(&c.class)
        }) {
            let dp = contact.position - position();
            set_radar_heading(dp.angle());
            set_radar_width(radar_width() * 0.5);
            self.target_position = contact.position;
            self.target_velocity = contact.velocity;
        } else {
            set_radar_heading(
                (self.target_position - position()).angle() + rand(-1.0, 1.0) * TAU / 32.0,
            );
            set_radar_width(TAU / 120.0);
        }

        seek(self.target_position, self.target_velocity, 5.0, true, 10e3);
    }
}

// Library functions
pub fn seek(p: Vec2, v: Vec2, n: f64, turn: bool, max_speed: f64) {
    let dp = p - position();
    let dv = v - velocity();
    let los = dp.angle();
    let los_rate = (dp.y * dv.x - dp.x * dv.y) / (dp.length() * dp.length());
    let closing_speed = -dv.dot(dp).abs() / dp.length();
    let low_fuel = fuel() != 0.0 && fuel() < 500.0;
    let closing_acc = (low_fuel || closing_speed.abs() > max_speed)
        .then_some(0.0)
        .unwrap_or_else(max_forward_acceleration);
    let a = vec2(closing_acc, n * closing_speed * los_rate).rotate(los);
    accelerate(a);
    if turn {
        turn_to(a.angle());
    }
    let los_color = if los_rate.abs() < 0.01 {
        0x42f560
    } else if los_rate.abs() < 0.1 {
        0xf5f242
    } else {
        0xf54242
    };
    draw_line(position(), p, los_color);
}

fn turn_to(target_heading: f64) {
    let heading_error = angle_diff(heading(), target_heading);
    turn(10.0 * heading_error);
}

fn lead_target(target_position: Vec2, target_velocity: Vec2) -> Option<f64> {
    let dp = target_position - position();
    let dv = target_velocity - velocity();
    for i in 0..60 {
        let t = 10.0 * i as f64 * TICK_LENGTH;
        let dp2 = dp + dv * t;
        if (dp2.length() - BULLET_SPEED * t).abs() < 1e3 {
            return Some(dp2.angle());
        }
    }
    None
}

fn parse_orders(msg: Option<Message>) -> (Vec2, Vec2) {
    if let Some(msg) = msg {
        (vec2(msg[0], msg[1]), vec2(msg[2], msg[3]))
    } else {
        (vec2(0.0, 0.0), vec2(0.0, 0.0))
    }
}

fn make_orders(p: Vec2, v: Vec2) -> Message {
    [p.x, p.y, v.x, v.y]
}
