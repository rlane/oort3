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
) -> (
    f64, /* probability of detection */
    f64, /* distance error RMS */
) {
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

    let trials = 100;
    let mut square_errors = 0.0;
    let mut detections = 0;

    for _ in 0..trials {
        sim.step();
        let x = sim
            .ship(ship0)
            .radar()
            .unwrap()
            .result
            .map(|contact| (contact.position - target_position).magnitude());
        if let Some(error) = x {
            detections += 1;
            square_errors += error.powi(2);
        }
    }
    let rms_error = (square_errors / (trials as f64)).sqrt();

    (detections as f64 / trials as f64, rms_error)
}

fn check_accuracy(
    emitter_class: ShipClass,
    reflector_class: ShipClass,
    range: f64,
    beamwidth: f64,
) -> bool {
    let (detection_chance, rms_error) =
        run_simulation(emitter_class, reflector_class, range, beamwidth);
    detection_chance >= 0.9 && rms_error < 10.0
}

fn main() {
    let ship_classes = [Missile, Torpedo, Fighter, Frigate, Cruiser];

    let ranges = [
        200.0, 180.0, 160.0, 140.0, 120.0, 100.0, 90.0, 80.0, 70.0, 60.0, 50.0, 40.0, 30.0, 25.0,
        20.0, 15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.5, 1.0,
        0.5,
    ];

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
                    let range_km = ranges.iter().cloned().find(|x| {
                        let range = x * 1e3;
                        let (detection_chance, _rms_error) =
                            run_simulation(emitter_class, reflector_class, range, beamwidth);
                        detection_chance >= 0.5
                    });
                    row.push(range_km.unwrap_or_default().to_string());
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
                    let range_km = ranges.iter().cloned().find(|x| {
                        let range = x * 1e3;
                        check_accuracy(emitter_class, reflector_class, range, beamwidth)
                    });
                    row.push(range_km.unwrap_or_default().to_string());
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
