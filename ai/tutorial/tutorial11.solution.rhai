// tutorial11
// Destroy the enemy ships with your Cruiser.

fn tick() {
    if ship.class() == "missile" || ship.class() == "torpedo" {
        missile_tick();
    } else {
        ship_tick();
    }
}

let initial_position = ship.position();
let target_position = initial_position;
let target_velocity = vec2(0.0, 0.0);

fn ship_tick() {
    let contact = radar.scan();
    if (contact.found) {
        let dp = contact.position - ship.position();
        let dv = contact.velocity - ship.velocity();
        let predicted_dp = dp + dv * dp.magnitude() / 1000.0;
        radar.set_heading(dp.angle() - ship.heading());
        target_position = contact.position;
        target_velocity = contact.velocity;
        ship.aim_weapon(0, (predicted_dp - vec2(0.0, 15.0).rotate(ship.heading())).angle() - ship.heading());
        ship.fire_weapon();
        for i in 0..3 {
            ship.launch_missile_with_index(i);
        }
        dbg.draw_diamond(contact.position, 20.0, 0xffff00);
        dbg.draw_diamond(ship.position() + predicted_dp, 5.0, 0xffffff);
    } else {
        radar.set_heading(rng.next(0.0, PI() * 2));
        if (target_position - ship.position()).magnitude() < 100 {
            target_position = vec2(rng.next(3500.0, 4500.0), 0).rotate(rng.next(0.0, 2*PI()));
            target_velocity = vec2(0.0, 0.0);
        }
    }
    let dp = target_position - ship.position();
    let dist = dp.magnitude();
    turn_to(dp.angle(), 0.0);

    if contact.found && dist < 1000 {
        ship.accelerate(-dp - ship.velocity().rotate(-ship.heading()));
    } else {
        ship.accelerate((dp - ship.velocity()).rotate(-ship.heading()));
    }
}

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

fn missile_tick() {
    let acc = 400;

    let contact = radar.scan();
    if (!contact.found) {
        radar.set_heading(rng.next(0.0, PI() * 2));
        return;
    }
    radar.set_heading((contact.position - ship.position()).angle() - ship.heading());

    let dp = contact.position - ship.position();
    let dv = contact.velocity - ship.velocity();

    let dist = dp.magnitude();
    let next_dist = (dp + dv / 60).magnitude();
    if next_dist < 30 || dist < 100 && next_dist > dist {
        ship.explode();
        return;
    }

    let badv = -(dv - dot(dv, dp) * dp.normalize() / dp.magnitude());
    let a = (dp - badv * 10).rotate(-ship.heading()).normalize() * acc;
    ship.accelerate(a);
    turn_to(a.rotate(ship.heading()).angle(), 0);

    dbg.draw_diamond(contact.position, 20.0, 0xffff00);
    dbg.draw_diamond(ship.position() + dp, 5.0, 0xffffff);
    dbg.draw_line(ship.position(), ship.position() + dp, 0x222222);
    dbg.draw_line(ship.position(), ship.position() - dv, 0xffffff);
    dbg.draw_line(ship.position(), ship.position() + badv, 0x222299);
}
