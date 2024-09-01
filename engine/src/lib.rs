use std::fmt;

pub mod state_space;
pub mod transfer_function;

/// Helper function for displaying floats in a certain format
fn write_float(f: &mut impl fmt::Write, num: f64) -> fmt::Result {
    // TODO: use sci notation for very small and large numbers
    // TODO: do not heap allocate
    let s = format!("{:.3}", num);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    write!(f, "{s}")
}
