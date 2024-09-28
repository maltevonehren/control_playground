use std::fmt;

// pub mod arx;
pub mod dynamic_system;
pub mod state_space;
pub mod transfer_function;

/// Helper for displaying floats in a certain format
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct NiceFloat(pub f64);

impl fmt::Display for NiceFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: use sci notation for very small and large numbers
        // TODO: do not heap allocate
        let s = format!("{:.3}", self.0);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        write!(f, "{s}")
    }
}
