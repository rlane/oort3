pub struct Ship {}

#[allow(clippy::new_without_default)]
impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {}
}
