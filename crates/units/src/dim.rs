//! Physical dimensions as integer exponent vectors.

use serde::{Deserialize, Serialize};

/// Index of the angle exponent in [`Dim::exp`].
const ANGLE_INDEX: usize = 7;

/// A physical dimension: integer exponents over `[L, M, T, I, Θ, N, J, angle]`.
///
/// Index `0` is length, `1` mass, `2` time, `3` electric current, `4` thermodynamic temperature,
/// `5` amount of substance, `6` luminous intensity, and `7` plane angle. Angle is tracked so a
/// radian can be told from a plain number, yet it is ignored by [`Dim::is_dimensionless`] and
/// [`Dim::same_ignoring_angle`]. Every combining operation returns `None` when an exponent would
/// leave the `i8` range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dim {
    /// Exponent of each base dimension, in the order documented on [`Dim`].
    pub exp: [i8; 8],
}

// `mul` and `div` return `Option`, so they cannot be the `std::ops` trait methods of the same name.
#[allow(clippy::should_implement_trait)]
impl Dim {
    /// The dimension of a plain number.
    pub const ONE: Dim = Dim { exp: [0; 8] };
    /// Length (metre).
    pub const LENGTH: Dim = Dim::base(0);
    /// Mass (kilogram).
    pub const MASS: Dim = Dim::base(1);
    /// Time (second).
    pub const TIME: Dim = Dim::base(2);
    /// Electric current (ampere).
    pub const CURRENT: Dim = Dim::base(3);
    /// Thermodynamic temperature (kelvin).
    pub const TEMPERATURE: Dim = Dim::base(4);
    /// Amount of substance (mole).
    pub const AMOUNT: Dim = Dim::base(5);
    /// Luminous intensity (candela).
    pub const LUMINOSITY: Dim = Dim::base(6);
    /// Plane angle (radian).
    pub const ANGLE: Dim = Dim::base(ANGLE_INDEX);

    const fn base(index: usize) -> Dim {
        let mut exp = [0i8; 8];
        exp[index] = 1;
        Dim { exp }
    }

    /// The dimension of a product: exponents add. `None` on `i8` overflow.
    pub fn mul(self, other: Dim) -> Option<Dim> {
        let mut exp = [0i8; 8];
        for (out, (a, b)) in exp.iter_mut().zip(self.exp.iter().zip(other.exp.iter())) {
            *out = a.checked_add(*b)?;
        }
        Some(Dim { exp })
    }

    /// The dimension of a quotient: exponents subtract. `None` on `i8` overflow.
    pub fn div(self, other: Dim) -> Option<Dim> {
        let mut exp = [0i8; 8];
        for (out, (a, b)) in exp.iter_mut().zip(self.exp.iter().zip(other.exp.iter())) {
            *out = a.checked_sub(*b)?;
        }
        Some(Dim { exp })
    }

    /// The dimension raised to an integer power. `None` on `i8` overflow.
    pub fn powi(self, n: i32) -> Option<Dim> {
        let mut exp = [0i8; 8];
        for (out, e) in exp.iter_mut().zip(self.exp.iter()) {
            *out = i8::try_from(i32::from(*e).checked_mul(n)?).ok()?;
        }
        Some(Dim { exp })
    }

    /// The `n`th root. `None` when `n` is zero or any exponent is not divisible by `n`.
    pub fn root(self, n: u8) -> Option<Dim> {
        if n == 0 {
            return None;
        }
        let n = i32::from(n);
        let mut exp = [0i8; 8];
        for (out, e) in exp.iter_mut().zip(self.exp.iter()) {
            let e = i32::from(*e);
            if e % n != 0 {
                return None;
            }
            *out = i8::try_from(e / n).ok()?;
        }
        Some(Dim { exp })
    }

    /// `true` when every exponent but the angle's is zero.
    pub fn is_dimensionless(self) -> bool {
        self.same_ignoring_angle(Dim::ONE)
    }

    /// `true` when the two dimensions differ at most in their angle exponent.
    pub fn same_ignoring_angle(self, other: Dim) -> bool {
        self.exp.iter().zip(other.exp.iter()).take(ANGLE_INDEX).all(|(a, b)| a == b)
    }
}
