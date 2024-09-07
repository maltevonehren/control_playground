use nalgebra::{DMatrix, DVector, Matrix2xX, RowDVector, SMatrix, Vector2};
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

    pub fn step(&self) -> Matrix2xX<f64> {
        let mut x = DVector::zeros(self.state_size());
        let steps = 35;
        let mut output = Matrix2xX::zeros(steps + 1);
        let u = 1.0;
        for i in 0..steps {
            // calculate y first so we can update x in place
            let y = &self.c * &x + self.d * u;
            x = &self.a * x + &self.b * u;
            output.set_column(i, &Vector2::new(i as f64, y[0]));
        }
        let y = &self.c * &x + self.d * u;
        output.set_column(steps, &Vector2::new(steps as f64, y[0]));
        output
    }
}
