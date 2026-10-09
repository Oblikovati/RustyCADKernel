//! BRepFill_Edge3DLaw: the location law along the edges of a path.
use rustycad_tkbool::brep_fill::{BRepFillEdge3DLaw, BRepFillLocationLaw};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillCorrectedFrenet, GeomFillCurveAndTrihedron, GeomFillLocationLaw, GeomFillTrihedronLaw,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;

#[test]
fn a_bent_path_has_one_law_per_edge_and_cumulated_lengths() -> Result<(), Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for p in [
        Point3::new(0., 0., 0.),
        Point3::new(0., 0., 2.),
        Point3::new(0., 3., 2.),
    ] {
        polygon.add_point(p)?;
    }
    let path = polygon.wire()?;
    let tri: Handle<dyn GeomFillTrihedronLaw> = Handle::new(GeomFillCorrectedFrenet::new()).cast();
    let model: Handle<dyn GeomFillLocationLaw> =
        Handle::new(GeomFillCurveAndTrihedron::new(&tri)).cast();
    let law = BRepFillEdge3DLaw::new(&path, &model)?;
    assert_eq!(law.nb_law()?, 2);
    assert!(!law.is_closed()?);
    let (f, l) = law.curvilinear_bounds(2)?;
    assert!((f - 2.).abs() < 1e-9 && (l - 5.).abs() < 1e-9);
    assert_eq!(law.parameter(10.)?.0, 0);
    // The corner is not G1; the open ends are not G0.
    assert_eq!(law.is_g1(0, 1e-7, 1e-4)?, -1);
    Ok(())
}
