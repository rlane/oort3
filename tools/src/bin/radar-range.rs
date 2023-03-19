use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use nalgebra::vector;
use oort_simulator::ship;
use oort_simulator::ship::ShipClass::*;
use oort_simulator::ship::{ShipClass, ShipData};
use oort_simulator::simulation::{Code, Simulation, MAX_WORLD_SIZE};
use std::f64::consts::TAU;

fn class_to_ship_data(class: ShipClass, team: i32) -> ShipData {
    match class {
        ShipClass::Fighter => ship::fighter(team),
        ShipClass::Frigate => ship::frigate(team),
        ShipClass::Cruiser => ship::cruiser(team),
        ShipClass::Missile => ship::missile(team),
        ShipClass::Torpedo => ship::torpedo(team),
        _ => unimplemented!(),
    }
}

fn run_simulation(
    emitter_class: ShipClass,
    reflector_class: ShipClass,
    range: f64,
    beamwidth: f64,
) -> Option<f64> /* distance error */ {
    let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);
    let offset = -MAX_WORLD_SIZE / 2.0 + 100.0;
    let ship0 = ship::create(
        &mut sim,
        vector![offset, 0.0],
        vector![0.0, 0.0],
        0.0,
        class_to_ship_data(emitter_class, 0),
    );
    let target_position = vector![range + offset, 0.0];
    ship::create(
        &mut sim,
        target_position,
        vector![0.0, 0.0],
        0.0,
        class_to_ship_data(reflector_class, 1),
    );
    sim.ship_mut(ship0).radar_mut().unwrap().heading = 0.0;
    sim.ship_mut(ship0).radar_mut().unwrap().width = beamwidth;
    sim.step();

    sim.ship(ship0)
        .radar()
        .unwrap()
        .result
        .map(|contact| (contact.position - target_position).magnitude())
}

fn check_accuracy(
    emitter_class: ShipClass,
    reflector_class: ShipClass,
    range: f64,
    beamwidth: f64,
) -> bool {
    let mut square_errors = 0.0;
    let trials = 10;
    for _ in 0..trials {
        square_errors += run_simulation(emitter_class, reflector_class, range, beamwidth)
            .unwrap_or(100.0)
            .powi(2);
    }
    let rms_error = (square_errors / (trials as f64)).sqrt();
    rms_error < 10.0
}

fn main() {
    let ship_classes = [Missile, Torpedo, Fighter, Frigate, Cruiser];

    for beamwidth in [TAU / 360.0, TAU / 60.0, TAU / 16.0] {
        {
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(
                ["".to_string()]
                    .into_iter()
                    .chain(ship_classes.iter().map(|x| format!("{x:?}")))
                    .collect::<Vec<_>>(),
            );

            for emitter_class in ship_classes {
                let mut row = vec![format!("{emitter_class:?}")];
                for reflector_class in ship_classes {
                    let range_km = (0..202).find(|x| {
                        let range = *x as f64 * 1e3 + 1e3;
                        run_simulation(emitter_class, reflector_class, range, beamwidth).is_none()
                    });
                    row.push(range_km.unwrap_or(200).to_string());
                }
                table.add_row(row);
            }

            println!(
                "Maximum detection range with beamwidth {:.1} degrees:\n{}\n",
                beamwidth * 360.0 / TAU,
                table,
            );
        }

        {
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(
                ["".to_string()]
                    .into_iter()
                    .chain(ship_classes.iter().map(|x| format!("{x:?}")))
                    .collect::<Vec<_>>(),
            );

            for emitter_class in ship_classes {
                let mut row = vec![format!("{emitter_class:?}")];
                for reflector_class in ship_classes {
                    let range_km = (0..202).find(|x| {
                        let range = *x as f64 * 1e3 + 1e3;
                        !check_accuracy(emitter_class, reflector_class, range, beamwidth)
                    });
                    row.push(range_km.unwrap_or(200).to_string());
                }
                table.add_row(row);
            }

            println!(
                "Accurate (10m) range with beamwidth {:.1} degrees:\n{}\n",
                beamwidth * 360.0 / TAU,
                table,
            );
        }
    }
}
