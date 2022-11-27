use crate::ship::ShipHandle;
use crate::simulation::Simulation;
use nalgebra::Point2;
use oort_api::Message;
use std::collections::BTreeMap;
use std::f64::consts::TAU;

const NUM_CHANNELS: usize = 10;

#[derive(Clone, Debug)]
pub struct Radio {
    pub(crate) power: f64,
    pub(crate) rx_cross_section: f64,
    pub(crate) min_rssi: f64,
    pub(crate) channel: usize,
    pub(crate) sent: Option<Message>,
    pub(crate) received: Option<Message>,
}

impl Radio {
    pub fn get_channel(&self) -> usize {
        self.channel
    }

    pub fn set_channel(&mut self, channel: usize) {
        self.channel = channel.clamp(0, NUM_CHANNELS - 1);
    }

    pub fn set_sent(&mut self, sent: Option<Message>) {
        self.sent = sent;
    }

    pub fn get_received(&self) -> Option<Message> {
        self.received
    }
}

struct RadioSender {
    position: Point2<f64>,
    power: f64,
    msg: Message,
}

struct RadioReceiver {
    handle: ShipHandle,
    radio_index: usize,
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
        for (radio_index, radio) in ship_data.radios.iter().enumerate() {
            receivers
                .entry(radio.channel)
                .or_default()
                .push(RadioReceiver {
                    handle,
                    radio_index,
                    position: ship.position().vector.into(),
                    rx_cross_section: radio.rx_cross_section,
                    min_rssi: radio.min_rssi,
                });

            if let Some(msg) = radio.sent {
                senders.entry(radio.channel).or_default().push(RadioSender {
                    position: ship.position().vector.into(),
                    power: radio.power,
                    msg,
                });
            }
        }
    }

    for channel in 0..NUM_CHANNELS {
        for rx in receivers.get(&channel).unwrap_or(&Vec::new()) {
            let mut best_msg = None;
            let mut best_rssi = rx.min_rssi;
            for tx in senders.get(&channel).unwrap_or(&Vec::new()) {
                let rssi = compute_rssi(tx, rx);
                if rssi > best_rssi {
                    best_rssi = rssi;
                    best_msg = Some(tx.msg);
                }
            }
            sim.ship_mut(rx.handle)
                .radio_mut(rx.radio_index)
                .as_mut()
                .unwrap()
                .received = best_msg;
        }
    }

    for handle in handle_snapshot.iter().cloned() {
        for radio in sim.ship_mut(handle).data_mut().radios.iter_mut() {
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
    use nalgebra::vector;
    use test_log::test;

    #[test]
    fn test_basic() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(
            &mut sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );
        let ship1 = ship::create(
            &mut sim,
            vector![1000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );

        sim.step();

        assert!(sim.ship(ship0).radio(0).unwrap().received.is_none());
        assert!(sim.ship(ship1).radio(0).unwrap().received.is_none());

        let msg = [42.0, 43.0, 44.0, 45.0];

        sim.ship_mut(ship1).radio_mut(0).unwrap().sent = Some(msg);

        sim.step();

        assert_eq!(sim.ship(ship0).radio(0).unwrap().received, Some(msg));
        assert_eq!(sim.ship(ship1).radio(0).unwrap().received, Some(msg));

        sim.step();

        assert!(sim.ship(ship0).radio(0).unwrap().received.is_none());
        assert!(sim.ship(ship1).radio(0).unwrap().received.is_none());
    }

    #[test]
    fn test_channel() {
        let mut sim = Simulation::new("test", 0, &[Code::None, Code::None]);

        // Initial state.
        let ship0 = ship::create(
            &mut sim,
            vector![0.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );
        let ship1 = ship::create(
            &mut sim,
            vector![1000.0, 0.0],
            vector![0.0, 0.0],
            0.0,
            ship::fighter(0),
        );

        let msg = [42.0, 43.0, 44.0, 45.0];

        sim.ship_mut(ship1).radio_mut(0).unwrap().sent = Some(msg);
        sim.ship_mut(ship1).radio_mut(0).unwrap().channel = 5;

        sim.step();

        assert_eq!(sim.ship(ship0).radio(0).unwrap().received, None);

        sim.ship_mut(ship0).radio_mut(0).unwrap().channel = 5;
        sim.ship_mut(ship1).radio_mut(0).unwrap().sent = Some(msg);

        sim.step();

        assert_eq!(sim.ship(ship0).radio(0).unwrap().received, Some(msg));
    }
}
