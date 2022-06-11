// tutorial07
// Destroy the enemy ships.

fn tick() {
    let contact = ship.scan();
    ship.accelerate(0.1 * (contact.position - ship.position()));
    ship.fire_gun();
}
