use nalgebra::{DMatrix, DVector, RowDVector, SMatrix};
use std::fmt;

use crate::dynamic_system::DiscreteSystem;

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
        write!(f, "A: {} ", self.a)?;
        write!(f, "B: {} ", self.b)?;
        write!(f, "C: {} ", self.c)?;
        write!(f, "D: {} ", self.d)?;
        Ok(())
    }
}

impl DiscreteSystem for DiscreteStateSpaceModel {
    fn num_states(&self) -> usize {
        self.a.nrows()
    }

    fn update_states(&self, input: f64, mut states: nalgebra::DVectorViewMut<'_, f64>) {
        // TODO avoid temp alloc
        let new_states = &self.a * &states + &self.b * input;
        states.set_column(0, &new_states);
    }

    fn calculate_output(&self, input: f64, states: nalgebra::DVectorView<'_, f64>) -> f64 {
        let y = &self.c * states + self.d * input;
        y[0]
    }
}
