// tutorial11
// Destroy the enemy ships with your Cruiser.

fn tick() {
    let contact = ship.scan();
    if contact.found {
        if ship.class() == "missile" || ship.class == "torpedo" {
            let dp = contact.position - ship.position();
            let dv = contact.velocity - ship.velocity();
            ship.torque(20 * (angle_diff(ship.heading(), dp.angle())
                - 0.1 * ship.angular_velocity()));
            ship.accelerate((dp+dv).rotate(-ship.heading()));
            if dp.magnitude() < 20 {
                ship.explode();
            }
        } else {
            ship.fire_gun();
            for i in 0..3 {
                ship.launch_missile(i);
            }
        }
    }
}
