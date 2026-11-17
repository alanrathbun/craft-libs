//! Display formatting: natural units, rounding and exponential notation.

use crate::catalog::Scale;
use crate::parse::parse_unit;
use crate::{Dim, Quantity, UnitError};
use serde::{Deserialize, Serialize};

/// Largest number of decimal places [`Precision`] allows.
const MAX_DECIMALS: u8 = 15;
/// Magnitudes at or above this show in exponential form.
const EXP_HIGH: f64 = 1e6;
/// Non-zero magnitudes below this always show in exponential form.
const EXP_LOW: f64 = 1e-4;
/// Index of the angle exponent in [`Dim::exp`].
const ANGLE: usize = 7;

/// Which family of units a quantity is shown in when the author names none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitSystem {
    /// Inches, pounds, seconds and degrees Rankine.
    #[default]
    UsCustomary,
    /// Metres, kilograms, seconds and kelvin.
    Si,
}

/// How many decimal places a displayed number keeps.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Precision {
    /// Decimal places, `0..=15`. A larger value (from untrusted data) is treated as 15.
    pub decimals: u8,
}

impl Precision {
    /// A precision of `decimals` places, clamped to `0..=15`.
    pub fn new(decimals: u8) -> Precision {
        Precision { decimals: decimals.min(MAX_DECIMALS) }
    }

    /// This precision with `decimals` clamped to `0..=15`.
    pub fn clamped(self) -> Precision {
        Precision::new(self.decimals)
    }
}

impl Default for Precision {
    fn default() -> Precision {
        Precision { decimals: 3 }
    }
}

/// A quantity ready to show: the number text and the unit text, kept apart so a document can
/// style them separately.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formatted {
    /// The magnitude, with `.` as the decimal separator, trailing zeros trimmed, and the form
    /// `6×10^6` for very large or small values.
    pub number: String,
    /// The unit text; empty for a plain number. It parses with [`parse_unit`].
    pub unit: String,
}

/// Names of the base units of each system, in [`Dim::exp`] order.
fn base_names(sys: UnitSystem) -> [&'static str; 8] {
    match sys {
        UnitSystem::UsCustomary => ["in", "lbm", "sec", "A", "°R", "mol", "cd", "rad"],
        UnitSystem::Si => ["m", "kg", "sec", "A", "K", "mol", "cd", "rad"],
    }
}

/// A scale for `text`, parsed from the catalog. The text is fixed by this module and always
/// parses; should it ever not, the scale falls back to an SI one-to-one so this stays total.
fn scale_for(text: &str, dim: Dim) -> Scale {
    parse_unit(text).unwrap_or_else(|_| Scale { dim, factor: 1.0, offset: 0.0, text: text.to_string() })
}

/// The unit a quantity of dimension `dim` shows in when the author names none.
///
/// The angle exponent is ignored when matching the table (so `N·m` shows as `joule`), except that
/// a pure angle is `rad` (SI) or `deg` (US customary). A dimension the table lacks is written
/// in base units, such as `lbm/(in·sec)` or `m/sec²`, with `rad` included when the angle exponent
/// is not zero. The text parses with [`parse_unit`].
pub fn natural_unit(dim: Dim, sys: UnitSystem) -> Scale {
    let us = sys == UnitSystem::UsCustomary;
    let mut base = dim;
    base.exp[ANGLE] = 0;
    if base == Dim::ONE {
        return match dim.exp[ANGLE] {
            0 => Scale { dim: Dim::ONE, factor: 1.0, offset: 0.0, text: String::new() },
            1 => scale_for(if us { "deg" } else { "rad" }, Dim::ANGLE),
            _ => composed(dim, sys),
        };
    }
    let table = match base.exp {
        [1, 0, 0, 0, 0, 0, 0, 0] => Some(("in", "m")),
        [2, 0, 0, 0, 0, 0, 0, 0] => Some(("in²", "m²")),
        [3, 0, 0, 0, 0, 0, 0, 0] => Some(("in³", "m³")),
        [0, 1, 0, 0, 0, 0, 0, 0] => Some(("lbm", "kg")),
        [1, 1, -2, 0, 0, 0, 0, 0] => Some(("lbf", "N")),
        [-1, 1, -2, 0, 0, 0, 0, 0] => Some(("psi", "Pa")),
        [2, 1, -2, 0, 0, 0, 0, 0] => Some(("lbf·in", "joule")),
        [2, 1, -3, 0, 0, 0, 0, 0] => Some(("hp", "watt")),
        [0, 0, 1, 0, 0, 0, 0, 0] => Some(("sec", "sec")),
        [0, 0, 0, 0, 1, 0, 0, 0] => Some(("°R", "K")),
        _ => None,
    };
    match table {
        Some((us_text, si_text)) => scale_for(if us { us_text } else { si_text }, base),
        None => composed(dim, sys),
    }
}

/// The base-unit composition of `dim`, with the product of the base units' factors.
fn composed(dim: Dim, sys: UnitSystem) -> Scale {
    let names = base_names(sys);
    let mut factor = 1.0_f64;
    for (name, e) in names.iter().zip(dim.exp.iter()) {
        if *e != 0 {
            let unit_factor = parse_unit(name).map_or(1.0, |s| s.factor);
            factor *= unit_factor.powi(i32::from(*e));
        }
    }
    Scale { dim, factor, offset: 0.0, text: dim.compose(&names) }
}

/// Show `q` as number and unit text.
///
/// With `unit` the magnitude is converted to it; its dimension must match `q`'s apart from the
/// angle exponent, else [`UnitError::Mismatch`]. Without one the [`natural_unit`] for `sys` is
/// used. `p` sets the decimal places (clamped to 15): rounding is half away from zero, trailing
/// zeros and a trailing `.` are dropped, and `-0` shows as `0`. A magnitude of at least `1e6`
/// shows as `6×10^6`; a non-zero one too small to keep two significant digits in fixed form
/// (below `10^(1−decimals)`, at most 1 and at least `1e-4`) shows as `1.5×10^-5`, its mantissa
/// rounded the same way. A non-finite `q`, or a conversion that overflows, is
/// [`UnitError::NonFinite`].
pub fn format(q: Quantity, unit: Option<&Scale>, sys: UnitSystem, p: Precision) -> Result<Formatted, UnitError> {
    let natural;
    let scale = match unit {
        Some(scale) => {
            if !q.dim.same_ignoring_angle(scale.dim) {
                return Err(UnitError::Mismatch { left: q.dim, right: scale.dim });
            }
            scale
        }
        None => {
            natural = natural_unit(q.dim, sys);
            &natural
        }
    };
    let shown = scale.from_si(q.value);
    if !q.value.is_finite() || !shown.is_finite() {
        return Err(UnitError::NonFinite);
    }
    Ok(Formatted { number: number_text(shown, p.clamped().decimals), unit: scale.text.clone() })
}

/// `x` (finite) as text with at most `decimals` places, trimmed, in exponential form when large
/// or small.
///
/// Large is decided on the value as it would print in fixed form (one that rounds up to `1e6`).
/// Small is a non-zero value below [`small_below`], which fixed form would show with fewer than
/// two significant digits (`0.0005` at 3 places is `0.001`, so it shows as `5×10^-4`); and a
/// non-zero value that rounds to zero is always small, so a non-zero value never shows as `0`.
fn number_text(x: f64, decimals: u8) -> String {
    let magnitude = x.abs();
    let rounded = (magnitude * 10f64.powi(i32::from(decimals))).round() / 10f64.powi(i32::from(decimals));
    let wants_exponential = rounded >= EXP_HIGH || (magnitude > 0.0 && (magnitude < small_below(decimals) || rounded == 0.0));
    wants_exponential.then(|| exponential(x, decimals)).flatten().unwrap_or_else(|| fixed(x, decimals))
}

/// Magnitudes below this show in exponential form at `decimals` places: `10^(1−decimals)`, the
/// smallest value fixed form shows with two significant digits, kept between [`EXP_LOW`] and 1
/// (at 0 or 1 places a value of 1 or more shows as it is).
fn small_below(decimals: u8) -> f64 {
    10f64.powi(1 - i32::from(decimals)).clamp(EXP_LOW, 1.0)
}

/// Fixed-point text, rounded half away from zero and trimmed. `-0` gives `0`.
fn fixed(x: f64, decimals: u8) -> String {
    let scaled = (x.abs() * 10f64.powi(i32::from(decimals))).round();
    // The cast saturates and maps NaN to 0, so it cannot panic; callers pass finite values below
    // 1e21, well inside `u128`.
    let mut digits = (scaled as u128).to_string();
    let places = usize::from(decimals);
    if digits.len() <= places {
        digits.insert_str(0, &"0".repeat(places + 1 - digits.len()));
    }
    let (int, frac) = digits.split_at(digits.len() - places);
    let frac = frac.trim_end_matches('0');
    let mut text = String::new();
    if x < 0.0 && (int != "0" || !frac.is_empty()) {
        text.push('-');
    }
    text.push_str(int);
    if !frac.is_empty() {
        text.push('.');
        text.push_str(frac);
    }
    text
}

/// `m×10^e` text. The mantissa and exponent come from the shortest round-trip decimal form of
/// `x`, so a value typed as `1.25e7` rounds as its digits say. `None` if that form is unreadable.
fn exponential(x: f64, decimals: u8) -> Option<String> {
    let repr = format!("{:e}", x.abs());
    let (mantissa, exp) = repr.split_once('e')?;
    let mut exp: i32 = exp.parse().ok()?;
    let mut mantissa = fixed(mantissa.parse().ok()?, decimals);
    if mantissa == "10" {
        mantissa = "1".to_string();
        exp = exp.checked_add(1)?;
    }
    let sign = if x < 0.0 { "-" } else { "" };
    Some(format!("{sign}{mantissa}×10^{exp}"))
}
