use craft_units::{Dim, Precision, Quantity, UnitError, UnitSystem, format, natural_unit, parse_unit};

const PSI: f64 = 6894.757293168361;

fn stress(psi: f64) -> Quantity {
    Quantity::new(psi * PSI, parse_unit("psi").unwrap().dim)
}

fn num(x: f64, decimals: u8) -> String {
    format(Quantity::dimensionless(x), None, UnitSystem::Si, Precision { decimals }).unwrap().number
}

#[test]
fn stress_in_psi_us() {
    let f = format(stress(600.0), None, UnitSystem::UsCustomary, Precision::default()).unwrap();
    assert_eq!((f.number.as_str(), f.unit.as_str()), ("600", "psi"));
}

#[test]
fn stress_in_pa_si() {
    let f = format(Quantity::new(2.5e5, stress(1.0).dim), None, UnitSystem::Si, Precision::default()).unwrap();
    assert_eq!((f.number.as_str(), f.unit.as_str()), ("250000", "Pa"));
}

#[test]
fn explicit_unit() {
    let f = format(stress(600.0), Some(&parse_unit("ksi").unwrap()), UnitSystem::UsCustomary, Precision::default()).unwrap();
    assert_eq!((f.number.as_str(), f.unit.as_str()), ("0.6", "ksi"));
}

#[test]
fn precision_and_trim() {
    let q = Quantity::dimensionless(2.0 / 3.0);
    assert_eq!(format(q, None, UnitSystem::Si, Precision { decimals: 3 }).unwrap().number, "0.667");
}

#[test]
fn exponential() {
    assert_eq!(format(Quantity::dimensionless(6.0e6), None, UnitSystem::Si, Precision::default()).unwrap().number, "6×10^6");
}

#[test]
fn wrong_display_unit() {
    let q = Quantity::new(1.0, Dim::LENGTH);
    assert!(format(q, Some(&parse_unit("psi").unwrap()), UnitSystem::Si, Precision::default()).is_err());
}

#[test]
fn wrong_display_unit_is_mismatch() {
    let q = Quantity::new(1.0, Dim::LENGTH);
    let err = format(q, Some(&parse_unit("psi").unwrap()), UnitSystem::Si, Precision::default()).unwrap_err();
    assert!(matches!(err, UnitError::Mismatch { .. }));
}

#[test]
fn fallback_composition() {
    let d = Dim::MASS.div(Dim::LENGTH.mul(Dim::TIME).unwrap()).unwrap();
    assert_eq!(natural_unit(d, UnitSystem::UsCustomary).text, "lbm/(in·s)");
    assert_eq!(natural_unit(d, UnitSystem::Si).text, "kg/(m·s)");
}

#[test]
fn temperature_display_in_degf() {
    let q = Quantity::new(373.15, Dim::TEMPERATURE);
    assert_eq!(format(q, Some(&parse_unit("°F").unwrap()), UnitSystem::UsCustomary, Precision::default()).unwrap().number, "212");
}

#[test]
fn defaults() {
    assert_eq!(UnitSystem::default(), UnitSystem::UsCustomary);
    assert_eq!(Precision::default().decimals, 3);
}

#[test]
fn natural_table_text_parses_with_matching_dim() {
    let l = Dim::LENGTH;
    let dims = [
        l,
        l.powi(2).unwrap(),
        l.powi(3).unwrap(),
        Dim::MASS,
        Dim { exp: [1, 1, -2, 0, 0, 0, 0, 0] },
        Dim { exp: [-1, 1, -2, 0, 0, 0, 0, 0] },
        Dim { exp: [2, 1, -2, 0, 0, 0, 0, 0] },
        Dim { exp: [2, 1, -3, 0, 0, 0, 0, 0] },
        Dim::TIME,
        Dim::TEMPERATURE,
        Dim::ONE,
        Dim::ANGLE,
        Dim { exp: [1, 0, -2, 0, 0, 0, 0, 0] },
        Dim { exp: [-3, 1, 0, 0, 0, 0, 0, 0] },
        Dim { exp: [0, 0, -1, 0, 0, 0, 0, 1] },
        Dim { exp: [4, 0, 0, 0, 0, 0, 0, 0] },
        Dim { exp: [0, 0, 0, 0, 0, 0, 0, 2] },
    ];
    for sys in [UnitSystem::UsCustomary, UnitSystem::Si] {
        for d in dims {
            let s = natural_unit(d, sys);
            if d.is_dimensionless() && d.exp[7] == 0 {
                assert_eq!(s.text, "");
                assert_eq!((s.factor, s.offset), (1.0, 0.0));
                continue;
            }
            let parsed = parse_unit(&s.text).unwrap_or_else(|e| panic!("`{}` does not parse: {e}", s.text));
            assert!(parsed.dim.same_ignoring_angle(d), "`{}` has wrong dim for {d}", s.text);
            assert!(s.dim.same_ignoring_angle(d));
            assert_eq!(parsed.factor, s.factor, "`{}`", s.text);
            assert_eq!(parsed.offset, 0.0);
            assert_eq!(s.offset, 0.0);
        }
    }
}

#[test]
fn natural_table_texts() {
    let l = Dim::LENGTH;
    let cases: [(Dim, &str, &str); 11] = [
        (l, "in", "m"),
        (l.powi(2).unwrap(), "in²", "m²"),
        (l.powi(3).unwrap(), "in³", "m³"),
        (Dim::MASS, "lbm", "kg"),
        (Dim { exp: [1, 1, -2, 0, 0, 0, 0, 0] }, "lbf", "N"),
        (Dim { exp: [-1, 1, -2, 0, 0, 0, 0, 0] }, "psi", "Pa"),
        (Dim { exp: [2, 1, -2, 0, 0, 0, 0, 0] }, "lbf·in", "J"),
        (Dim { exp: [2, 1, -3, 0, 0, 0, 0, 0] }, "hp", "W"),
        (Dim::TIME, "s", "s"),
        (Dim::TEMPERATURE, "°R", "K"),
        (Dim::ONE, "", ""),
    ];
    for (d, us, si) in cases {
        assert_eq!(natural_unit(d, UnitSystem::UsCustomary).text, us);
        assert_eq!(natural_unit(d, UnitSystem::Si).text, si);
    }
}

#[test]
fn fallback_powers_and_ordering() {
    let accel = Dim { exp: [1, 0, -2, 0, 0, 0, 0, 0] };
    assert_eq!(natural_unit(accel, UnitSystem::Si).text, "m/s²");
    assert_eq!(natural_unit(accel, UnitSystem::UsCustomary).text, "in/s²");
    let freq = Dim { exp: [0, 0, -1, 0, 0, 0, 0, 0] };
    assert_eq!(natural_unit(freq, UnitSystem::Si).text, "1/s");
    let density = Dim { exp: [-3, 1, 0, 0, 0, 0, 0, 0] };
    assert_eq!(natural_unit(density, UnitSystem::UsCustomary).text, "lbm/in³");
    let l4 = Dim { exp: [4, 0, 0, 0, 0, 0, 0, 0] };
    assert_eq!(natural_unit(l4, UnitSystem::Si).text, "m^4");
    let rate = Dim { exp: [0, 0, -1, 0, 0, 0, 0, 1] };
    assert_eq!(natural_unit(rate, UnitSystem::Si).text, "rad/s");
    let many = Dim { exp: [1, 1, 0, 1, -1, -1, 0, 0] };
    assert_eq!(natural_unit(many, UnitSystem::Si).text, "m·kg·A/(K·mol)");
    let high = Dim { exp: [0, 0, -12, 0, 0, 0, 0, 0] };
    assert_eq!(natural_unit(high, UnitSystem::Si).text, "1/s^12");
}

#[test]
fn fallback_factor_is_consistent() {
    let d = Dim::MASS.div(Dim::LENGTH.mul(Dim::TIME).unwrap()).unwrap();
    let s = natural_unit(d, UnitSystem::UsCustomary);
    let p = parse_unit(&s.text).unwrap();
    assert!((s.factor - p.factor).abs() <= 1e-12 * p.factor);
}

#[test]
fn pure_angle_natural_units() {
    let si = natural_unit(Dim::ANGLE, UnitSystem::Si);
    assert_eq!((si.text.as_str(), si.factor), ("rad", 1.0));
    let us = natural_unit(Dim::ANGLE, UnitSystem::UsCustomary);
    assert_eq!(us.text, "deg");
    let f = format(Quantity::new(std::f64::consts::PI, Dim::ANGLE), None, UnitSystem::UsCustomary, Precision::default()).unwrap();
    assert_eq!((f.number.as_str(), f.unit.as_str()), ("180", "deg"));
}

#[test]
fn angle_counts_like_dimensionless_for_explicit_units() {
    let torque = Quantity::new(5.0, Dim { exp: [2, 1, -2, 0, 0, 0, 0, 0] });
    let with_angle = Quantity::new(5.0, Dim { exp: [2, 1, -2, 0, 0, 0, 0, 1] });
    let j = parse_unit("J").unwrap();
    assert_eq!(format(torque, Some(&j), UnitSystem::Si, Precision::default()).unwrap().number, "5");
    assert_eq!(format(with_angle, Some(&j), UnitSystem::Si, Precision::default()).unwrap().number, "5");
    let nm = parse_unit("N·m").unwrap();
    assert_eq!(format(with_angle, Some(&nm), UnitSystem::Si, Precision::default()).unwrap().number, "5");
    let deg = parse_unit("deg").unwrap();
    let f = format(Quantity::new(std::f64::consts::FRAC_PI_2, Dim::ANGLE), Some(&deg), UnitSystem::Si, Precision::default()).unwrap();
    assert_eq!((f.number.as_str(), f.unit.as_str()), ("90", "deg"));
}

#[test]
fn natural_unit_ignores_angle_exponent_on_table_match() {
    let with_angle = Dim { exp: [2, 1, -2, 0, 0, 0, 0, 1] };
    assert_eq!(natural_unit(with_angle, UnitSystem::Si).text, "J");
}

#[test]
fn rounding_is_half_away_from_zero() {
    assert_eq!(num(0.5, 0), "1");
    assert_eq!(num(1.5, 0), "2");
    assert_eq!(num(2.5, 0), "3");
    assert_eq!(num(-0.5, 0), "-1");
    assert_eq!(num(-2.5, 0), "-3");
    assert_eq!(num(0.125, 2), "0.13");
    assert_eq!(num(-0.125, 2), "-0.13");
    assert_eq!(num(1.0005, 3), "1.001");
}

#[test]
fn trailing_zeros_and_point_trimmed() {
    assert_eq!(num(1.0, 3), "1");
    assert_eq!(num(1.50, 3), "1.5");
    assert_eq!(num(100.0, 3), "100");
    assert_eq!(num(10.0, 0), "10");
    assert_eq!(num(0.0, 3), "0");
}

#[test]
fn negative_zero_displays_as_zero() {
    assert_eq!(num(-0.0, 3), "0");
    assert_eq!(num(-0.0004, 3), "0");
    assert_eq!(num(-0.4, 0), "0");
    assert_eq!(num(-0.0006, 3), "-0.001");
}

#[test]
fn decimals_clamped_to_fifteen() {
    assert_eq!(num(0.25, 200), "0.25");
    assert_eq!(Precision { decimals: 200 }.clamped().decimals, 15);
    assert_eq!(Precision::new(200).decimals, 15);
    assert_eq!(Precision::new(4).decimals, 4);
}

#[test]
fn exponential_thresholds() {
    assert_eq!(num(999_999.0, 3), "999999");
    assert_eq!(num(1.0e6, 3), "1×10^6");
    assert_eq!(num(-1.0e6, 3), "-1×10^6");
    assert_eq!(num(0.0001, 3), "0");
    assert_eq!(num(0.00009, 3), "9×10^-5");
    assert_eq!(num(1.5e-5, 3), "1.5×10^-5");
    assert_eq!(num(-1.5e-5, 3), "-1.5×10^-5");
    assert_eq!(num(1.23456789e9, 3), "1.235×10^9");
    assert_eq!(num(1.25e7, 1), "1.3×10^7");
}

#[test]
fn exponential_mantissa_carry() {
    assert_eq!(num(9.9996e6, 3), "1×10^7");
    assert_eq!(num(9.99996e-5, 3), "1×10^-4");
}

#[test]
fn exponential_extremes_do_not_panic() {
    assert_eq!(num(f64::MAX, 3), "1.798×10^308");
    assert_eq!(num(f64::MIN_POSITIVE, 3), "2.225×10^-308");
    assert_eq!(num(5e-324, 3), "5×10^-324");
}

#[test]
fn exponential_applies_in_display_unit() {
    // 1e7 Pa is 10 MPa: no exponential once shown in MPa.
    let q = Quantity::new(1.0e7, stress(1.0).dim);
    let f = format(q, Some(&parse_unit("MPa").unwrap()), UnitSystem::Si, Precision::default()).unwrap();
    assert_eq!((f.number.as_str(), f.unit.as_str()), ("10", "MPa"));
}

#[test]
fn non_finite_is_error() {
    assert_eq!(format(Quantity::new(f64::NAN, Dim::LENGTH), None, UnitSystem::Si, Precision::default()), Err(UnitError::NonFinite));
    assert_eq!(format(Quantity::new(f64::INFINITY, Dim::LENGTH), None, UnitSystem::Si, Precision::default()), Err(UnitError::NonFinite));
}

#[test]
fn negative_absolute_temperature_is_just_formatted() {
    let f = format(Quantity::new(-10.0, Dim::TEMPERATURE), None, UnitSystem::Si, Precision::default()).unwrap();
    assert_eq!((f.number.as_str(), f.unit.as_str()), ("-10", "K"));
}

#[test]
fn celsius_display() {
    let q = Quantity::new(300.0, Dim::TEMPERATURE);
    let f = format(q, Some(&parse_unit("°C").unwrap()), UnitSystem::Si, Precision::default()).unwrap();
    assert_eq!((f.number.as_str(), f.unit.as_str()), ("26.85", "°C"));
}

#[test]
fn natural_unit_for_extreme_dim_does_not_panic() {
    let d = Dim { exp: [127, -128, 127, -128, 127, -128, 127, -128] };
    let s = natural_unit(d, UnitSystem::UsCustomary);
    assert!(!s.text.is_empty());
    let r = format(Quantity::new(1.0, d), None, UnitSystem::UsCustomary, Precision::default());
    assert!(r.is_ok() || r == Err(UnitError::NonFinite));
}
