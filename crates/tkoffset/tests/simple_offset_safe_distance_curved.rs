//! Regression for native endpoint trimming and ordered samples on curved face junctions.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomSurface, GeomSurfaceOfLinearExtrusion};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkoffset::brep_offset::compute_simple_offset_max_angle;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPISewing};

const NATIVE_CURVED_JUNCTION_ANGLE: f64 = 2.059_189_731_238_268;

#[test]
fn curved_junction_uses_the_native_trimmed_sample_interval() -> Result<(), Failure> {
    let shape = sew_curved_faces()?;
    let measured = compute_simple_offset_max_angle(&shape)?;
    assert!((measured - NATIVE_CURVED_JUNCTION_ANGLE).abs() < 1.0e-13);
    Ok(())
}

fn sew_curved_faces() -> Result<Shape, Failure> {
    let basis: Handle<dyn GeomCurve> = Handle::new(curved_bezier()?).cast();
    let first = extrusion_face(&basis, Direction3::new(0.0, 1.0, 0.0)?)?;
    let second = extrusion_face(&basis, Direction3::new(0.0, 0.0, 1.0)?)?;
    let mut sewing = BRepBuilderAPISewing::new(1.0e-6, true, true, true, false);
    sewing.add(&first)?;
    sewing.add(&second)?;
    sewing.perform(&ProgressRange::default())?;
    Ok(sewing.sewed_shape().clone())
}

fn curved_bezier() -> Result<GeomBezierCurve, Failure> {
    let mut poles = Array1::<Point3>::with_bounds(1, 4).expect("valid Bezier pole bounds");
    poles[1] = Point3::new(0.0, 0.0, 0.0);
    poles[2] = Point3::new(1.0, 0.0, 2.0);
    poles[3] = Point3::new(2.0, 1.0, -1.0);
    poles[4] = Point3::new(3.0, 0.0, 0.0);
    GeomBezierCurve::new(&poles)
}

fn extrusion_face(basis: &Handle<dyn GeomCurve>, direction: Direction3) -> Result<Shape, Failure> {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfLinearExtrusion::new(basis, direction)?).cast();
    let maker =
        BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0.0, 1.0, 0.0, 1.0), 1.0e-7)?;
    Ok(maker.face()?.into_shape())
}
