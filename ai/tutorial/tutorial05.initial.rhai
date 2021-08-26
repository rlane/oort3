// Tutorial 05
// Destroy the enemy ship. Its location is given by the
// "target" variable.

fn tick() {
    ship.accelerate(0.1 * (target - ship.position()));
    ship.fire_weapon();
}
