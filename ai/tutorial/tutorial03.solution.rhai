// Tutorial 03
// Fly through the target circle. The target is in a random
// location given by the "target" variable.
// Hint: Look for a "position" method in the documenation, linked at
// the top of the screen.

fn tick() {
    ship.accelerate((target - ship.position()).normalize() * 100);
}
