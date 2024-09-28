use std::fmt;
use std::fmt::Write;

use nalgebra::{DMatrix, DVector, RowDVector, SMatrix};

use crate::{state_space::DiscreteStateSpaceModel, NiceFloat};

/// Discrete Time Transfer Function
///
/// Invariant: `num.len() > 0 && den.len() == num.len()`
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteTransferFunction {
    /// numerator polynomial.
    /// num[i] is the coefficient for z^(-i)
    num: DVector<f64>,
    /// numerator polynomial.
    /// den[j] is the coefficient for z^(-j)
    den: DVector<f64>,
}

impl DiscreteTransferFunction {
    pub fn new(mut num: DVector<f64>, mut den: DVector<f64>) -> Option<Self> {
        if num.is_empty() || den.is_empty() {
            return None;
        }
        let numlen = num.len();
        let denlen = den.len();
        if numlen < denlen {
            num = num.insert_rows(0, denlen - numlen, 0.0);
        }
        if denlen < numlen {
            den = den.insert_rows(0, numlen - denlen, 0.0);
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

        let mut a = DMatrix::zeros(order, order);
        let mut b = DVector::zeros(order);
        let mut c =
            RowDVector::from_iterator(order, self.den.iter().skip(1).map(|di| -di / d0 * n0 / d0));
        let d = SMatrix::from_element(n0 / d0);

        if order > 0 {
            a.row_mut(0)
                .iter_mut()
                .zip(self.den.iter().skip(1))
                .for_each(|(el, di)| *el = -di / d0);
            a.view_mut((1, 0), (order - 1, order - 1)).fill_diagonal(1.);

            b[0] = 1.;

            c.iter_mut()
                .zip(self.num.iter().skip(1))
                .for_each(|(el, ni)| *el += ni / d0);
        }

        // Matlab uses a rescaling step (prescale) here with diagonal T were the elements are powers of 2
        //     A' = inv(T) * A * T
        //     B' = inv(T) * B
        //     C' = C * T

        Some(DiscreteStateSpaceModel { a, b, c, d })
    }
}

impl fmt::Display for DiscreteTransferFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_poly(vals: &DVector<f64>) -> Result<String, fmt::Error> {
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
        let num = format_poly(&self.num)?;
        let den_is_one = self.den[0] == 1.0 && self.den.iter().skip(1).all(|e| *e == 0.0);
        let den = format_poly(&self.den)?;
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
            num: DVector::from_iterator(3, [-1.0, 1.5, -2.0].iter().copied()),
            den: DVector::from_iterator(3, [1.5, 0.5, 0.75].iter().copied()),
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
            num: DVector::from_iterator(3, [1.0, 1.5, 2.0].iter().copied()),
            den: DVector::from_iterator(3, [1.5, 0.5, 0.75].iter().copied()),
        };
        let ss = tf.convert_to_state_space().unwrap();
        assert_relative_eq!(
            ss.a,
            DMatrix::from_iterator(2, 2, [-1. / 3., 1., -0.5, 0.].iter().copied())
        );
        assert_relative_eq!(ss.b, DVector::from_iterator(2, [1., 0.].iter().copied()));
        assert_relative_eq!(
            ss.c,
            RowDVector::from_iterator(2, [7. / 9., 1.0].iter().copied())
        );
        assert_relative_eq!(ss.d, SMatrix::<f64, 1, 1>::from_element(2. / 3.));
    }

    #[test]
    fn state_space_conversion_gain_only() {
        let tf = DiscreteTransferFunction {
            num: DVector::from_iterator(1, [2.0].iter().copied()),
            den: DVector::from_iterator(1, [3.0].iter().copied()),
        };
        let ss = tf.convert_to_state_space().unwrap();
        assert_relative_eq!(ss.a, DMatrix::zeros(0, 0));
        assert_relative_eq!(ss.b, DVector::zeros(0));
        assert_relative_eq!(ss.c, RowDVector::zeros(0));
        assert_relative_eq!(ss.d, SMatrix::<f64, 1, 1>::from_element(2. / 3.));
    }
}
