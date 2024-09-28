use nalgebra::{DMatrix, DVector};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ArxModelStructure {
    pub na: usize,
    pub nb: usize,
    pub nk: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArxModel {
    pub a: DVector<f64>,
    pub b: DVector<f64>,
    pub nk: usize,
}

impl ArxModelStructure {
    fn num_params(&self) -> usize {
        self.na + self.nb
    }

    fn maximum_delay(&self) -> usize {
        if self.nb > 0 {
            self.na.max(self.nb + self.nk - 1)
        } else {
            self.na
        }
    }

    fn build_regressor_set(&self, y: &DVector<f64>, u: &DVector<f64>, t: usize) -> DVector<f64> {
        assert!(y.len() == u.len() && t >= self.maximum_delay() && t <= y.len());
        let mut res = DVector::zeros(self.num_params());
        for i in 0..self.na {
            res[i] = y[t - i - 1];
        }
        for i in 0..self.nb {
            res[i + self.na] = u[t - i - self.nk];
        }
        res
    }

    fn to_model(&self, theta: &DVector<f64>) -> ArxModel {
        assert!(theta.len() == self.num_params());
        let mut a = DVector::zeros(self.na);
        let mut b = DVector::zeros(self.nb);
        for i in 0..self.na {
            a[i] = theta[i];
        }
        for i in 0..self.nb {
            b[i] = theta[i + self.na];
        }
        ArxModel { a, b, nk: self.nk }
    }
}

pub fn ident(structure: ArxModelStructure, y: &DVector<f64>, u: &DVector<f64>) -> ArxModel {
    assert!(y.len() == u.len());
    let delay = structure.maximum_delay();
    assert!(y.len() >= delay);
    let num_params = structure.num_params();
    let num_samples = y.len() - delay;
    let mut x_mat = DMatrix::zeros(num_samples, num_params);
    for i in 0..num_samples {
        let phi = structure.build_regressor_set(y, u, i + delay);
        x_mat.set_row(i, &phi.transpose());
    }
    let theta = (x_mat.transpose() * &x_mat)
        .qr()
        .solve(&(x_mat.transpose() * y.rows(delay, num_samples)))
        .unwrap();
    structure.to_model(&theta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regressor_set_construction() {
        let y = DVector::from_vec(vec![10.0, 11.0, 12.0, 13.0]);
        let u = DVector::from_vec(vec![20.0, 21.0, 22.0, 23.0]);

        let struc = ArxModelStructure {
            na: 1,
            nb: 1,
            nk: 1,
        };
        assert_eq!(
            struc.build_regressor_set(&y, &u, 1),
            DVector::from_vec(vec![10.0, 20.0])
        );
        assert_eq!(
            struc.build_regressor_set(&y, &u, 2),
            DVector::from_vec(vec![11.0, 21.0])
        );
        let struc = ArxModelStructure {
            na: 2,
            nb: 2,
            nk: 2,
        };
        assert_eq!(
            struc.build_regressor_set(&y, &u, 3),
            DVector::from_vec(vec![12.0, 11.0, 21.0, 20.0])
        );
    }

    #[test]
    fn test_delayed_input() {
        let y = DVector::from_vec(vec![0.0, 10.0, 15.0, 15.0]);
        let u = DVector::from_vec(vec![20.0, 30.0, 30.0, 30.0]);
        let struc = ArxModelStructure {
            na: 1,
            nb: 1,
            nk: 1,
        };
        assert_eq!(
            ident(struc, &y, &u),
            ArxModel {
                a: DVector::from_vec(vec![0.0]),
                b: DVector::from_vec(vec![0.5]),
                nk: struc.nk,
            }
        );
    }

    #[test]
    fn test_auto_regressive() {
        let y = DVector::from_vec(vec![16.0, 8.0, 4.0, 2.0]);
        let u = DVector::from_vec(vec![20.0, 30.0, 30.0, 30.0]);
        let struc = ArxModelStructure {
            na: 1,
            nb: 1,
            nk: 1,
        };
        assert_eq!(
            ident(struc, &y, &u),
            ArxModel {
                a: DVector::from_vec(vec![0.5]),
                b: DVector::from_vec(vec![0.0]),
                nk: struc.nk,
            }
        );
    }

    #[test]
    fn test_first_order() {
        let y = DVector::from_vec(vec![16.0, 18.0, 24.0, 27.0]);
        let u = DVector::from_vec(vec![20.0, 30.0, 30.0, 30.0]);
        let struc = ArxModelStructure {
            na: 1,
            nb: 1,
            nk: 1,
        };
        assert_eq!(
            ident(struc, &y, &u),
            ArxModel {
                a: DVector::from_vec(vec![0.5]),
                b: DVector::from_vec(vec![0.5]),
                nk: struc.nk,
            }
        );
    }
}
