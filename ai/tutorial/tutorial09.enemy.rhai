let initial_position = ship.position();
let target = initial_position;

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.5 * ship.angular_velocity()));
}

fn tick() {
    if (target - ship.position()).magnitude() < 50 {
        target = initial_position + vec2(rng.next(0.0, 1000.0), 0).rotate(rng.next(0.0, 2*PI()));
    }
    ship.accelerate((target - ship.position() - ship.velocity()).rotate(-ship.heading()));
    let contact = radar.scan();
    if !contact.found {
        radar.set_heading(rng.next(0.0, PI() * 2));
    } else {
        radar.set_heading((contact.position - ship.position()).angle() - ship.heading());
    }
    if (contact.position.distance(ship.position()) < 1000.0 && contact.velocity.magnitude() > 100) {
        turn_to((contact.position - ship.position()).angle());
        ship.fire_weapon();
    } else {
        turn_to((target - ship.position()).angle());
    }
}
