fn tick() {
    if ship.class() == "missile" {
        missile_tick();
    } else if ship.class() == "torpedo" {
        torpedo_tick();
    } else {
        ship_tick();
    }
}

let initial_position = ship.position();
let target_position = initial_position;
let target_velocity = vec2(0.0, 0.0);
let ticks = 0;

fn ship_tick() {
    ticks += 1;
    if ship.class() == "cruiser" {
        if ticks % 6 == 0 {
            radar.set_width(2 * PI());
        } else {
            radar.set_width(2 * PI() / 60);
            radar.set_heading(2 * PI() * (ticks/2) / 60.0 - ship.heading());
        }
    }

    let contact = radar.scan();
    if (contact.found) {
        let dp = contact.position - ship.position();
        let dv = contact.velocity - ship.velocity();
        let predicted_dp = dp;
        let bullet_speed = 1000.0;
        for i in 0..3 {
            predicted_dp = dp + dv * predicted_dp.magnitude() / bullet_speed;
        }
        radar.set_heading(dp.angle() - ship.heading());
        target_position = contact.position;
        target_velocity = contact.velocity;
        if ship.class() == "fighter" {
            ship.fire_gun();
            ship.launch_missile();
        } else if ship.class() == "frigate" {
            ship.fire_gun();
            ship.aim_gun(1, (predicted_dp - vec2(0.0, 15.0).rotate(ship.heading())).angle() - ship.heading());
            ship.fire_gun(1);
            ship.aim_gun(2, (predicted_dp - vec2(0.0, -15.0).rotate(ship.heading())).angle() - ship.heading());
            ship.fire_gun(2);
            ship.launch_missile();
        } else if ship.class() == "cruiser" {
            ship.aim_gun(0, predicted_dp.angle() - ship.heading());
            ship.fire_gun(0);
            for i in 0..2 {
                ship.launch_missile(i, #{x: contact.position.x, y: contact.position.y});
            }
            if contact.class == "frigate" || contact.class == "cruiser" {
                ship.launch_missile(2, #{x: contact.position.x, y: contact.position.y});
            }
            dbg.draw_diamond(contact.position, 30.0, 0xffff00);
        }
    } else {
        radar.set_heading(rng.next(0.0, PI() * 2));
        if (target_position - ship.position()).magnitude() < 100 {
            target_position = vec2(rng.next(3500.0, 4500.0), 0).rotate(rng.next(0.0, 2*PI()));
            target_velocity = vec2(0.0, 0.0);
        }
    }
    let dp = target_position - ship.position();
    let dist = dp.magnitude();
    let bullet_speed = 1000.0;
    if ship.class() == "frigate" {
        bullet_speed = 4000.0;
    }
    let t = dist / bullet_speed;
    let predicted_dp = dp + t * (target_velocity - ship.velocity());
    turn_to(predicted_dp.angle(), 0.0);

    if contact.found && dist < 1000 {
        ship.accelerate(-ship.velocity().rotate(-ship.heading()));
    } else {
        ship.accelerate((dp - ship.velocity()).rotate(-ship.heading()));
    }
}

fn turn_to(target_heading, target_angular_velocity) {
    let acc = 2 * PI();
    if ship.class() == "frigate" {
        acc = 2 * PI() / 6;
    } else if ship.class() == "cruiser" {
        acc = 2 * PI() / 16.0;
    }
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

let has_locked = false;
let no_contact_ticks = 0;

fn missile_tick() {
    let acc = 400;

    if !has_locked {
        target_position = vec2(orders.x, orders.y);
        radar.set_heading((target_position - ship.position()).angle() - ship.heading());
        radar.set_width(PI() * 2 / 32);
        dbg.draw_diamond(target_position, 20.0, 0xff0000);
    }

    let contact = radar.scan();
    if contact.found && ship.class() == "torpedo" && contact.class != "frigate" && contact.class != "cruiser" {
        contact = #{ found: false };
    }
    if (!contact.found) {
        if has_locked {
            radar.set_heading(rng.next(0.0, PI() * 2));
            radar.set_width(PI() * 2 / 6);
        } else {
            let dp = target_position - ship.position();
            turn_to(dp.angle(), 0.0);
            let a = dp.rotate(-ship.heading()).normalize() * acc;
            ship.accelerate(a);
        }
        return;
    }
    has_locked = true;
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

fn torpedo_tick() {
    let acc = 1000;
    target_velocity = ship.velocity();

    if ticks == 0 {
        target_position = vec2(orders.x, orders.y);
    }
    ticks += 1;

    let target_heading = (target_position - ship.position()).angle();
    radar.set_heading(target_heading - ship.heading() + rng.next(-PI(), PI()) * (no_contact_ticks / 600.0));
    if (target_position - ship.position()).magnitude() < 200 {
        radar.set_width(PI() * 2 / 6);
    } else {
        radar.set_width(PI() * 2 / 60);
    }

    let contact = radar.scan();
    if contact.found && contact.class != "frigate" && contact.class != "cruiser" && contact.class != "unknown" {
        contact = #{ found: false };
    }
    if contact.found {
        target_position = contact.position;
        target_velocity = contact.velocity;
        no_contact_ticks = 0;
    } else {
        target_position += target_velocity / 60.0;
        no_contact_ticks += 1;
    }

    let dp = target_position - ship.position();
    let dv = target_velocity - ship.velocity();

    if contact.found {
        let dist = dp.magnitude();
        let next_dist = (dp + dv / 60).magnitude();
        if next_dist < 60 || dist < 100 && next_dist > dist {
            ship.explode();
            return;
        }
    } else {
        acc = acc / 10;
    }

    let predicted_position = target_position + target_velocity * (dp.magnitude() / 8000);
    let pdp = predicted_position - ship.position();

    let badv = -(dv - dot(dv, pdp) * pdp.normalize() / pdp.magnitude());
    let a = (pdp - badv * 10).rotate(-ship.heading()).normalize() * acc;
    ship.accelerate(a);
    turn_to(a.rotate(ship.heading()).angle(), 0);

    if no_contact_ticks > 0 {
        dbg.draw_diamond(target_position, 20.0, 0xff0000);
    } else {
        dbg.draw_diamond(contact.position, 20.0, 0xffff00);
        dbg.draw_diamond(ship.position() + pdp, 5.0, 0xffffff);
    }

    dbg.draw_line(ship.position(), ship.position() + dp, 0x222222);
    dbg.draw_line(ship.position(), ship.position() - dv, 0xffffff);
    dbg.draw_line(ship.position(), ship.position() + badv, 0x222299);
}
