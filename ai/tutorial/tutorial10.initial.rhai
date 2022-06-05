// tutorial10
// Destroy the enemy ships with your Frigate.

fn tick() {
    let contact = ship.scan();
    if contact.found {
        if ship.class() == "missile" {
            let dp = contact.position - ship.position();
            let dv = contact.velocity - ship.velocity();
            ship.torque(20 * (angle_diff(ship.heading(), dp.angle())
                - 0.1 * ship.angular_velocity()));
            ship.accelerate((dp+dv).rotate(-ship.heading()));
            if dp.magnitude() < 20 {
                ship.explode();
            }
        } else {
            ship.fire_weapon();
            ship.fire_weapon_with_index(1);
            ship.fire_weapon_with_index(2);
            ship.launch_missile();
        }
    }
}
