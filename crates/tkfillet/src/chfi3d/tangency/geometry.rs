use rustycad_tkbrep::{
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
pub(super) struct TangencyGeometry {
    pub first_curve: Handle<dyn Geom2dCurve>,
    pub second_curve: Handle<dyn Geom2dCurve>,
    pub first_surface: Handle<dyn GeomSurface>,
    pub second_surface: Handle<dyn GeomSurface>,
    pub first: f64,
    pub last: f64,
}
impl TangencyGeometry {
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
