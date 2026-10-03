//! Source-faithful first OCC28131 fill regression from GeomFill_BSplineCurves_Test.cxx.
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomCurve, GeomSurface,
};
use rustycad_tkgeomalgo::geom_api::to_3d;
use rustycad_tkgeomalgo::geom_fill::{GeomFillBSplineCurves, GeomFillFillingStyle};
use rustycad_tkgeomalgo::geom2d_api::Interpolate;
use rustycad_tkgeombase::geom_convert::curve_to_bspline_curve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Axis3, Plane3, Point2, Point3, StandardDirection3, Vector2};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

#[test]
fn geom_fill_bspline_curves_occ28131_fill_surface_from_bezier_and_interpolated_curves() {
    let surface = occ28131_surface().unwrap();
    let geometry: Handle<dyn GeomSurface> = surface.cast();
    let mut maker = BRepBuilderAPIMakeFace::from_surface(&geometry, 0.0).unwrap();
    let shape = if maker.is_done() {
        maker.shape().unwrap().clone()
    } else {
        rustycad_tkbrep::topods::Shape::default()
    };
    assert!(!shape.is_null());
    assert!(
        BRepCheckAnalyzer::new(&shape, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

/// Builds the same three boundaries and Coons surface as the shared native fixture.
fn occ28131_surface() -> Result<Handle<GeomBSplineSurface>, Failure> {
    let (v0, v1) = (Point3::new(-17.6, 0.0, 0.0), Point3::new(0.0, 32.8, 0.0));
    let outline = outline_curve(v0, v1)?;
    let curve1 = lift_interpolation(
        first_side_points(v1),
        Some((Vector2::new(0.0, 1.0), Vector2::new(1.0, 0.0))),
        first_side_plane(),
    )?;
    let curve2 = lift_second_interpolation(second_side_points(v0), second_side_plane())?;
    GeomFillBSplineCurves::with_three(&outline, &curve1, &curve2, GeomFillFillingStyle::CoonsStyle)
        .map(|filling| filling.surface())
}

fn outline_curve(v0: Point3, v1: Point3) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    poles[1] = v0;
    poles[2] = Point3::new(v0.x(), (5.4 / 13.2) * v1.y(), 0.0);
    poles[3] = Point3::new((6.0 / 6.8) * v0.x(), v1.y(), 0.0);
    poles[4] = v1;
    let bezier: Handle<GeomBezierCurve> = Handle::new(GeomBezierCurve::new(&poles)?);
    let curve: Handle<dyn GeomCurve> = bezier.cast();
    curve_to_bspline_curve(&curve, ParameterisationType::TgtThetaOver2)
}

fn first_side_points(v1: Point3) -> [Point2; 2] {
    [Point2::new(-v1.y(), 0.0), Point2::new(0.0, 8.5 + 8.5 / 2.0)]
}

fn second_side_points(v0: Point3) -> [Point2; 3] {
    [
        Point2::new(-v0.x(), 0.0),
        Point2::new(-v0.x() - 2.6, 8.5),
        Point2::new(0.0, 8.5 + 8.5 / 2.0),
    ]
}

fn first_side_plane() -> Plane3 {
    let frame = Axis3::with_standard_frame(
        Point3::default(),
        StandardDirection3::X,
        StandardDirection3::NegY,
    );
    Plane3::from_position(frame)
}

fn second_side_plane() -> Plane3 {
    let frame = Axis3::with_standard_frame(
        Point3::default(),
        StandardDirection3::NegY,
        StandardDirection3::NegX,
    );
    Plane3::from_position(frame)
}

fn lift_interpolation(
    points: [Point2; 2],
    tangents: Option<(Vector2, Vector2)>,
    plane: Plane3,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let points = Handle::new(HArray1::from_owned(point_array(&points)));
    let mut interpolate = Interpolate::new(&points, false, 1.0e-6)?;
    if let Some((start, end)) = tangents {
        interpolate.load_ends(start, end, true)?;
    }
    interpolate.perform()?;
    let pcurve: Handle<dyn Geom2dCurve> = interpolate.curve()?.cast();
    to_3d(&pcurve, &plane).map(|curve| curve.cast())
}

fn lift_second_interpolation(
    points: [Point2; 3],
    plane: Plane3,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let points = Handle::new(HArray1::from_owned(point_array(&points)));
    let mut interpolate = Interpolate::new(&points, false, 1.0e-6)?;
    interpolate.perform()?;
    let pcurve: Handle<dyn Geom2dCurve> = interpolate.curve()?.cast();
    to_3d(&pcurve, &plane).map(|curve| curve.cast())
}

fn point_array(points: &[Point2]) -> Array1<'static, Point2> {
    let mut result = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (index, point) in points.iter().enumerate() {
        result[index as i32 + 1] = *point;
    }
    result
}
