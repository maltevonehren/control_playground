use nalgebra::{DMatrix, DVector, DVectorView, DVectorViewMut, RowDVector, SMatrix};
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
        write!(f, "A: {} ", self.a)?;
        write!(f, "B: {} ", self.b)?;
        write!(f, "C: {} ", self.c)?;
        write!(f, "D: {} ", self.d)?;
        Ok(())
    }
}

impl DiscreteStateSpaceModel {
    pub fn state_size(&self) -> usize {
        self.a.nrows()
    }

    pub fn update_state(&self, input: f64, mut state: DVectorViewMut<'_, f64>) {
        // TODO avoid temp alloc
        let new_state = &self.a * &state + &self.b * input;
        state.set_column(0, &new_state);
    }

    pub fn calculate_output(&self, input: f64, state: DVectorView<'_, f64>, output: &mut f64) {
        *output = (&self.c * state + self.d * input)[0];
    }

    pub fn has_feedthrough(&self) -> bool {
        self.d.iter().any(|e| *e != 0.0)
    }
}
