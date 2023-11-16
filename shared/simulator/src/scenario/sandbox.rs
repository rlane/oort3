use crate::ship::ShipClass;

use super::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

pub struct Sandbox {
    beacon_handle: Option<ShipHandle>,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            beacon_handle: None,
        }
    }

    fn handle_command(sim: &mut Simulation, cmd: &str) -> anyhow::Result<()> {
        lazy_static! {
            static ref SPAWN_RE: Regex =
                Regex::new(r"spawn (\w+) team (\d+) position (\(.+?\)) heading ([\-\d.+])")
                    .unwrap();
        }
        if let Some(cap) = SPAWN_RE.captures(cmd) {
            let class: ShipClass = cap.get(1).unwrap().as_str().parse()?;
            let team: i32 = cap.get(2).unwrap().as_str().parse()?;
            let position: Vector2<f64> = parse_vec2(cap.get(3).unwrap().as_str())?;
            let heading: f64 = cap.get(4).unwrap().as_str().parse()?;
            let heading = heading.to_radians();
            let data = match class {
                ShipClass::Fighter => fighter(team),
                ShipClass::Frigate => frigate(team),
                ShipClass::Cruiser => cruiser(team),
                ShipClass::Asteroid { variant } => asteroid(variant),
                ShipClass::BigAsteroid { variant } => big_asteroid(variant),
                ShipClass::Target => target(team),
                ShipClass::Missile => missile(team),
                ShipClass::Torpedo => torpedo(team),
                ShipClass::Beacon => beacon(team),
                _ => anyhow::bail!("Unsupported ship class {:?}", class),
            };
            ship::create(sim, position, vector![0.0, 0.0], heading, data);
        } else {
            anyhow::bail!("Unknown command {:?}", cmd);
        }
        Ok(())
    }
}

impl Scenario for Sandbox {
    fn name(&self) -> String {
        "sandbox".into()
    }

    fn init(&mut self, sim: &mut Simulation, _seed: u32) {
        self.beacon_handle = Some(ship::create(
            sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            beacon(0),
        ));
    }

    fn tick(&mut self, sim: &mut Simulation) {
        if let Some(beacon_handle) = self.beacon_handle {
            let text = sim.events().debug_text.get(&beacon_handle.into()).cloned();
            if let Some(text) = text {
                for line in text.lines() {
                    if line.contains("CPU:") {
                        continue;
                    }
                    if let Err(e) = Sandbox::handle_command(sim, line) {
                        log::warn!("Failed to parse sandbox command {:?}: {:?}", line, e);
                    }
                }
            }
        }
    }

    fn status(&self, _: &Simulation) -> Status {
        Status::Running
    }
}

fn parse_vec2(s: &str) -> anyhow::Result<Vector2<f64>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\(([\-\d.]+), ([\-\d.]+)\)").unwrap();
    }
    let cap = RE
        .captures(s)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse {:?}", s))?;
    let x: f64 = cap.get(1).unwrap().as_str().parse()?;
    let y: f64 = cap.get(2).unwrap().as_str().parse()?;
    Ok(vector![x, y])
}
