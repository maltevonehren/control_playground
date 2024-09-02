use nalgebra::{DMatrix, DVector, RowDVector, SMatrix};
use std::fmt;

/// Discrete Time SISO State Space Model
///
/// x_(k+1) = a * x_k + b * u_k
/// y_k = c * x_k + d * u_k
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteStateSpaceModel {
    pub(crate) a: DMatrix<f64>,
    pub(crate) b: DVector<f64>,
    pub(crate) c: RowDVector<f64>,
    pub(crate) d: SMatrix<f64, 1, 1>,
}

impl fmt::Display for DiscreteStateSpaceModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A: {}", self.a)?;
        write!(f, "B: {}", self.b)?;
        write!(f, "C: {}", self.c)?;
        writeln!(f, "D: {}", self.d)?;
        Ok(())
    }
}
