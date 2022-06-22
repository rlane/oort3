pub mod native;
pub mod rhai;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

use self::native::NativeTeamController;
use self::rhai::RhaiTeamController;
use crate::ship::ShipHandle;
use crate::simulation::{Code, Simulation};
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

pub fn new_team_controller(code: &Code) -> Result<Box<dyn TeamController>, Error> {
    log::info!("Creating team controller with code {:?}", code);
    match code {
        Code::Rhai(s) => RhaiTeamController::create(s),
        Code::Native => NativeTeamController::create(),
        #[cfg(target_arch = "wasm32")]
        Code::Wasm(b) => {
            return WasmTeamController::create(b);
        }
        _ => unreachable!(),
    }
}
