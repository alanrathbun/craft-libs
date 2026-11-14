use craft_units::{Dim, Quantity, UnitError};
use proptest::prelude::*;

#[test]
fn force_over_area_is_pressure() {
    let f = Quantity::new(10.0, Dim::MASS.mul(Dim::LENGTH).unwrap().div(Dim::TIME.powi(2).unwrap()).unwrap());
    let a = Quantity::new(2.0, Dim::LENGTH.powi(2).unwrap());
    let p = f.div(a).unwrap();
    assert_eq!(p.value, 5.0);
    assert_eq!(p.dim.exp, [-1, 1, -2, 0, 0, 0, 0, 0]);
}

#[test]
fn adding_length_to_mass_is_mismatch() {
    assert!(matches!(Quantity::new(1.0, Dim::LENGTH).add(Quantity::new(1.0, Dim::MASS)), Err(UnitError::Mismatch { .. })));
}

#[test]
fn sqrt_of_odd_dimension_is_fractional() {
    assert_eq!(Quantity::new(4.0, Dim::LENGTH).sqrt(), Err(UnitError::Fractional));
    assert_eq!(Quantity::new(4.0, Dim::LENGTH.powi(2).unwrap()).sqrt().unwrap(), Quantity::new(2.0, Dim::LENGTH));
}

#[test]
fn overflowing_exponent_is_error() {
    assert!(Dim::LENGTH.powi(200).is_none());
}

#[test]
fn non_finite_is_error() {
    assert_eq!(Quantity::dimensionless(1.0).div(Quantity::dimensionless(0.0)), Err(UnitError::NonFinite));
}

#[test]
fn angle_adds_to_dimensionless() {
    assert!(Quantity::new(1.0, Dim::ANGLE).add(Quantity::dimensionless(1.0)).is_ok());
}

#[test]
fn add_and_sub_keep_the_left_dimension() {
    let sum = Quantity::new(1.0, Dim::LENGTH).add(Quantity::new(2.0, Dim::LENGTH)).unwrap();
    assert_eq!(sum, Quantity::new(3.0, Dim::LENGTH));
    let diff = Quantity::new(5.0, Dim::LENGTH).sub(Quantity::new(2.0, Dim::LENGTH)).unwrap();
    assert_eq!(diff, Quantity::new(3.0, Dim::LENGTH));
    assert!(matches!(Quantity::new(1.0, Dim::LENGTH).sub(Quantity::new(1.0, Dim::TIME)), Err(UnitError::Mismatch { .. })));
}

#[test]
fn sum_and_difference_angle_does_not_depend_on_operand_order() {
    let rad = Quantity::new(1.0, Dim::ANGLE);
    let one = Quantity::dimensionless(1.0);
    assert_eq!(rad.add(one).unwrap().dim, Dim::ANGLE);
    assert_eq!(one.add(rad).unwrap().dim, Dim::ANGLE);
    assert_eq!(rad.sub(one).unwrap().dim, Dim::ANGLE);
    assert_eq!(one.sub(rad).unwrap().dim, Dim::ANGLE);
    // The non-angle exponents are untouched.
    let m_rad = Quantity::new(1.0, Dim::LENGTH.mul(Dim::ANGLE).unwrap());
    let m = Quantity::new(1.0, Dim::LENGTH);
    let want = Dim::LENGTH.mul(Dim::ANGLE).unwrap();
    assert_eq!(m.add(m_rad).unwrap().dim, want);
    assert_eq!(m_rad.add(m).unwrap().dim, want);
}

#[test]
fn sum_of_differing_nonzero_angle_exponents_drops_the_angle() {
    let rad = Quantity::new(1.0, Dim::ANGLE);
    let rad2 = Quantity::new(1.0, Dim::ANGLE.powi(2).unwrap());
    assert_eq!(rad.add(rad2).unwrap().dim, Dim::ONE);
    assert_eq!(rad2.add(rad).unwrap().dim, Dim::ONE);
    assert_eq!(rad.sub(rad2).unwrap().dim, Dim::ONE);
    assert_eq!(rad2.sub(rad).unwrap().dim, Dim::ONE);
    assert_eq!(rad2.add(rad2).unwrap().dim, Dim::ANGLE.powi(2).unwrap());
}

#[test]
fn mul_adds_exponents_and_neg_flips_sign() {
    let area = Quantity::new(3.0, Dim::LENGTH).mul(Quantity::new(2.0, Dim::LENGTH)).unwrap();
    assert_eq!(area, Quantity::new(6.0, Dim::LENGTH.powi(2).unwrap()));
    assert_eq!(area.neg().unwrap().value, -6.0);
}

#[test]
fn dimension_overflow_in_quantity_is_overflow_error() {
    let big = Quantity::new(1.0, Dim::LENGTH.powi(100).unwrap());
    assert_eq!(big.mul(big), Err(UnitError::Overflow));
    assert_eq!(big.powi(2), Err(UnitError::Overflow));
}

#[test]
fn non_finite_results_are_rejected() {
    assert_eq!(Quantity::new(f64::MAX, Dim::ONE).add(Quantity::new(f64::MAX, Dim::ONE)), Err(UnitError::NonFinite));
    assert_eq!(Quantity::new(f64::NAN, Dim::ONE).neg(), Err(UnitError::NonFinite));
    assert_eq!(Quantity::dimensionless(-1.0).sqrt(), Err(UnitError::NonFinite));
}

#[test]
fn powf_needs_dimensionless_base_unless_exponent_is_integer() {
    let two_m = Quantity::new(2.0, Dim::LENGTH);
    assert_eq!(two_m.powf(2.0).unwrap(), Quantity::new(4.0, Dim::LENGTH.powi(2).unwrap()));
    assert_eq!(two_m.powf(0.5), Err(UnitError::NotDimensionless(Dim::LENGTH)));
    assert_eq!(Quantity::dimensionless(4.0).powf(0.5).unwrap(), Quantity::dimensionless(2.0));
    assert_eq!(two_m.powf(f64::NAN), Err(UnitError::NotDimensionless(Dim::LENGTH)));
    assert_eq!(two_m.powf(1e30), Err(UnitError::NotDimensionless(Dim::LENGTH)));
}

#[test]
fn powi_scales_value_and_dimension() {
    let q = Quantity::new(2.0, Dim::LENGTH).powi(-3).unwrap();
    assert_eq!(q.value, 0.125);
    assert_eq!(q.dim.exp, [-3, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn dim_root_requires_divisible_exponents() {
    assert_eq!(Dim::LENGTH.powi(4).unwrap().root(2), Some(Dim::LENGTH.powi(2).unwrap()));
    assert_eq!(Dim::LENGTH.root(2), None);
    assert_eq!(Dim::LENGTH.root(0), None);
    assert_eq!(Dim::ONE.root(3), Some(Dim::ONE));
}

#[test]
fn dimensionless_ignores_angle() {
    assert!(Dim::ONE.is_dimensionless());
    assert!(Dim::ANGLE.is_dimensionless());
    assert!(!Dim::LENGTH.is_dimensionless());
    assert!(Dim::LENGTH.same_ignoring_angle(Dim::LENGTH.mul(Dim::ANGLE).unwrap()));
    assert!(!Dim::LENGTH.same_ignoring_angle(Dim::TIME));
}

#[test]
fn quantity_and_dim_implement_serde() {
    // Compile-time check of the derived bounds; no serde format crate is a dependency.
    fn assert_serde<T: serde::Serialize + for<'de> serde::Deserialize<'de>>() {}
    assert_serde::<Quantity>();
    assert_serde::<Dim>();
}

fn dim_strategy() -> impl Strategy<Value = Dim> {
    proptest::array::uniform8(-3i8..=3).prop_map(|exp| Dim { exp })
}

proptest! {
    #[test]
    fn mul_then_div_round_trips(a in dim_strategy(), b in dim_strategy()) {
        prop_assert_eq!(a.mul(b).unwrap().div(b), Some(a));
    }
}
