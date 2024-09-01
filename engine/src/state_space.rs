use nalgebra::{DMatrix, DVector, RowDVector, SMatrix};

/// Discrete Time SISO State Space Model
///
/// x_(k+1) = a * x_k + b * u_k
/// y_k = c * x_k + d * u_k
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteStateSpace {
    pub(crate) a: DMatrix<f64>,
    pub(crate) b: DVector<f64>,
    pub(crate) c: RowDVector<f64>,
    pub(crate) d: SMatrix<f64, 1, 1>,
}
