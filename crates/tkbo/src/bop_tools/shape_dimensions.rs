//! Shape dimension and compound flattening from BOPTools_AlgoTools_2.cxx.
use rustycad_tkbrep::{
    topods::{Shape, ShapeIterator},
    toptools::MapOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
/// Append compound leaves, optionally deduplicating against an existing fence; `treat_compound(&shape,&mut leaves,Some(&mut fence))?`.
pub fn treat_compound(
    shape: &Shape,
    leaves: &mut Vec<Shape>,
    mut fence: Option<&mut MapOfShape>,
) -> Result<(), Failure> {
    if shape.shape_type()? != ShapeEnum::Compound {
        if fence.as_mut().is_none_or(|seen| seen.add(shape.clone())) {
            leaves.push(shape.clone());
        }
        return Ok(());
    }
    for child in ShapeIterator::new(shape, true, true) {
        treat_compound(&child, leaves, fence.as_deref_mut())?;
    }
    Ok(())
}
/// Return the minimum and maximum leaf dimensions; `dimensions(&shape,&mut minimum,&mut maximum)?`.
pub fn dimensions(shape: &Shape, minimum: &mut i32, maximum: &mut i32) -> Result<(), Failure> {
    *minimum = basic_dimension(shape)?;
    *maximum = *minimum;
    if *maximum >= 0 {
        return Ok(());
    }
    let mut leaves = Vec::new();
    let mut fence = MapOfShape::default();
    treat_compound(shape, &mut leaves, Some(&mut fence))?;
    if leaves.is_empty() {
        *minimum = -1;
        *maximum = -1;
        return Ok(());
    }
    *minimum = 3;
    *maximum = 0;
    for leaf in leaves {
        let dim = basic_dimension(&leaf)?;
        *minimum = (*minimum).min(dim);
        *maximum = (*maximum).max(dim);
    }
    Ok(())
}
/// Return a uniform dimension or -1 for empty/mixed compounds; `dimension(&shape)?`.
pub fn dimension(shape: &Shape) -> Result<i32, Failure> {
    let mut minimum = 0;
    let mut maximum = 0;
    dimensions(shape, &mut minimum, &mut maximum)?;
    Ok(if minimum == maximum { minimum } else { -1 })
}
fn basic_dimension(shape: &Shape) -> Result<i32, Failure> {
    Ok(match shape.shape_type()? {
        ShapeEnum::Vertex => 0,
        ShapeEnum::Edge | ShapeEnum::Wire => 1,
        ShapeEnum::Face | ShapeEnum::Shell => 2,
        ShapeEnum::Solid | ShapeEnum::CompSolid => 3,
        _ => -1,
    })
}
