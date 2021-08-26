// tutorial08
// Destroy the enemy ships. They are initially outside of your radar range.

let initial_position = ship.position();
let target_position = initial_position;
let target_velocity = vec2(0.0, 0.0);
let last_target_heading = 0.0;

fn turn_to(target_heading, target_angular_velocity) {
    let acc = 2 * PI();
    let dh = angle_diff(ship.heading(), target_heading);
    let vh = ship.angular_velocity() - target_angular_velocity;
    let margin = 0.001;
    let t = abs(vh / acc);
    let pdh = vh * t + 0.5 * -acc * t*t - dh;
    if pdh < 0 {
        ship.torque(acc);
    } else if pdh > 0 {
        ship.torque(-acc);
    }
}

fn tick() {
    let contact = radar.scan();
    if (contact.found) {
        radar.set_heading((contact.position - ship.position()).angle() - ship.heading());
        target_position = contact.position;
        target_velocity = contact.velocity;
    } else {
        radar.set_heading(rng.next(0.0, PI() * 2));
        if (target_position - ship.position()).magnitude() < 100 {
            target_position = vec2(rng.next(3500.0, 4500.0), 0).rotate(rng.next(0.0, 2*PI()));
            target_velocity = vec2(0.0, 0.0);
        }
    }

    let dp = target_position - ship.position();
    let dv = target_velocity - ship.velocity();
    let bullet_speed = 1000.0;
    let bullet_offset = 20.0;
    let predicted_dp = dp;
    for i in range(0, 4) {
        let dist = predicted_dp.magnitude() - bullet_offset;
        let t = dist / bullet_speed;
        predicted_dp = dp + t * dv;
    }
    let target_heading = predicted_dp.angle();
    let target_angular_velocity = (target_heading - last_target_heading) * 60.0;
    turn_to(target_heading, target_angular_velocity);
    if contact.found && vec2(predicted_dp.magnitude(), 0).rotate(ship.heading()).distance(predicted_dp) <= 10 {
        ship.fire_weapon();
    }
    last_target_heading = target_heading;
    ship.accelerate((target_position - ship.position() - ship.velocity())
        .normalize().rotate(-ship.heading()) * 100.0);
}
