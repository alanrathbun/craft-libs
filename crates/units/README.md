# craft-units

Dimensional analysis and unit-aware quantity arithmetic for the Craft suite. A `Dim` is a vector of
integer exponents over the seven SI base dimensions plus plane angle, and a `Quantity` is an `f64`
magnitude in SI base units paired with its `Dim`. Every operation is total and returns a `UnitError`
rather than panicking, and the crate knows nothing about any application's types.
