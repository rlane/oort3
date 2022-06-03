// tutorial09
// Destroy the enemy ships with your missiles.

fn tick() {
    if ship.class() == "missile" {
        missile_tick();
    } else {
        radar.set_width(2 * PI() / 60.0);
        let contact = radar.scan();
        if contact.found {
            turn_to((contact.position - ship.position()).angle(), 0.0);
            ship.launch_missile();
            radar.set_heading((contact.position - ship.position()).angle() - ship.heading());
        } else {
            radar.set_heading(rng.next(0.0, PI() * 2));
        }
    }
}

fn turn_to(target_heading, target_angular_velocity) {
    let acc = 4 * PI();
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
    if !contact.found {
        radar.set_width(2 * PI() / rng.next(0.0, 60.0));
        radar.set_heading(rng.next(0.0, PI() * 2));
        if ship.velocity().magnitude() < 1000.0 {
            ship.accelerate(vec2(100, 0));
        }
        return;
    } else {
        let dp = contact.position - ship.position();
        radar.set_width(2 * PI() * 20 / dp.magnitude());
        let dv = contact.velocity - ship.velocity();
        radar.set_heading((contact.position - ship.position() + dv/60).angle() - ship.heading() - ship.angular_velocity()/60);
    }

    let dp = contact.position - ship.position();
    let dv = contact.velocity - ship.velocity();

    let dist = dp.magnitude();
    let next_dist = (dp + dv / 60).magnitude();
    if next_dist < 10 || dist < 100 && next_dist > dist {
        ship.explode();
        return;
    }

    dp = dp.rotate(dp.magnitude() / 1e4);  // Evade guns.
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
