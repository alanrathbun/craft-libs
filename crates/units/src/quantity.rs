//! Quantities: an SI-base-unit magnitude with a dimension.

use crate::dim::Dim;
use serde::{Deserialize, Serialize};

/// Why a unit operation failed.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UnitError {
    /// Two quantities of different dimensions were added or subtracted.
    #[error("dimension mismatch: {left:?} versus {right:?}")]
    Mismatch {
        /// Dimension of the left operand.
        left: Dim,
        /// Dimension of the right operand.
        right: Dim,
    },
    /// A dimension exponent left the `i8` range.
    #[error("dimension exponent overflow")]
    Overflow,
    /// The result was NaN or infinite.
    #[error("result is not a finite number")]
    NonFinite,
    /// A root or power would give a dimension a fractional exponent.
    #[error("dimension would have a fractional exponent")]
    Fractional,
    /// A dimensionless quantity was required.
    #[error("expected a dimensionless quantity, found {0:?}")]
    NotDimensionless(Dim),
    /// A unit name is not in the catalog.
    #[error("unknown unit `{0}`")]
    UnknownUnit(String),
    /// A unit expression could not be parsed.
    #[error("unit syntax error: {0}")]
    Syntax(String),
    /// An operation is not defined on an offset (affine) temperature scale.
    #[error("operation is not defined on an offset temperature scale")]
    AffineTemperature,
}

/// A magnitude in SI base units with its dimension.
///
/// The fields are public and `Quantity::new` does not validate, so a value may hold a non-finite
/// `f64`. Every arithmetic method checks its result and returns [`UnitError::NonFinite`] instead
/// of passing a NaN or infinity on.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    /// Magnitude in SI base units.
    pub value: f64,
    /// Dimension of the magnitude.
    pub dim: Dim,
}

// The arithmetic methods return `Result`, so they cannot be the `std::ops` trait methods of the same name.
#[allow(clippy::should_implement_trait)]
impl Quantity {
    /// A quantity from an SI-base magnitude and its dimension.
    pub fn new(value: f64, dim: Dim) -> Quantity {
        Quantity { value, dim }
    }

    /// A plain number.
    pub fn dimensionless(value: f64) -> Quantity {
        Quantity { value, dim: Dim::ONE }
    }

    /// Checked constructor for results: the value must be finite.
    fn finite(value: f64, dim: Dim) -> Result<Quantity, UnitError> {
        if value.is_finite() { Ok(Quantity { value, dim }) } else { Err(UnitError::NonFinite) }
    }

    fn same_dim(self, other: Quantity) -> Result<(), UnitError> {
        if self.dim.same_ignoring_angle(other.dim) { Ok(()) } else { Err(UnitError::Mismatch { left: self.dim, right: other.dim }) }
    }

    /// Sum. The dimensions must agree apart from angle. The result is the same whichever operand comes first: its angle exponent is the shared one if equal, else the non-zero one if only one is zero, else zero.
    pub fn add(self, other: Quantity) -> Result<Quantity, UnitError> {
        self.same_dim(other)?;
        Quantity::finite(self.value + other.value, sum_dim(self.dim, other.dim))
    }

    /// Difference. The dimensions must agree apart from angle. The result is the same whichever operand comes first: its angle exponent is the shared one if equal, else the non-zero one if only one is zero, else zero.
    pub fn sub(self, other: Quantity) -> Result<Quantity, UnitError> {
        self.same_dim(other)?;
        Quantity::finite(self.value - other.value, sum_dim(self.dim, other.dim))
    }

    /// Product. Exponents add.
    pub fn mul(self, other: Quantity) -> Result<Quantity, UnitError> {
        let dim = self.dim.mul(other.dim).ok_or(UnitError::Overflow)?;
        Quantity::finite(self.value * other.value, dim)
    }

    /// Quotient. Exponents subtract.
    pub fn div(self, other: Quantity) -> Result<Quantity, UnitError> {
        let dim = self.dim.div(other.dim).ok_or(UnitError::Overflow)?;
        Quantity::finite(self.value / other.value, dim)
    }

    /// Negation.
    pub fn neg(self) -> Result<Quantity, UnitError> {
        Quantity::finite(-self.value, self.dim)
    }

    /// Integer power. The dimension's exponents scale.
    pub fn powi(self, n: i32) -> Result<Quantity, UnitError> {
        let dim = self.dim.powi(n).ok_or(UnitError::Overflow)?;
        Quantity::finite(self.value.powi(n), dim)
    }

    /// Real power. An exponent that is an exact `i32` behaves like [`Quantity::powi`] for any base.
    /// Any other exponent needs a dimensionless base (angle ignored) and gives a plain number.
    pub fn powf(self, exponent: f64) -> Result<Quantity, UnitError> {
        if let Some(n) = exact_i32(exponent) {
            let dim = self.dim.powi(n).ok_or(UnitError::Overflow)?;
            return Quantity::finite(self.value.powf(exponent), dim);
        }
        if !self.dim.is_dimensionless() {
            return Err(UnitError::NotDimensionless(self.dim));
        }
        Quantity::finite(self.value.powf(exponent), Dim::ONE)
    }

    /// Square root. Every dimension exponent must be even.
    pub fn sqrt(self) -> Result<Quantity, UnitError> {
        let dim = self.dim.root(2).ok_or(UnitError::Fractional)?;
        Quantity::finite(self.value.sqrt(), dim)
    }
}

/// `x` as an `i32` when it is a finite integer in range.
fn exact_i32(x: f64) -> Option<i32> {
    // `as` saturates and maps NaN to 0, so the round trip is what proves exactness.
    let n = x as i32;
    if x.is_finite() && f64::from(n) == x { Some(n) } else { None }
}

/// Dimension of a sum or difference of two operands that agree apart from angle.
///
/// The result must not depend on operand order. Its angle exponent is the shared one when the
/// operands' are equal, the non-zero one when exactly one is zero, and zero when both are non-zero
/// and differ. Every other exponent is the operands' common value.
fn sum_dim(left: Dim, right: Dim) -> Dim {
    const ANGLE: usize = 7;
    let (a, b) = (left.exp[ANGLE], right.exp[ANGLE]);
    let angle = if a == b || b == 0 {
        a
    } else if a == 0 {
        b
    } else {
        0
    };
    let mut dim = left;
    dim.exp[ANGLE] = angle;
    dim
}
