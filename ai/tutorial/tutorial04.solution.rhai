// Tutorial 04
// Destroy the asteroid. The target is in a random
// location given by the "target" variable.

fn tick() {
    let target_heading = (target - ship.position()).angle();
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.33 * ship.angular_velocity()));
    ship.fire_weapon();
}
