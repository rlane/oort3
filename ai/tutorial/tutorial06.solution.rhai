// Tutorial 06
// Destroy the enemy ships. Use your radar to find them.
// Hint: Press 'g' in-game to show where your radar is looking.

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.33*ship.angular_velocity()));
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
        .normalize().rotate(-ship.heading()) * 200.0);
    ship.fire_weapon();
}
