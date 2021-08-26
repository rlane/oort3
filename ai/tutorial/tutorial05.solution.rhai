// Tutorial 05
// Destroy the enemy ship. Its location is given by the
// "target" variable.

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.33*ship.angular_velocity()));
}

fn tick() {
    turn_to((target - ship.position()).angle());
    ship.accelerate((target - ship.position() - ship.velocity())
        .normalize().rotate(-ship.heading()) * 200.0);
    ship.fire_weapon();
}
