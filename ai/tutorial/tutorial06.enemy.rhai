let target = ship.position();

fn turn_to(target_heading) {
    ship.torque(20 * (angle_diff(ship.heading(), target_heading)
        - 0.33 * ship.angular_velocity()));
}

fn tick() {
    if (target - ship.position()).magnitude() < 50 {
        target = vec2(rng.next(200.0, 500.0), 0).rotate(rng.next(0.0, 2*PI()));
    }
    ship.accelerate((target - ship.position() - ship.velocity()).rotate(-ship.heading()));
    turn_to((target - ship.position()).angle());
}
