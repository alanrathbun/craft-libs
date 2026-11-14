//! Parser for unit expressions such as `lbf/in^2`, `N·m²` or `kg m/s^2`.

use crate::catalog::{Scale, lookup};
use crate::{Dim, UnitError};

/// Longest accepted expression, in bytes.
const MAX_INPUT_BYTES: usize = 256;
/// Deepest accepted parenthesis nesting.
const MAX_DEPTH: u32 = 16;

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Ident(String),
    Int(i32),
    /// A run of superscript characters such as `²` or `⁻¹`, already decoded.
    Sup(i32),
    Star,
    Slash,
    Caret,
    Plus,
    Minus,
    LParen,
    RParen,
}

fn syntax<T>(msg: &str) -> Result<T, UnitError> {
    Err(UnitError::Syntax(msg.to_string()))
}

fn is_ident_char(c: char) -> bool {
    c.is_alphabetic() || c == '°' || c == '∆'
}

fn superscript_digit(c: char) -> Option<i32> {
    match c {
        '⁰' => Some(0),
        '¹' => Some(1),
        '²' => Some(2),
        '³' => Some(3),
        '⁴' => Some(4),
        '⁵' => Some(5),
        '⁶' => Some(6),
        '⁷' => Some(7),
        '⁸' => Some(8),
        '⁹' => Some(9),
        _ => None,
    }
}

fn push_digit(acc: i32, digit: i32) -> Result<i32, UnitError> {
    acc.checked_mul(10).and_then(|v| v.checked_add(digit)).map_or_else(|| syntax("number too large"), Ok)
}

fn tokenize(input: &str) -> Result<Vec<Tok>, UnitError> {
    let mut toks = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {}
            '*' | '·' | '⋅' => toks.push(Tok::Star),
            '/' => toks.push(Tok::Slash),
            '^' => toks.push(Tok::Caret),
            '+' => toks.push(Tok::Plus),
            '-' => toks.push(Tok::Minus),
            '(' => toks.push(Tok::LParen),
            ')' => toks.push(Tok::RParen),
            c if c.is_ascii_digit() => {
                let mut n = push_digit(0, i32::from(c as u8 - b'0'))?;
                while let Some(d) = chars.next_if(char::is_ascii_digit) {
                    n = push_digit(n, i32::from(d as u8 - b'0'))?;
                }
                toks.push(Tok::Int(n));
            }
            '⁻' | '⁺' | '⁰' | '¹' | '²' | '³' | '⁴' | '⁵' | '⁶' | '⁷' | '⁸' | '⁹' => {
                let negative = c == '⁻';
                let mut digit = superscript_digit(c);
                let mut n = 0i32;
                let mut digits = 0;
                if digit.is_none() {
                    digit = chars.next_if(|&d| superscript_digit(d).is_some()).and_then(superscript_digit);
                }
                while let Some(d) = digit {
                    digits += 1;
                    n = push_digit(n, d)?;
                    digit = chars.next_if(|&d| superscript_digit(d).is_some()).and_then(superscript_digit);
                }
                if digits == 0 {
                    return syntax("superscript sign without digits");
                }
                if digits > 1 {
                    return syntax("ambiguous superscript: write a multi-digit power as ^n");
                }
                toks.push(Tok::Sup(if negative { -n } else { n }));
            }
            c if is_ident_char(c) => {
                let mut name = String::from(c);
                while let Some(n) = chars.next_if(|&n| is_ident_char(n)) {
                    name.push(n);
                }
                toks.push(Tok::Ident(name));
            }
            other => return syntax(&format!("unexpected character `{other}`")),
        }
    }
    Ok(toks)
}

/// A partially evaluated expression. `offset` is non-zero only for a lone offset unit.
#[derive(Clone, Debug)]
struct Term {
    dim: Dim,
    factor: f64,
    offset: f64,
}

impl Term {
    fn one() -> Term {
        Term { dim: Dim::ONE, factor: 1.0, offset: 0.0 }
    }

    fn checked(self) -> Result<Term, UnitError> {
        if self.factor.is_finite() && self.factor > 0.0 { Ok(self) } else { Err(UnitError::NonFinite) }
    }

    fn mul(self, other: Term) -> Result<Term, UnitError> {
        if self.offset != 0.0 || other.offset != 0.0 {
            return Err(UnitError::AffineTemperature);
        }
        let dim = self.dim.mul(other.dim).ok_or(UnitError::Overflow)?;
        Term { dim, factor: self.factor * other.factor, offset: 0.0 }.checked()
    }

    fn div(self, other: Term) -> Result<Term, UnitError> {
        if self.offset != 0.0 || other.offset != 0.0 {
            return Err(UnitError::AffineTemperature);
        }
        let dim = self.dim.div(other.dim).ok_or(UnitError::Overflow)?;
        Term { dim, factor: self.factor / other.factor, offset: 0.0 }.checked()
    }

    fn powi(self, n: i32) -> Result<Term, UnitError> {
        if self.offset != 0.0 {
            return Err(UnitError::AffineTemperature);
        }
        let dim = self.dim.powi(n).ok_or(UnitError::Overflow)?;
        Term { dim, factor: self.factor.powi(n), offset: 0.0 }.checked()
    }
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn bump(&mut self) -> Option<Tok> {
        let tok = self.toks.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    /// `product := factor (('*' | <juxtaposition>) factor)* ('/' factor)*`, left to right.
    ///
    /// Once a `/` has appeared at this level, only further `/` may follow: a multiplication
    /// after a division is ambiguous (`J/mol K`) and is rejected.
    fn product(&mut self, depth: u32) -> Result<Term, UnitError> {
        let mut acc = self.factor(depth)?;
        let mut divided = false;
        loop {
            match self.peek() {
                Some(Tok::Slash) => {
                    self.pos += 1;
                    acc = acc.div(self.factor(depth)?)?;
                    divided = true;
                }
                Some(Tok::Star | Tok::Ident(_) | Tok::LParen) if divided => {
                    return syntax("ambiguous unit: put the denominator in parentheses, e.g. J/(mol·K)");
                }
                Some(Tok::Star) => {
                    self.pos += 1;
                    acc = acc.mul(self.factor(depth)?)?;
                }
                Some(Tok::Ident(_) | Tok::LParen) => acc = acc.mul(self.factor(depth)?)?,
                _ => return Ok(acc),
            }
        }
    }

    /// `factor := atom [('^' ['+' | '-'] int) | superscript]`
    fn factor(&mut self, depth: u32) -> Result<Term, UnitError> {
        let atom = self.atom(depth)?;
        let power = match self.peek() {
            Some(Tok::Caret) => {
                self.pos += 1;
                let negative = match self.peek() {
                    Some(Tok::Minus) => {
                        self.pos += 1;
                        true
                    }
                    Some(Tok::Plus) => {
                        self.pos += 1;
                        false
                    }
                    _ => false,
                };
                match self.bump() {
                    Some(Tok::Int(n)) => Some(if negative { -n } else { n }),
                    _ => return syntax("expected an integer after `^`"),
                }
            }
            Some(Tok::Sup(n)) => {
                let n = *n;
                self.pos += 1;
                Some(n)
            }
            _ => None,
        };
        match power {
            None => Ok(atom),
            Some(n) => {
                if matches!(self.peek(), Some(Tok::Caret | Tok::Sup(_))) {
                    return syntax("a unit takes only one power");
                }
                atom.powi(n)
            }
        }
    }

    fn atom(&mut self, depth: u32) -> Result<Term, UnitError> {
        match self.bump() {
            Some(Tok::Ident(name)) => {
                let scale = lookup(&name).ok_or(UnitError::UnknownUnit(name))?;
                Ok(Term { dim: scale.dim, factor: scale.factor, offset: scale.offset })
            }
            Some(Tok::Int(1)) => Ok(Term::one()),
            Some(Tok::Int(_)) => syntax("only `1` may stand alone as a number"),
            Some(Tok::LParen) => {
                let inner = depth + 1;
                if inner > MAX_DEPTH {
                    return syntax("parentheses nested too deeply");
                }
                let term = self.product(inner)?;
                match self.bump() {
                    Some(Tok::RParen) => Ok(term),
                    _ => syntax("missing `)`"),
                }
            }
            Some(_) => syntax("unexpected token"),
            None => syntax("unexpected end of unit expression"),
        }
    }
}

/// Parse a unit expression into its SI [`Scale`].
///
/// Grammar: units joined by `*`, `·`, `⋅` or whitespace (multiply) and `/` (divide), all
/// evaluated left to right. Once a `/` has appeared, a following unit joined by `*`, `·`, `⋅` or
/// a space (as in `J/mol K`) is ambiguous and is [`UnitError::Syntax`]: write `J/(mol·K)`.
/// Repeated division (`m/s/s`) is fine. A unit takes one integer power, written `^n`, `^-n` or
/// with a single superscript digit (`²`, `³`, `⁻¹`). Parentheses group, nested at most 16
/// deep. A bare `1` is allowed as a unit (`1/s`). Input longer than 256 bytes, or empty, is
/// [`UnitError::Syntax`].
///
/// The offset units `°C` and `°F` are accepted only on their own (parentheses aside); any
/// product, quotient or power involving one is [`UnitError::AffineTemperature`]. Use `Δ°C` or
/// `Δ°F` for temperature differences in compounds. The returned [`Scale::text`] is the input
/// with surrounding whitespace trimmed.
pub fn parse_unit(expr: &str) -> Result<Scale, UnitError> {
    if expr.len() > MAX_INPUT_BYTES {
        return syntax("unit expression too long");
    }
    let text = expr.trim();
    let toks = tokenize(text)?;
    if toks.is_empty() {
        return syntax("empty unit expression");
    }
    let mut parser = Parser { toks, pos: 0 };
    let term = parser.product(0)?;
    if parser.peek().is_some() {
        return syntax("unexpected token after unit expression");
    }
    Ok(Scale { dim: term.dim, factor: term.factor, offset: term.offset, text: text.to_string() })
}
