use oort_api::prelude::*;

pub struct Ship {}

impl Ship {
    pub fn new() -> Ship {
        Ship {}
    }

    pub fn tick(&mut self) {
        let testcase = oort_api::sys::getenv("TESTCASE").unwrap_or("none");
        match testcase {
            "scenario_name" => debug!("Scenario: {}", scenario_name()),
            "world_size" => debug!("World size: {}", world_size()),
            "id" => debug!("ID: {}", id()),
            _ => debug!("Unknown testcase: {:?}", testcase),
        }
    }
}
