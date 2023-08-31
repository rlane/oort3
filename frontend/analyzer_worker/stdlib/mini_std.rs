#[macro_export]
macro_rules! format_args {
    ($fmt:expr) => {{ /* compiler built-in */ }};
    ($fmt:expr, $($args:tt)*) => {{ /* compiler built-in */ }};
}

pub mod f64 {
    pub mod consts {
        const PI: f64 = 0.0;
        const TAU: f64 = 0.0;
    }
}

pub mod prelude {
    pub use core::prelude::*;
}
