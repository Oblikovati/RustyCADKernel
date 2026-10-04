//! Source-compatible edge tangents, including caller output mutation on failure.
use rustycad_tkbrep::{brep::BRepTool, topods::Edge};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{self, Vector3};
/// Normalize and orient the spatial derivative; `edge_tangent(&edge,0.5,&mut tangent)?`.
pub fn edge_tangent(edge: &Edge, parameter: f64, tangent: &mut Vector3) -> Result<bool, Failure> {
    if BRepTool::degenerated(edge)? {
        return Ok(false);
    }
    let (curve, _, _) = BRepTool::curve(edge)?;
    *tangent = curve.d1(parameter)?.1;
    let magnitude = tangent.magnitude();
    if magnitude > gp::resolution() {
        *tangent /= magnitude;
    } else {
        return Ok(false);
    }
    if edge.orientation() == Orientation::Reversed {
        tangent.reverse();
    }
    Ok(true)
}
