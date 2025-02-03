use ndarray::prelude::*;
use std::fmt;
use std::fmt::Write;

use crate::{state_space::DiscreteStateSpaceModel, NiceFloat};

/// Discrete Time Transfer Function
///
/// Invariant: `num.len() > 0 && den.len() == num.len()`
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteTransferFunction {
    /// numerator polynomial.
    /// num[i] is the coefficient for z^(-i)
    num: Array1<f64>,
    /// numerator polynomial.
    /// den[j] is the coefficient for z^(-j)
    den: Array1<f64>,
}

impl DiscreteTransferFunction {
    pub fn new(mut num: Array1<f64>, mut den: Array1<f64>) -> Option<Self> {
        if num.is_empty() || den.is_empty() {
            return None;
        }
        let num_len = num.len();
        let den_len = den.len();
        if num_len < den_len {
            num.append(Axis(0), Array::zeros(den_len - num_len).view())
                .unwrap();
        }
        if den_len < num_len {
            den.append(Axis(0), Array::zeros(num_len - den_len).view())
                .unwrap();
        }
        Some(Self { num, den })
    }

    pub fn convert_to_state_space(&self) -> Option<DiscreteStateSpaceModel> {
        let d0 = self.den[0]; // normalization coeff
        if d0 == 0. {
            return None;
        }
        let order = self.den.len() - 1;
        let n0 = self.num[0];

        let mut a = Array2::zeros([order, order]);
        let mut b = Array1::zeros(order);
        let mut c = Array1::from_iter(self.den.iter().skip(1).map(|di| -di / d0 * n0 / d0));
        let d = Array0::from_elem([], n0 / d0);

        if order > 0 {
            a.row_mut(0)
                .iter_mut()
                .zip(self.den.iter().skip(1))
                .for_each(|(el, di)| *el = -di / d0);
            a.slice_mut(s![1.., ..order])
                .diag_mut()
                .mapv_inplace(|_| 1.0);

            b[0] = 1.;

            c.iter_mut()
                .zip(self.num.iter().skip(1))
                .for_each(|(el, ni)| *el += ni / d0);
        }

        // Matlab uses a rescaling step (prescale) here with diagonal T were the elements are powers of 2
        //     A' = inv(T) * A * T
        //     B' = inv(T) * B
        //     C' = C * T

        Some(DiscreteStateSpaceModel::new(
            a,
            b.insert_axis(Axis(1)),
            c.insert_axis(Axis(0)),
            d.insert_axis(Axis(0)).insert_axis(Axis(0)),
        ))
    }
}

impl fmt::Display for DiscreteTransferFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_poly(vals: ArrayView1<'_, f64>) -> Result<String, fmt::Error> {
            let mut out = String::new();
            let mut written = false;
            for (i, el) in vals.iter().enumerate() {
                if *el == 0.0 {
                    continue;
                }
                if *el != 1.0 || i == 0 {
                    if written {
                        if *el < 0. {
                            write!(out, " - {}", NiceFloat(el.abs()))?;
                        } else {
                            write!(out, " + {}", NiceFloat(el.abs()))?;
                        }
                    } else {
                        write!(out, "{}", NiceFloat(*el))?;
                    }
                }
                if i > 0 {
                    write!(out, " z^-{}", i)?;
                }
                written = true;
            }
            if !written {
                write!(out, "0")?;
            }
            Ok(out)
        }
        let num = format_poly(self.num.view())?;
        let den_is_one = self.den[0] == 1.0 && self.den.iter().skip(1).all(|e| *e == 0.0);
        let den = format_poly(self.den.view())?;
        let mut len = num.len();
        if !den_is_one {
            len = len.max(den.len())
        };
        writeln!(f, "{}{}", " ".repeat((len - num.len()) / 2), num)?;
        if !den_is_one {
            writeln!(f, "{}", "-".repeat(len))?;
            writeln!(f, "{}{}", " ".repeat((len - den.len()) / 2), den)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn tf_display() {
        let tf = DiscreteTransferFunction {
            num: array![-1.0, 1.5, -2.0],
            den: array![1.5, 0.5, 0.75],
        };
        let out = format!("{tf}");
        assert_eq!(
            &out,
            "  -1 + 1.5 z^-1 - 2 z^-2\n--------------------------\n1.5 + 0.5 z^-1 + 0.75 z^-2\n"
        );
    }

    #[test]
    fn state_space_conversion() {
        let tf = DiscreteTransferFunction {
            num: array![1.0, 1.5, 2.0],
            den: array![1.5, 0.5, 0.75],
        };
        let ss = tf.convert_to_state_space().unwrap();
        assert_relative_eq!(ss.a(), array![[-1. / 3., 1.], [-0.5, 0.]]);
        assert_relative_eq!(ss.b(), array![[1.0], [0.0]]);
        assert_relative_eq!(ss.c(), array![[7.0 / 9.0, 1.0]]);
        assert_relative_eq!(ss.d(), array![[2.0 / 3.0]]);
    }

    #[test]
    fn state_space_conversion_gain_only() {
        let tf = DiscreteTransferFunction {
            num: array![2.0],
            den: array![3.0],
        };
        let ss = tf.convert_to_state_space().unwrap();
        assert_relative_eq!(ss.a(), Array2::zeros((0, 0)));
        assert_relative_eq!(ss.b(), Array2::zeros((0, 1)));
        assert_relative_eq!(ss.c(), Array2::zeros((1, 0)));
        assert_relative_eq!(ss.d(), array![[2.0 / 3.0]]);
    }
}
