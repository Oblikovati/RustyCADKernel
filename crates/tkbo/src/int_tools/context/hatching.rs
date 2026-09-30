use super::*;
use rustycad_tkbrep::{topexp::Explorer, topods};
use rustycad_tkg2d::{
    geom2d::{Geom2dCurve, Geom2dTrimmedCurve},
    geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom2d_hatch::Intersector;
impl Context {
    /// Returns a cached hatcher populated with oriented face pcurves; `context.hatcher(&face)?`.
    pub fn hatcher(&self, face: &Face) -> Result<Cached<Hatcher>, Failure> {
        self.hatchers.get(face, || build_hatcher(face))
    }
}
fn build_hatcher(face: &Face) -> Result<Hatcher, Failure> {
    let mut hatcher = Hatcher::new(&Intersector::new(1e-10, 1e-10), 1e-8, 1e-8, true, false);
    let face = Face::unchecked(face.oriented(Orientation::Forward));
    BRepTool::surface(&face)?;
    for shape in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        add_edge(&mut hatcher, &topods::edge(&shape)?, &face)?;
    }
    Ok(hatcher)
}
fn add_edge(hatcher: &mut Hatcher, edge: &Edge, face: &Face) -> Result<(), Failure> {
    let pcurve = BRepTool::curve_on_face(edge, face)?;
    if pcurve.pcurve.is_null()
        || (pcurve.first - pcurve.last).abs() < rustycad_tkernel::precision::p_confusion()
    {
        return Ok(());
    }
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dTrimmedCurve::new(
        &pcurve.pcurve,
        pcurve.first,
        pcurve.last,
        true,
        true,
    )?)
    .cast();
    hatcher.add_element(Geom2dAdaptorCurve::with_curve(&curve)?, edge.orientation());
    Ok(())
}
