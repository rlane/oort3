use crate::ship::ShipHandle;
use crate::simulation::Simulation;
use nalgebra::Point2;
use std::collections::BTreeMap;
use std::f64::consts::TAU;

const NUM_CHANNELS: usize = 10;

#[derive(Clone, Debug)]
pub struct Radio {
    pub power: f64,
    pub rx_cross_section: f64,
    pub min_rssi: f64,
    pub channel: usize,
    pub sent: Option<f64>,
    pub received: Option<f64>,
}

struct RadioSender {
    position: Point2<f64>,
    power: f64,
    data: f64,
}

struct RadioReceiver {
    handle: ShipHandle,
    position: Point2<f64>,
    rx_cross_section: f64,
    min_rssi: f64,
}

#[inline(never)]
pub fn tick(sim: &mut Simulation) {
    let handle_snapshot: Vec<ShipHandle> = sim.ships.iter().cloned().collect();

    let mut receivers: BTreeMap<usize, Vec<RadioReceiver>> = BTreeMap::new();
    let mut senders: BTreeMap<usize, Vec<RadioSender>> = BTreeMap::new();

    for handle in handle_snapshot.iter().cloned() {
        let ship = sim.ship(handle);
        let ship_data = ship.data();
        if let Some(radio) = ship_data.radio.as_ref() {
            receivers
                .entry(radio.channel)
                .or_default()
                .push(RadioReceiver {
                    handle,
                    position: ship.position().vector.into(),
                    rx_cross_section: radio.rx_cross_section,
                    min_rssi: radio.min_rssi,
                });

            if let Some(data) = radio.sent {
                senders.entry(radio.channel).or_default().push(RadioSender {
                    position: ship.position().vector.into(),
                    power: radio.power,
                    data,
                });
            }
        }
    }

    for channel in 0..NUM_CHANNELS {
        for rx in receivers.get(&channel).unwrap_or(&Vec::new()) {
            let mut best_data = None;
            let mut best_rssi = rx.min_rssi;
            for tx in senders.get(&channel).unwrap_or(&Vec::new()) {
                let rssi = compute_rssi(tx, rx);
                if rssi > best_rssi {
                    best_rssi = rssi;
                    best_data = Some(tx.data);
                }
            }
            sim.ship_mut(rx.handle)
                .data_mut()
                .radio
                .as_mut()
                .unwrap()
                .received = best_data;
        }
    }

    for handle in handle_snapshot.iter().cloned() {
        if let Some(radio) = sim.ship_mut(handle).data_mut().radio.as_mut() {
            radio.sent = None;
        }
    }
}

fn compute_rssi(sender: &RadioSender, receiver: &RadioReceiver) -> f64 {
    let r_sq = nalgebra::distance_squared(&sender.position, &receiver.position);
    sender.power * receiver.rx_cross_section / (TAU * r_sq)
}

#[cfg(test)]
mod test {
    use crate::ship;
    use crate::simulation::Code;
    use crate::simulation::Simulation;
    use test_log::test;

    #[test]
    fn test_basic() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let ship1 = ship::create(&mut sim, 1000.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));

        sim.step();

        assert!(sim.ship(ship0).radio().unwrap().received.is_none());
        assert!(sim.ship(ship1).radio().unwrap().received.is_none());

        sim.ship_mut(ship1).radio_mut().unwrap().sent = Some(1.0);

        sim.step();

        assert_eq!(sim.ship(ship0).radio().unwrap().received, Some(1.0));
        assert_eq!(sim.ship(ship1).radio().unwrap().received, Some(1.0));

        sim.step();

        assert!(sim.ship(ship0).radio().unwrap().received.is_none());
        assert!(sim.ship(ship1).radio().unwrap().received.is_none());
    }

    #[test]
    fn test_channel() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(&mut sim, 0.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));
        let ship1 = ship::create(&mut sim, 1000.0, 0.0, 0.0, 0.0, 0.0, ship::fighter(0));

        sim.ship_mut(ship1).radio_mut().unwrap().sent = Some(1.0);
        sim.ship_mut(ship1).radio_mut().unwrap().channel = 5;

        sim.step();

        assert_eq!(sim.ship(ship0).radio().unwrap().received, None);

        sim.ship_mut(ship0).radio_mut().unwrap().channel = 5;
        sim.ship_mut(ship1).radio_mut().unwrap().sent = Some(1.0);

        sim.step();

        assert_eq!(sim.ship(ship0).radio().unwrap().received, Some(1.0));
    }
}
