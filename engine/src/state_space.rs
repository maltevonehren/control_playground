use ndarray::prelude::*;
use ndarray::Data;
use std::fmt;

/// Discrete Time MIMO State Space Model
///
/// x_(k+1) = a * x_k + b * u_k
/// y_k = c * x_k + d * u_k
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteStateSpaceModel {
    data: Array2<f64>,
    n: usize,
}

impl DiscreteStateSpaceModel {
    pub fn new<
        S1: Data<Elem = f64>,
        S2: Data<Elem = f64>,
        S3: Data<Elem = f64>,
        S4: Data<Elem = f64>,
    >(
        a: ArrayBase<S1, Ix2>,
        b: ArrayBase<S2, Ix2>,
        c: ArrayBase<S3, Ix2>,
        d: ArrayBase<S4, Ix2>,
    ) -> Self {
        let n = a.nrows();
        let m = b.ncols();
        let r = c.nrows();
        if a.ncols() != n || b.nrows() != n || c.ncols() != n || d.ncols() != m || d.nrows() != r {
            panic!();
        }
        let mut data = Array2::zeros([n + r, n + m]);
        data.slice_mut(s![..n, ..n]).assign(&a);
        data.slice_mut(s![..n, n..]).assign(&b);
        data.slice_mut(s![n.., ..n]).assign(&c);
        data.slice_mut(s![n.., n..]).assign(&d);
        Self { data, n }
    }

    pub fn state_size(&self) -> usize {
        self.n
    }
    pub fn input_size(&self) -> usize {
        self.data.ncols() - self.n
    }
    pub fn output_size(&self) -> usize {
        self.data.nrows() - self.n
    }

    pub fn a(&self) -> ArrayView2<'_, f64> {
        self.data.slice(s![..self.n, ..self.n])
    }
    pub fn b(&self) -> ArrayView2<'_, f64> {
        self.data.slice(s![..self.n, self.n..])
    }
    pub fn c(&self) -> ArrayView2<'_, f64> {
        self.data.slice(s![self.n.., ..self.n])
    }
    pub fn d(&self) -> ArrayView2<'_, f64> {
        self.data.slice(s![self.n.., self.n..])
    }

    pub fn update_state(&self, input: ArrayView1<'_, f64>, mut state: ArrayViewMut1<'_, f64>) {
        let new_state = self.b().dot(&input) + self.a().dot(&state);
        state.assign(&new_state);
    }

    pub fn calculate_output(
        &self,
        input: ArrayView1<'_, f64>,
        state: ArrayView1<'_, f64>,
        mut output: ArrayViewMut1<'_, f64>,
    ) {
        output.assign(&(self.c().dot(&state) + self.d().dot(&input)));
    }

    pub fn has_feedthrough(&self) -> bool {
        self.d().iter().any(|e| *e != 0.0)
    }
}

impl fmt::Display for DiscreteStateSpaceModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A: {} ", self.a())?;
        write!(f, "B: {} ", self.b())?;
        write!(f, "C: {} ", self.c())?;
        write!(f, "D: {} ", self.d())?;
        Ok(())
    }
}
