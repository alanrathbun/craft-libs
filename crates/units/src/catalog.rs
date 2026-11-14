//! The unit catalog and SI-prefix lookup.

use crate::Dim;

/// One entry of the unit catalog. The SI value of a magnitude `x` is `(x + offset) * factor`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitDef {
    /// Every spelling that names this unit.
    pub names: &'static [&'static str],
    /// Dimension of the unit.
    pub dim: Dim,
    /// SI value of one unit (after applying `offset`).
    pub factor: f64,
    /// Added to a magnitude before scaling; non-zero only for the offset temperature scales.
    pub offset: f64,
    /// Whether an SI prefix such as `k` or `m` may be put in front of the name.
    pub prefixable: bool,
}

/// A unit resolved to its SI scale. The SI value of a magnitude `x` is `(x + offset) * factor`.
#[derive(Clone, Debug, PartialEq)]
pub struct Scale {
    /// Dimension of the unit.
    pub dim: Dim,
    /// SI value of one unit (after applying `offset`).
    pub factor: f64,
    /// Added to a magnitude before scaling; non-zero only for a lone `°C` or `°F`.
    pub offset: f64,
    /// The unit text this scale came from: the catalog name for [`lookup`], the input with
    /// surrounding whitespace trimmed for [`parse_unit`](crate::parse_unit).
    pub text: String,
}

impl Scale {
    /// Convert a magnitude in this unit to its SI value. Non-finite input stays non-finite.
    pub fn to_si(&self, x: f64) -> f64 {
        (x + self.offset) * self.factor
    }

    /// Convert an SI value to a magnitude in this unit. Non-finite input stays non-finite.
    pub fn from_si(&self, si: f64) -> f64 {
        si / self.factor - self.offset
    }
}

const IN: f64 = 0.0254;
const FT: f64 = 0.3048;
const YD: f64 = 0.9144;
const MI: f64 = 1609.344;
const LBM: f64 = 0.45359237;
const LBF: f64 = 4.4482216152605;
const PSI: f64 = LBF / (IN * IN);
/// International Table British thermal unit, in joules.
const BTU: f64 = 1055.05585262;
/// Mechanical horsepower: 550 ft·lbf/s.
const HP: f64 = 550.0 * FT * LBF;
/// US liquid gallon, in cubic metres.
const GAL: f64 = 0.003785411784;
const DEG: f64 = std::f64::consts::PI / 180.0;
const RANKINE: f64 = 5.0 / 9.0;

// Dimension exponent order: [L, M, T, I, Θ, N, J, angle].
const LENGTH: Dim = Dim::LENGTH;
const MASS: Dim = Dim::MASS;
const TIME: Dim = Dim::TIME;
const FORCE: Dim = Dim { exp: [1, 1, -2, 0, 0, 0, 0, 0] };
const PRESSURE: Dim = Dim { exp: [-1, 1, -2, 0, 0, 0, 0, 0] };
const ENERGY: Dim = Dim { exp: [2, 1, -2, 0, 0, 0, 0, 0] };
const POWER: Dim = Dim { exp: [2, 1, -3, 0, 0, 0, 0, 0] };
const FREQUENCY: Dim = Dim { exp: [0, 0, -1, 0, 0, 0, 0, 0] };
const VOLUME: Dim = Dim { exp: [3, 0, 0, 0, 0, 0, 0, 0] };

const fn unit(names: &'static [&'static str], dim: Dim, factor: f64, prefixable: bool) -> UnitDef {
    UnitDef { names, dim, factor, offset: 0.0, prefixable }
}

const fn offset_unit(names: &'static [&'static str], dim: Dim, factor: f64, offset: f64) -> UnitDef {
    UnitDef { names, dim, factor, offset, prefixable: false }
}

/// Every unit the parser knows, before SI prefixes.
pub static CATALOG: &[UnitDef] = &[
    // SI base and derived
    unit(&["m"], LENGTH, 1.0, true),
    unit(&["g"], MASS, 1e-3, true),
    unit(&["s"], TIME, 1.0, true),
    unit(&["A"], Dim::CURRENT, 1.0, true),
    unit(&["K"], Dim::TEMPERATURE, 1.0, true),
    unit(&["mol"], Dim::AMOUNT, 1.0, true),
    unit(&["cd"], Dim::LUMINOSITY, 1.0, true),
    unit(&["rad"], Dim::ANGLE, 1.0, true),
    unit(&["deg", "°"], Dim::ANGLE, DEG, false),
    unit(&["N"], FORCE, 1.0, true),
    unit(&["Pa"], PRESSURE, 1.0, true),
    unit(&["J"], ENERGY, 1.0, true),
    unit(&["W"], POWER, 1.0, true),
    unit(&["Hz"], FREQUENCY, 1.0, true),
    unit(&["L"], VOLUME, 1e-3, true),
    unit(&["min"], TIME, 60.0, false),
    unit(&["h"], TIME, 3600.0, false),
    // US customary
    unit(&["in"], LENGTH, IN, false),
    unit(&["ft"], LENGTH, FT, false),
    unit(&["yd"], LENGTH, YD, false),
    unit(&["mi"], LENGTH, MI, false),
    unit(&["mil"], LENGTH, IN / 1000.0, false),
    unit(&["lbm"], MASS, LBM, false),
    unit(&["slug"], MASS, LBF / FT, false),
    unit(&["lbf"], FORCE, LBF, false),
    unit(&["kip"], FORCE, 1000.0 * LBF, false),
    unit(&["psi"], PRESSURE, PSI, false),
    unit(&["ksi"], PRESSURE, 1000.0 * PSI, false),
    unit(&["psf"], PRESSURE, LBF / (FT * FT), false),
    unit(&["BTU"], ENERGY, BTU, false),
    unit(&["hp"], POWER, HP, false),
    unit(&["gal"], VOLUME, GAL, false),
    // Temperature: absolute scales, offset scales (lone use only), and differences.
    unit(&["°R"], Dim::TEMPERATURE, RANKINE, false),
    offset_unit(&["°C"], Dim::TEMPERATURE, 1.0, 273.15),
    offset_unit(&["°F"], Dim::TEMPERATURE, RANKINE, 459.67),
    unit(&["Δ°C", "∆°C"], Dim::TEMPERATURE, 1.0, false),
    unit(&["Δ°F", "∆°F"], Dim::TEMPERATURE, RANKINE, false),
];

/// SI prefixes the catalog accepts, with their scale.
const PREFIXES: &[(&str, f64)] = &[("k", 1e3), ("M", 1e6), ("G", 1e9), ("m", 1e-3), ("µ", 1e-6), ("μ", 1e-6), ("u", 1e-6), ("n", 1e-9), ("c", 1e-2)];

fn exact(name: &str) -> Option<&'static UnitDef> {
    CATALOG.iter().find(|def| def.names.contains(&name))
}

/// Resolve a single unit name, with or without an SI prefix.
///
/// An exact catalog name always wins over a prefix split, so `min` is minutes, `mi` miles and
/// `Pa` pascal. A prefix applies only to units marked `prefixable`, and only one prefix is
/// allowed. The name is matched as given: no trimming, no case folding. `µ` (U+00B5), `μ`
/// (U+03BC) and `u` all mean micro.
pub fn lookup(name: &str) -> Option<Scale> {
    if let Some(def) = exact(name) {
        return Some(scale_of(def, 1.0, name));
    }
    PREFIXES.iter().find_map(|(prefix, mult)| {
        let rest = name.strip_prefix(prefix)?;
        let def = exact(rest).filter(|def| def.prefixable)?;
        Some(scale_of(def, *mult, name))
    })
}

fn scale_of(def: &UnitDef, mult: f64, text: &str) -> Scale {
    Scale { dim: def.dim, factor: def.factor * mult, offset: def.offset, text: text.to_string() }
}
