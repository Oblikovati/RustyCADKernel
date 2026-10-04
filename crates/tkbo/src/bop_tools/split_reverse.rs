//! Edge IsSplitToReverse from BOPTools_AlgoTools.cxx.
use super::algo_tools2d::edge_tangent;
use crate::int_tools::Context;
use rustycad_tkbrep::{brep::BRepTool, topods::Edge};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Vector3;
use rustycad_tktopalgo::brep_lib;
/// Compare oriented tangents at matching projected points; `is_split_edge_to_reverse(&split,&original,&context,&mut error)?`.
pub fn is_split_edge_to_reverse(
    split: &Edge,
    original: &Edge,
    context: &Context,
    error: &mut i32,
) -> Result<bool, Failure> {
    if BRepTool::degenerated(split)? || BRepTool::degenerated(original)? {
        *error = 1;
        return Ok(false);
    }
    *error = 0;
    let (split_curve, _, _) = BRepTool::curve(split)?;
    let (original_curve, _, _) = BRepTool::curve(original)?;
    if split_curve == original_curve {
        return Ok(split.orientation() != original.orientation());
    }
    let (first, last) = valid_split_range(split)?;
    let increment = (last - first) / 11.;
    for index in 1..11 {
        let parameter = first + f64::from(index) * increment;
        if let Some(reverse) = compare_at(split, original, context, parameter, error)? {
            *error = 0;
            return Ok(reverse);
        }
    }
    Ok(false)
}
fn valid_split_range(edge: &Edge) -> Result<(f64, f64), Failure> {
    let range = brep_lib::find_valid_range_of_edge(edge)?;
    if !range.is_valid {
        return BRepTool::range(edge);
    }
    match (range.first, range.last) {
        (Some(first), Some(last)) => Ok((first, last)),
        _ => Err(Failure::with_message(
            "Standard_ProgramError",
            "FindValidRange success with missing bounds; expected two parameters",
        )),
    }
}
fn compare_at(
    split: &Edge,
    original: &Edge,
    context: &Context,
    parameter: f64,
    error: &mut i32,
) -> Result<Option<bool>, Failure> {
    let mut split_tangent = Vector3::default();
    if !edge_tangent(split, parameter, &mut split_tangent)? {
        *error = 2;
        return Ok(None);
    }
    let (curve, _, _) = BRepTool::curve(split)?;
    let mut projected = 0.;
    if !context.project_point_on_edge(curve.value(parameter)?, original, &mut projected)? {
        *error = 3;
        return Ok(None);
    }
    let mut original_tangent = Vector3::default();
    if !edge_tangent(original, projected, &mut original_tangent)? {
        *error = 4;
        return Ok(None);
    }
    Ok(Some(split_tangent.dot(original_tangent) < 0.))
}
