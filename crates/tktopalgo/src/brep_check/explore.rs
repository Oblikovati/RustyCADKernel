// Rust translation of the TopExp_Explorer loops shared by the BRepCheck checkers
// (ModelingAlgorithms/TKTopAlgo/BRepCheck) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The explorer loops that every BRepCheck checker repeats.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The sub-shapes of `kind` of `shape` in explorer order (`TopExp_Explorer(S, kind)`).
pub(crate) fn sub_shapes(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(found)
}

/// The InContext test `for (exp(S, kind); ...) if (Current().IsSame(myShape)) break;
/// if (!exp.More())`: whether `context` holds a sub-shape of `kind` same as `shape`.
pub(crate) fn holds_same(context: &Shape, kind: ShapeEnum, shape: &Shape) -> Result<bool, Failure> {
    Ok(sub_shapes(context, kind)?
        .iter()
        .any(|sub| sub.is_same(shape)))
}

/// The native `IsOriented(S)`: FORWARD or REVERSED.
pub(crate) fn is_oriented(shape: &Shape) -> bool {
    matches!(
        shape.orientation(),
        Orientation::Forward | Orientation::Reversed
    )
}
