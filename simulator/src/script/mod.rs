pub mod native;
pub mod rhai;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

use self::native::NativeTeamController;
use self::rhai::RhaiTeamController;
use crate::ship::ShipHandle;
use crate::simulation::Simulation;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use self::wasm::WasmTeamController;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub line: usize,
    pub msg: String,
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for Error {
    fn from(err: wasm_bindgen::JsValue) -> Self {
        Self {
            line: 0,
            msg: format!("JS error: {:?}", err),
        }
    }
}

/*
#[cfg(target_arch = "wasm32")]
impl From<wasmer::CompileError> for Error {
    fn from(err: wasmer::CompileError) -> Self {
        Self {
            line: 0,
            msg: format!("Wasmer compile error: {:?}", err),
        }
    }
}
*/

#[cfg(target_arch = "wasm32")]
impl From<wasmer::InstantiationError> for Error {
    fn from(err: wasmer::InstantiationError) -> Self {
        Self {
            line: 0,
            msg: format!("Wasmer instantiation error: {:?}", err),
        }
    }
}

pub trait TeamController {
    fn create_ship_controller(
        &mut self,
        handle: ShipHandle,
        sim: &mut Simulation,
        orders: String,
    ) -> Result<Box<dyn ShipController>, Error>;
}

pub trait ShipController {
    fn tick(&mut self) -> Result<(), Error>;
    fn write_target(&mut self, target: Vector2<f64>);
}

pub fn new_team_controller(code: &str) -> Result<Box<dyn TeamController>, Error> {
    if code.starts_with("native") {
        NativeTeamController::create()
    } else if code.starts_with("wasm") {
        #[cfg(target_arch = "wasm32")]
        return WasmTeamController::create();
        #[cfg(not(target_arch = "wasm32"))]
        unreachable!();
    } else {
        RhaiTeamController::create(code)
    }
}
