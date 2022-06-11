// tutorial07
// Destroy the enemy ships.

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.1*ship.angular_velocity()));
}

fn tick() {
    let contact = radar.scan();
    if !contact.found {
        radar.set_heading(rng.next(0.0, PI() * 2));
    } else {
        radar.set_heading((contact.position - ship.position()).angle() - ship.heading());
    }
    turn_to((contact.position - ship.position()).angle());
    ship.accelerate((contact.position - ship.position() - ship.velocity())
        .normalize().rotate(-ship.heading()) * 100.0);
    ship.fire_gun();
}
