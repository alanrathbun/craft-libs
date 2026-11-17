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
    assert!(close(parse_unit("hr").unwrap().factor, 3600.0));
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
    assert_eq!(parse_unit("m⁻³").unwrap().dim.exp[0], -3);
    assert_eq!(parse_unit("m⁴").unwrap().dim.exp[0], 4);
    assert_eq!(parse_unit("1").unwrap().dim, craft_units::Dim::ONE);
}

#[test]
fn multiplication_after_division_is_ambiguous() {
    for expr in ["J/mol K", "W/m·K", "W/m*K", "W/m⋅K", "kg/m s", "m/s (m)", "m/s/s K", "(m/s)/s*K"] {
        match parse_unit(expr) {
            Err(UnitError::Syntax(msg)) => assert!(msg.contains("ambiguous"), "{expr}: {msg}"),
            other => panic!("{expr:?} gave {other:?}"),
        }
    }
}

#[test]
fn division_forms_that_stay_legal() {
    let molar = parse_unit("J*K/mol").unwrap().dim;
    let mut want = parse_unit("J").unwrap().dim.div(parse_unit("mol").unwrap().dim).unwrap();
    want = want.div(craft_units::Dim::TEMPERATURE).unwrap();
    assert_eq!(parse_unit("J/(mol K)").unwrap().dim, want);
    assert_eq!(parse_unit("J/(mol·K)").unwrap().dim, want);
    assert_ne!(molar, want);
    assert_eq!(parse_unit("m/s/s").unwrap().dim.exp, [1, 0, -2, 0, 0, 0, 0, 0]);
    assert_eq!(parse_unit("N/m^2").unwrap().dim, parse_unit("Pa").unwrap().dim);
    assert_eq!(parse_unit("kg/m³").unwrap().dim.exp, [-3, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(parse_unit("(m/s)*s").unwrap().dim, craft_units::Dim::LENGTH);
    assert_eq!(parse_unit("(m/s) s").unwrap().dim, craft_units::Dim::LENGTH);
    assert_eq!(parse_unit("N*m/s").unwrap().dim, parse_unit("W").unwrap().dim);
    assert_eq!(parse_unit("N m/s").unwrap().dim, parse_unit("W").unwrap().dim);
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
        "m¹²",
        "m²³",
        "m⁻¹²",
        "m^2²",
        "2",
        "2m",
        "m 2",
        "m+s",
        "m-s",
        "m^(2)",
        "m⁻",
        "m^99999999999",
        "m$",
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

#[test]
fn gm_is_mathcads_gram() {
    let gm = parse_unit("gm").unwrap();
    let g = parse_unit("g").unwrap();
    assert_eq!((gm.dim, gm.factor), (g.dim, g.factor));
    assert_eq!(parse_unit("gm/cm^3").unwrap().dim, parse_unit("g/cm^3").unwrap().dim);
    assert!(parse_unit("kgm").is_err(), "gm takes no prefix: kg is the kilogram");
}

const LBF: f64 = 4.4482216152605;
const FT: f64 = 0.3048;

fn dim_of(name: &str) -> [i8; 8] {
    lookup(name).unwrap_or_else(|| panic!("{name}")).dim.exp
}

#[test]
fn mathcad_units() {
    let f = |n: &str| lookup(n).unwrap_or_else(|| panic!("{n}")).factor;
    assert_eq!(f("hr"), 3600.0);
    assert!(lookup("h").is_none());
    assert!(lookup("lb").is_none());
    assert!(lookup("R").is_none());
    assert_eq!(f("sec"), 1.0);
    assert_eq!(f("Btu"), f("BTU"));
    assert_eq!(f("liter"), 1e-3);
    assert_eq!(f("watt"), 1.0);
    assert_eq!(f("joule"), 1.0);
    assert_eq!(f("mole"), 1.0);
    assert_eq!(f("Å"), 1e-10);
    assert_eq!(f("Angstrom"), 1e-10);
    let near = [
        ("atm", 101325.0),
        ("bar", 1e5),
        ("mbar", 100.0),
        ("torr", 101325.0 / 760.0),
        ("gpm", 0.003785411784 / 60.0),
        ("cfm", FT.powi(3) / 60.0),
        ("mph", 1609.344 / 3600.0),
        ("poise", 0.1),
        ("cP", 1e-3),
        ("%", 0.01),
        ("kgf", 9.80665),
        ("ton", 2000.0 * 0.45359237),
        ("tonne", 1000.0),
        ("cal", 4.1868),
        ("kcal", 4186.8),
        ("day", 86400.0),
        ("yr", 365.2422 * 86400.0),
        ("plf", LBF / FT),
        ("pcf", LBF / FT.powi(3)),
        ("ksf", 1000.0 * LBF / FT.powi(2)),
    ];
    for (name, want) in near {
        assert!(close(f(name), want), "{name}");
    }
}

#[test]
fn mathcad_unit_dimensions() {
    let pressure = [-1, 1, -2, 0, 0, 0, 0, 0];
    let force = [1, 1, -2, 0, 0, 0, 0, 0];
    assert_eq!(dim_of("poise"), [-1, 1, -1, 0, 0, 0, 0, 0]);
    assert_eq!(dim_of("cP"), [-1, 1, -1, 0, 0, 0, 0, 0]);
    assert!(lookup("%").unwrap().dim.is_dimensionless());
    assert_eq!(dim_of("kgf"), force);
    assert_eq!(dim_of("ton"), craft_units::Dim::MASS.exp);
    assert_eq!(dim_of("tonne"), craft_units::Dim::MASS.exp);
    assert_eq!(dim_of("plf"), [0, 1, -2, 0, 0, 0, 0, 0]);
    assert_eq!(dim_of("pcf"), [-2, 1, -2, 0, 0, 0, 0, 0]);
    assert_eq!(dim_of("ksf"), pressure);
    for p in ["atm", "bar", "mbar", "torr"] {
        assert_eq!(dim_of(p), pressure, "{p}");
    }
    for e in ["cal", "kcal", "Btu", "joule"] {
        assert_eq!(dim_of(e), [2, 1, -2, 0, 0, 0, 0, 0], "{e}");
    }
    assert_eq!(dim_of("watt"), [2, 1, -3, 0, 0, 0, 0, 0]);
    assert_eq!(dim_of("liter"), [3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(dim_of("gpm"), [3, 0, -1, 0, 0, 0, 0, 0]);
    assert_eq!(dim_of("cfm"), [3, 0, -1, 0, 0, 0, 0, 0]);
    assert_eq!(dim_of("mph"), [1, 0, -1, 0, 0, 0, 0, 0]);
    assert_eq!(dim_of("Å"), craft_units::Dim::LENGTH.exp);
    assert_eq!(dim_of("mole"), craft_units::Dim::AMOUNT.exp);
    for t in ["sec", "hr", "day", "yr"] {
        assert_eq!(dim_of(t), craft_units::Dim::TIME.exp, "{t}");
    }
}

#[test]
fn new_names_keep_old_prefixed_spellings() {
    for (n, v) in [("ms", 1e-3), ("min", 60.0), ("mi", 1609.344), ("mm", 1e-3), ("kPa", 1e3), ("cd", 1.0)] {
        assert!(close(lookup(n).unwrap().factor, v), "{n}");
    }
}

#[test]
fn long_spellings_take_no_prefix() {
    for n in ["msec", "kliter", "kwatt", "kjoule", "kBtu", "kAngstrom", "mhr", "katm", "kpsi"] {
        assert_eq!(lookup(n), None, "{n}");
    }
    // The symbols stay prefixable, and cal takes prefixes.
    for (n, v) in [("kJ", 1e3), ("kW", 1e3), ("mL", 1e-6), ("mcal", 4.1868e-3)] {
        assert!(close(lookup(n).unwrap().factor, v), "{n}");
    }
}

#[test]
fn bar_takes_no_prefix_but_mbar_and_kbar_are_units() {
    // Prefixing bar would reserve plausible names (ubar, cbar, nbar, Mbar, µbar): only the
    // millibar and kilobar are units, as names of their own.
    for (n, v) in [("bar", 1e5), ("mbar", 100.0), ("kbar", 1e8)] {
        assert!(close(lookup(n).unwrap().factor, v), "{n}");
        assert_eq!(dim_of(n), [-1, 1, -2, 0, 0, 0, 0, 0], "{n}");
    }
    for n in ["ubar", "µbar", "μbar", "cbar", "nbar", "Mbar", "Gbar"] {
        assert_eq!(lookup(n), None, "{n}");
    }
}

#[test]
fn percent_is_a_unit_token_on_its_own() {
    let p = parse_unit("%").unwrap();
    assert!(close(p.factor, 0.01));
    assert!(p.dim.is_dimensionless());
    assert_eq!(p.offset, 0.0);
    assert_eq!(p.text, "%");
    // It never joins the letters around it: these are products of separate units.
    let a = parse_unit("%m").unwrap();
    assert!(close(a.factor, 0.01));
    assert_eq!(a.dim, craft_units::Dim::LENGTH);
    let b = parse_unit("m%").unwrap();
    assert!(close(b.factor, 0.01));
    assert_eq!(b.dim, craft_units::Dim::LENGTH);
    assert_eq!(lookup("m%"), None);
    assert_eq!(lookup("%m"), None);
}

#[test]
fn mathcad_compound_units_parse() {
    let btu_hr = parse_unit("Btu/hr").unwrap();
    assert_eq!(btu_hr.dim.exp, [2, 1, -3, 0, 0, 0, 0, 0]);
    assert!(close(btu_hr.factor, 1055.05585262 / 3600.0));
    let gpm = parse_unit("gal/min").unwrap();
    assert!(close(gpm.factor, lookup("gpm").unwrap().factor));
    assert_eq!(gpm.dim, lookup("gpm").unwrap().dim);
    let pcf = parse_unit("lbf/ft^3").unwrap();
    assert!(close(pcf.factor, lookup("pcf").unwrap().factor));
    assert_eq!(pcf.dim, lookup("pcf").unwrap().dim);
}
