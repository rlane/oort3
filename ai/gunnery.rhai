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

let last_target_heading = 0.0;
let radar_width_divisor = 16;

fn tick() {
    let contact = radar.scan();
    if (contact.found) {
        let dp = contact.position - ship.position();
        let dv = contact.velocity - ship.velocity();
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
        if vec2(predicted_dp.magnitude(), 0).rotate(ship.heading()).distance(predicted_dp) <= 5 {
            ship.fire_gun();
        }
        let next_tick_dp = dp + dv / 60;
        radar.set_heading(next_tick_dp.angle() - ship.heading());
        dbg.draw_diamond(dp, 10.0, 0xffffff);
        last_target_heading = target_heading;
        if radar_width_divisor < 360 {
            radar_width_divisor += 1;
        }
    } else {
        if radar_width_divisor > 16 {
            radar_width_divisor *= 0.8;
        } else {
            radar.set_heading(rng.next(0.0, 2*PI()));
        }
    }
    radar.set_width(2 * PI() / radar_width_divisor);
}
