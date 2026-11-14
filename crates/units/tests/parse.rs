use craft_units::{UnitError, lookup, parse_unit};

#[test]
fn psi_is_lbf_per_square_inch() {
    let a = parse_unit("psi").unwrap();
    let b = parse_unit("lbf/in^2").unwrap();
    assert_eq!(a.dim, b.dim);
    assert!((a.factor / b.factor - 1.0).abs() < 1e-12);
    assert!((a.factor - 6894.757293168361).abs() < 1e-6);
}
#[test]
fn prefixes() {
    assert!((parse_unit("kN").unwrap().factor - 1000.0).abs() < 1e-9);
    assert!((parse_unit("mm").unwrap().factor - 1e-3).abs() < 1e-15);
}
#[test]
fn superscripts_and_middle_dot() {
    assert_eq!(parse_unit("N·m²").unwrap().dim, parse_unit("N*m^2").unwrap().dim);
}
#[test]
fn unknown_unit() {
    assert!(matches!(parse_unit("furlongz"), Err(UnitError::UnknownUnit(_))));
}
#[test]
fn offset_unit_in_compound_is_rejected() {
    assert_eq!(parse_unit("°F/s"), Err(UnitError::AffineTemperature));
}
#[test]
fn degf_round_trip() {
    let f = parse_unit("°F").unwrap();
    assert!((f.to_si(32.0) - 273.15).abs() < 1e-9);
    assert!((f.from_si(373.15) - 212.0).abs() < 1e-9);
}
#[test]
fn delta_degf_has_no_offset() {
    assert!((parse_unit("Δ°F").unwrap().to_si(9.0) - 5.0).abs() < 1e-9);
}
#[test]
fn hostile_unit_text() {
    assert!(parse_unit(&"(".repeat(100)).is_err());
    assert!(parse_unit(&"m*".repeat(500)).is_err());
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * b.abs().max(1.0)
}

#[test]
fn lookup_exact_names_win_over_prefix_splits() {
    for (name, factor) in [("min", 60.0), ("mil", 0.0000254), ("mi", 1609.344), ("Pa", 1.0), ("cd", 1.0)] {
        let s = lookup(name).unwrap();
        assert!(close(s.factor, factor), "{name}");
        assert_eq!(s.text, name);
    }
    assert_eq!(lookup("min").unwrap().dim, craft_units::Dim::TIME);
    assert_eq!(lookup("cd").unwrap().dim, craft_units::Dim::LUMINOSITY);
    assert_eq!(lookup("Pa").unwrap().dim.exp, [-1, 1, -2, 0, 0, 0, 0, 0]);
    assert!(close(lookup("ksi").unwrap().factor, 1000.0 * lookup("psi").unwrap().factor));
    assert!(close(lookup("kip").unwrap().factor, 1000.0 * lookup("lbf").unwrap().factor));
}

#[test]
fn lookup_rejects_non_units() {
    for name in ["", "k", "u", "µ", "kft", "k°C", "kkm", " m", "m ", "Nm", "kmin", "mmi", "KN", "mM"] {
        assert_eq!(lookup(name), None, "{name:?}");
    }
}

#[test]
fn lookup_prefixes() {
    assert!(close(lookup("kg").unwrap().factor, 1.0));
    assert!(close(lookup("g").unwrap().factor, 1e-3));
    assert!(close(lookup("Mg").unwrap().factor, 1e3));
    assert!(close(lookup("cm").unwrap().factor, 1e-2));
    assert!(close(lookup("GPa").unwrap().factor, 1e9));
    assert!(close(lookup("mL").unwrap().factor, 1e-6));
    assert!(close(lookup("mmol").unwrap().factor, 1e-3));
    for micro in ["µm", "μm", "um"] {
        assert!(close(lookup(micro).unwrap().factor, 1e-6), "{micro}");
    }
}

#[test]
fn deg_and_degree_sign_are_the_same_angle_unit() {
    let a = lookup("deg").unwrap();
    let b = lookup("°").unwrap();
    assert_eq!((a.dim, a.factor), (b.dim, b.factor));
    assert!(close(a.factor, std::f64::consts::PI / 180.0));
    assert_eq!(a.dim, craft_units::Dim::ANGLE);
    let r = lookup("rad").unwrap();
    assert_eq!((r.dim, r.factor), (craft_units::Dim::ANGLE, 1.0));
    assert_eq!(lookup("mdeg"), None);
}

#[test]
fn catalog_names_are_unique_and_resolve_to_themselves() {
    let mut seen = std::collections::HashSet::new();
    for def in craft_units::CATALOG {
        for name in def.names {
            assert!(seen.insert(*name), "duplicate {name}");
            let s = lookup(name).unwrap();
            assert_eq!((s.dim, s.factor, s.offset), (def.dim, def.factor, def.offset));
        }
    }
}

#[test]
fn us_customary_factors() {
    assert!(close(parse_unit("in").unwrap().factor, 0.0254));
    assert!(close(parse_unit("ft").unwrap().factor, 0.3048));
    assert!(close(parse_unit("yd").unwrap().factor, 0.9144));
    assert!(close(parse_unit("lbm").unwrap().factor, 0.45359237));
    assert!(close(parse_unit("lbf").unwrap().factor, 4.4482216152605));
    assert!(close(parse_unit("slug").unwrap().factor, 14.593902937206364));
    let hp = parse_unit("hp").unwrap();
    assert!(close(hp.factor, 745.6998715822702));
    assert!(close(hp.factor / parse_unit("ft*lbf/s").unwrap().factor, 550.0));
    assert!(close(parse_unit("psf").unwrap().factor, parse_unit("lbf/ft^2").unwrap().factor));
    assert!(close(parse_unit("gal").unwrap().factor, 0.003785411784));
    assert!(close(parse_unit("BTU").unwrap().factor, 1055.05585262));
    assert!(close(parse_unit("L").unwrap().factor, 1e-3));
    assert!(close(parse_unit("h").unwrap().factor, 3600.0));
    let ftlbf = parse_unit("ft*lbf").unwrap();
    assert_eq!(ftlbf.dim, parse_unit("J").unwrap().dim);
}

#[test]
fn temperature_scales() {
    let c = parse_unit("°C").unwrap();
    assert!(close(c.to_si(0.0), 273.15));
    assert!(close(c.from_si(373.15), 100.0));
    assert!(close(parse_unit("(°C)").unwrap().to_si(0.0), 273.15));
    assert!(close(parse_unit("°R").unwrap().to_si(9.0), 5.0));
    assert!(close(parse_unit("K").unwrap().to_si(5.0), 5.0));
    assert!(close(parse_unit("Δ°C").unwrap().to_si(5.0), 5.0));
    assert!(close(parse_unit("∆°F").unwrap().to_si(9.0), 5.0));
    assert_eq!(parse_unit("°R").unwrap().offset, 0.0);
    let per_c = parse_unit("W/(m*Δ°C)").unwrap();
    assert_eq!(per_c.offset, 0.0);
    for expr in ["°C*m", "m/°C", "°C^2", "°C^1", "1/°F", "(°C)/s", "°F °F"] {
        assert_eq!(parse_unit(expr), Err(UnitError::AffineTemperature), "{expr}");
    }
}

#[test]
fn products_quotients_and_powers() {
    let a = parse_unit("kg m/s^2").unwrap();
    let n = parse_unit("N").unwrap();
    assert_eq!((a.dim, a.factor), (n.dim, n.factor));
    assert_eq!(parse_unit("N m").unwrap().dim, parse_unit("J").unwrap().dim);
    assert_eq!(parse_unit("N⋅m").unwrap().dim, parse_unit("J").unwrap().dim);
    assert_eq!(parse_unit("lbf / in ^ 2").unwrap().dim, parse_unit("psi").unwrap().dim);
    assert_eq!(parse_unit("m³").unwrap().dim.exp[0], 3);
    assert_eq!(parse_unit("m^+2").unwrap().dim.exp[0], 2);
    assert_eq!(parse_unit("m^0").unwrap().dim, craft_units::Dim::ONE);
    assert_eq!(parse_unit("(m/s)^2").unwrap().dim.exp, [2, 0, -2, 0, 0, 0, 0, 0]);
    assert_eq!(parse_unit("(m/s)²").unwrap().dim.exp, [2, 0, -2, 0, 0, 0, 0, 0]);
}

#[test]
fn negative_exponents_and_extended_superscripts() {
    let inv = parse_unit("1/s").unwrap();
    assert_eq!(inv.dim, parse_unit("Hz").unwrap().dim);
    assert_eq!(parse_unit("s^-1").unwrap().dim, inv.dim);
    assert_eq!(parse_unit("s⁻¹").unwrap().dim, inv.dim);
    assert_eq!(parse_unit("m⁻²").unwrap().dim.exp[0], -2);
    assert_eq!(parse_unit("m¹²").unwrap().dim.exp[0], 12);
    // A run of superscript digits is one multi-digit exponent, as in `m¹²`.
    assert_eq!(parse_unit("m²³").unwrap().dim.exp[0], 23);
    assert_eq!(parse_unit("1").unwrap().dim, craft_units::Dim::ONE);
}

#[test]
fn division_is_left_to_right() {
    let a = parse_unit("J/mol K").unwrap();
    let b = parse_unit("J*K/mol").unwrap();
    assert_eq!(a.dim, b.dim);
    let c = parse_unit("m/s/s").unwrap();
    assert_eq!(c.dim.exp, [1, 0, -2, 0, 0, 0, 0, 0]);
    assert_eq!(parse_unit("m/s*s").unwrap().dim, craft_units::Dim::LENGTH);
}

#[test]
fn text_is_the_trimmed_input() {
    assert_eq!(parse_unit("  kN * m ").unwrap().text, "kN * m");
    assert_eq!(parse_unit("psi").unwrap().text, "psi");
}

#[test]
fn syntax_errors() {
    for expr in [
        "",
        "   ",
        "m*",
        "*m",
        "m/",
        "/m",
        "(m",
        "m)",
        "()",
        "m^",
        "m^x",
        "m^-",
        "m^2^3",
        "m²^2",
        "m^2²",
        "2",
        "2m",
        "m 2",
        "m+s",
        "m-s",
        "m^(2)",
        "m⁻",
        "m^99999999999",
        "m%",
        "m,s",
    ] {
        assert!(matches!(parse_unit(expr), Err(UnitError::Syntax(_))), "{expr:?} gave {:?}", parse_unit(expr));
    }
}

#[test]
fn unknown_unit_names_the_unit() {
    assert_eq!(parse_unit("m*foo"), Err(UnitError::UnknownUnit("foo".to_string())));
    assert_eq!(parse_unit("kft"), Err(UnitError::UnknownUnit("kft".to_string())));
}

#[test]
fn input_length_boundary() {
    let ok = format!("{}m", " ".repeat(255));
    assert_eq!(ok.len(), 256);
    assert!(parse_unit(&ok).is_ok());
    let too_long = format!("{}m", " ".repeat(256));
    assert_eq!(too_long.len(), 257);
    assert!(matches!(parse_unit(&too_long), Err(UnitError::Syntax(_))));
    // 128 two-byte characters is 256 bytes; one more is over.
    assert!(matches!(parse_unit(&"µ".repeat(129)), Err(UnitError::Syntax(_))));
}

#[test]
fn nesting_boundary() {
    let nest = |n: usize| format!("{}m{}", "(".repeat(n), ")".repeat(n));
    assert!(parse_unit(&nest(16)).is_ok());
    assert!(matches!(parse_unit(&nest(17)), Err(UnitError::Syntax(_))));
}

#[test]
fn overflow_and_non_finite_are_errors() {
    assert_eq!(parse_unit("m^127").map(|s| s.dim.exp[0]), Ok(127));
    assert_eq!(parse_unit("m^128"), Err(UnitError::Overflow));
    assert_eq!(parse_unit("m^-129"), Err(UnitError::Overflow));
    assert_eq!(parse_unit("(m/in)^99999"), Err(UnitError::NonFinite));
    assert_eq!(parse_unit("(in/m)^99999"), Err(UnitError::NonFinite));
    assert_eq!(parse_unit("m^2147483647"), Err(UnitError::Overflow));
}

mod never_panics {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn arbitrary_text(s in "\\PC{0,300}") {
            let _ = parse_unit(&s);
            let _ = lookup(&s);
        }

        #[test]
        fn unit_flavoured_text(s in "[mkgsNPaµμu°ΔC()*/^·0-9²³⁻ -]{0,300}") {
            let _ = parse_unit(&s);
            let _ = lookup(&s);
        }
    }
}
