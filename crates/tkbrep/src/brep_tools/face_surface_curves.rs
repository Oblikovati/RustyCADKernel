//! Shared pcurve selection from ChFi3d::IsTangentFaces and BRepOffset_Analyse::CheckMixedContinuity.
use crate::{
    brep::{BRepTool, EdgePCurve},
    topexp::Explorer,
    topods::{self, Edge, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{
    geom::GeomSurface,
    top_abs::{Orientation, ShapeEnum},
};
/// Pcurves, located surfaces and the second pcurve range used by continuity analysis.
/// Obtain with `FaceSurfaceCurves::load(&edge, &first_face, &second_face)?`.
pub struct FaceSurfaceCurves {
    /// Pcurve selected on the first face occurrence.
    pub first_curve: Handle<dyn Geom2dCurve>,
    /// Pcurve selected on the second face occurrence.
    pub second_curve: Handle<dyn Geom2dCurve>,
    /// Located support surface of the first face.
    pub first_surface: Handle<dyn GeomSurface>,
    /// Located support surface of the second face.
    pub second_surface: Handle<dyn GeomSurface>,
    /// First bound from the second pcurve lookup, matching native overwrite order.
    pub first: f64,
    /// Last bound from the second pcurve lookup.
    pub last: f64,
}
impl FaceSurfaceCurves {
    /// Select seam occurrences and return None for absent support; `FaceSurfaceCurves::load(&edge, &first, &second)?`.
    pub fn load(edge: &Edge, first: &Face, second: &Face) -> Result<Option<Self>, Failure> {
        let Some((a, b)) = pcurve_pair(edge, first, second)? else {
            return Ok(None);
        };
        if a.pcurve.is_null() || b.pcurve.is_null() {
            return Ok(None);
        }
        let first_surface = BRepTool::surface(first)?;
        let second_surface = BRepTool::surface(second)?;
        if first_surface.is_null() || second_surface.is_null() {
            return Ok(None);
        }
        Ok(Some(Self {
            first_curve: a.pcurve,
            second_curve: b.pcurve,
            first_surface,
            second_surface,
            first: b.first,
            last: b.last,
        }))
    }
}
fn pcurve_pair(
    edge: &Edge,
    first: &Face,
    second: &Face,
) -> Result<Option<(EdgePCurve, EdgePCurve)>, Failure> {
    if !first.is_same(second)
        && BRepTool::is_closed_on_face(edge, first)?
        && BRepTool::is_closed_on_face(edge, second)?
    {
        return seam_pcurves(edge, first, second);
    }
    let a = BRepTool::curve_on_face(edge, first)?;
    let mut reversed = edge.clone();
    if first.is_same(second) {
        reversed.reverse();
    }
    Ok(Some((a, BRepTool::curve_on_face(&reversed, second)?)))
}
fn seam_pcurves(
    edge: &Edge,
    first: &Face,
    second: &Face,
) -> Result<Option<(EdgePCurve, EdgePCurve)>, Failure> {
    let mut first = first.clone();
    first.set_orientation(Orientation::Forward);
    let Some(found) =
        Explorer::new(&first, ShapeEnum::Edge, ShapeEnum::Shape)?.find(|shape| shape.is_same(edge))
    else {
        return Ok(None);
    };
    let mut found = topods::edge(&found)?;
    let a = BRepTool::curve_on_face(&found, &first)?;
    let mut second = second.clone();
    second.set_orientation(Orientation::Forward);
    found.reverse();
    Ok(Some((a, BRepTool::curve_on_face(&found, &second)?)))
}
