use oort_api::prelude::*;

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
            if let Some(angle) = lead_target(contact.position, contact.velocity, 1e3, 10.0) {
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
pub struct Frigate {
    pub move_target: Vec2,
    pub radar_state: FrigateRadarState,
    pub main_gun_radar: RadarRegs,
    pub point_defense_radar: RadarRegs,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FrigateRadarState {
    MainGun,
    PointDefense,
}

impl Frigate {
    pub fn new() -> Self {
        Self {
            move_target: vec2(0.0, 0.0),
            radar_state: FrigateRadarState::MainGun,
            main_gun_radar: RadarRegs::new(),
            point_defense_radar: RadarRegs::new(),
        }
    }

    pub fn tick(&mut self) {
        if self.radar_state == FrigateRadarState::MainGun {
            if let Some(contact) = scan()
                .filter(|c| [Class::Fighter, Class::Frigate, Class::Cruiser].contains(&c.class))
            {
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

                // Main gun
                if let Some(angle) = lead_target(contact.position, contact.velocity, 4e3, 60.0) {
                    turn_to(angle);
                    if angle_diff(angle, heading()).abs() < TAU / 360.0 {
                        fire(0);
                    }
                }

                // Missiles
                if reload_ticks(3) == 0 {
                    send(make_orders(contact.position, contact.velocity));
                    fire(3);
                }
            } else {
                set_radar_heading(radar_heading() + radar_width());
                set_radar_width(TAU / 120.0);
                seek(self.move_target, vec2(0.0, 0.0), 5.0, true, 1e3);
            }

            self.main_gun_radar.save();
            self.point_defense_radar.restore();
            self.radar_state = FrigateRadarState::PointDefense;
        } else if self.radar_state == FrigateRadarState::PointDefense {
            set_radar_width(TAU / 16.0);

            if let Some(contact) = scan()
                .filter(|c| [Class::Fighter, Class::Missile, Class::Torpedo].contains(&c.class))
            {
                for idx in [1, 2] {
                    if let Some(angle) = lead_target(contact.position, contact.velocity, 1e3, 10.0)
                    {
                        aim(idx, angle + rand(-1.0, 1.0) * TAU / 120.0);
                        fire(idx);
                    }
                }
                set_radar_heading((contact.position - position()).angle());
            } else {
                set_radar_heading(radar_heading() + radar_width());
            }

            self.point_defense_radar.save();
            self.main_gun_radar.restore();
            self.radar_state = FrigateRadarState::MainGun;
        }
    }
}

// Cruisers
pub struct Cruiser {
    pub move_target: Vec2,
    pub radar_state: CruiserRadarState,
    pub torpedo_radar: RadarRegs,
    pub missile_radar: RadarRegs,
    pub point_defense_radar: RadarRegs,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CruiserRadarState {
    Torpedo,
    Missile,
    PointDefense,
}

impl Cruiser {
    pub fn new() -> Self {
        Self {
            move_target: vec2(0.0, 0.0),
            radar_state: CruiserRadarState::Torpedo,
            torpedo_radar: RadarRegs::new(),
            missile_radar: RadarRegs::new(),
            point_defense_radar: RadarRegs::new(),
        }
    }

    pub fn tick(&mut self) {
        seek(self.move_target, vec2(0.0, 0.0), 5.0, true, 1e3);

        if self.radar_state == CruiserRadarState::Torpedo {
            if let Some(contact) =
                scan().filter(|c| [Class::Frigate, Class::Cruiser].contains(&c.class))
            {
                let dp = contact.position - position();
                set_radar_heading(dp.angle());
                set_radar_width(radar_width() * 0.5);

                if reload_ticks(3) == 0 {
                    send(make_orders(contact.position, contact.velocity));
                    fire(3);
                }
            } else {
                set_radar_heading(radar_heading() + radar_width());
                set_radar_width(TAU / 120.0);
            }

            self.torpedo_radar.save();
            self.missile_radar.restore();
            self.radar_state = CruiserRadarState::Missile;
        } else if self.radar_state == CruiserRadarState::Missile {
            set_radar_width(TAU / 64.0);

            if let Some(contact) = scan().filter(|c| {
                [
                    Class::Fighter,
                    Class::Frigate,
                    Class::Cruiser,
                    Class::Torpedo,
                ]
                .contains(&c.class)
            }) {
                let mut fired = false;
                for idx in [1, 2] {
                    if reload_ticks(idx) == 0 {
                        send(make_orders(contact.position, contact.velocity));
                        fire(idx);
                        fired = true;
                        break;
                    }
                }
                if fired {
                    set_radar_heading(radar_heading() + radar_width());
                } else {
                    set_radar_heading((contact.position - position()).angle());
                }
            } else {
                set_radar_heading(radar_heading() + radar_width());
            }

            self.missile_radar.save();
            self.point_defense_radar.restore();
            self.radar_state = CruiserRadarState::PointDefense;
        } else if self.radar_state == CruiserRadarState::PointDefense {
            set_radar_width(TAU / 16.0);

            if let Some(contact) =
                scan().filter(|c| [Class::Missile, Class::Torpedo].contains(&c.class))
            {
                let dp = contact.position - position();
                if dp.length() < 2e3 {
                    aim(0, dp.angle());
                    fire(0);
                }
                set_radar_heading(dp.angle());
            } else {
                set_radar_heading(radar_heading() + radar_width());
            }

            self.point_defense_radar.save();
            self.torpedo_radar.restore();
            self.radar_state = CruiserRadarState::Torpedo;
        }
    }
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

        let missile_target_classes = [
            Class::Fighter,
            Class::Frigate,
            Class::Cruiser,
            Class::Torpedo,
        ];
        let torpedo_target_classes = [Class::Frigate, Class::Cruiser];
        let target_classes = if class() == Class::Missile {
            missile_target_classes.as_slice()
        } else {
            torpedo_target_classes.as_slice()
        };

        if let Some(contact) = scan().filter(|c| target_classes.contains(&c.class)) {
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

fn lead_target(
    target_position: Vec2,
    target_velocity: Vec2,
    bullet_speed: f64,
    ttl: f64,
) -> Option<f64> {
    let dp = target_position - position();
    let dv = target_velocity - velocity();
    for i in 0..((ttl / TICK_LENGTH) as usize / 10) {
        let t = 10.0 * i as f64 * TICK_LENGTH;
        let dp2 = dp + dv * t;
        if (dp2.length() - bullet_speed * t).abs() < 1e3 {
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

/// Save and restore radar registers in order to use a single radar for multiple functions.
pub struct RadarRegs {
    heading: f64,
    width: f64,
}

impl RadarRegs {
    fn new() -> Self {
        Self {
            heading: 0.0,
            width: TAU / 120.0,
        }
    }

    fn save(&mut self) {
        self.heading = radar_heading();
        self.width = radar_width();
    }

    fn restore(&self) {
        set_radar_heading(self.heading);
        set_radar_width(self.width);
    }
}
