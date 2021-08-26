// tutorial08
// Destroy the enemy ships. They are initially outside of your radar range.

fn tick() {
    let contact = ship.scan();
    ship.accelerate(0.1 * (contact.position - ship.position()));
    ship.fire_weapon();
}
