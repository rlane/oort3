// Tutorial 06
// Destroy the enemy ships. Use your radar to find them.
// Hint: Press 'g' in-game to show where your radar is looking.

fn tick() {
    radar.set_heading(rng.next(0.0, 2 * PI()));
    let contact = radar.scan();
    ship.accelerate(0.1 * (contact.position - ship.position()));
    ship.fire_weapon();
}
