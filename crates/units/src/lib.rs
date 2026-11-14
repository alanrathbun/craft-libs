#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! Dimensional analysis and unit-aware quantity arithmetic.
//!
//! A [`Dim`] is a vector of integer exponents over the seven SI base dimensions plus plane angle.
//! A [`Quantity`] is an `f64` magnitude in SI base units together with its [`Dim`]. Every
//! operation is total: dimension overflow, mismatched dimensions, fractional exponents, and
//! non-finite results come back as a [`UnitError`] instead of panicking.
//!
//! The crate knows nothing about any application's types, so it can sit under a word processor's
//! inline calculations or a spreadsheet's cell formulas alike.

mod catalog;
mod dim;
mod parse;
mod quantity;

pub use catalog::{CATALOG, Scale, UnitDef, lookup};
pub use dim::Dim;
pub use parse::parse_unit;
pub use quantity::{Quantity, UnitError};
