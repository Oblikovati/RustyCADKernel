//! TopOpeBRepTool_GeomTool and TopOpeBRepTool_CurveTool: the curve options and the
//! section curves (approximations of intersection polylines, p-curves on faces).
use rustycad_tkbool::top_ope_brep_tool::{
    SectionCurves, TopOpeBRepToolCurveTool, TopOpeBRepToolGeomTool, TopOpeBRepToolOutCurveType,
};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point2, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;
use std::f64::consts::PI;

/// The square face [-2, 2]^2 on z = 1.
fn plane_face() -> Result<Shape, Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(-2., -2.), (2., -2.), (2., 2.), (-2., 2.)] {
        polygon.add_point(Point3::new(x, y, 1.))?;
    }
    polygon.close()?;
    Ok(BRepLibMakeFace::from_wire(&polygon.wire()?, true)?
        .face()?
        .into_shape())
}

/// The lateral face of the cylinder of radius 1 and height 2 on the Z axis.
fn cylinder_face() -> Result<Shape, Failure> {
    let solid = BRepPrimAPIMakeCylinder::new(1., 2.).shape()?.clone();
    let faces: Vec<Shape> = Explorer::new(&solid, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    for f in faces {
        if BRepAdaptorSurface::with_face(&topods::face(&f)?, false)?.get_type()?
            == SurfaceType::Cylinder
        {
            return Ok(f);
        }
    }
    Err(Failure::new("no cylindrical face"))
}

/// The polylines through n points of the unit circle at z = 1 between the angles 0.3
/// and 2.5, with their UV on the plane face (frame at the origin) and the cylinder
/// (`v_cyl` as V).
fn circle_polylines(angles: &[f64], v_cyl: f64) -> Result<SectionCurves, Failure> {
    let n = angles.len() as i32;
    let mut p: Array1<'static, Point3> =
        Array1::with_bounds(1, n).map_err(|e| Failure::new(&format!("{e:?}")))?;
    let mut p1: Array1<'static, Point2> =
        Array1::with_bounds(1, n).map_err(|e| Failure::new(&format!("{e:?}")))?;
    let mut p2: Array1<'static, Point2> =
        Array1::with_bounds(1, n).map_err(|e| Failure::new(&format!("{e:?}")))?;
    let face = topods::face(&plane_face()?)?;
    let plane = BRepAdaptorSurface::with_face(&face, false)?.plane()?;
    for (i, a) in angles.iter().enumerate() {
        let q = Point3::new(a.cos(), a.sin(), 1.);
        let (u, v) = rustycad_tkmath::elslib::plane_parameters(plane.position(), q);
        (p[i as i32 + 1], p1[i as i32 + 1], p2[i as i32 + 1]) =
            (q, Point2::new(u, v), Point2::new(*a, v_cyl));
    }
    Ok(SectionCurves {
        c3d: TopOpeBRepToolCurveTool::make_bspline1_from_pnt(&p)?,
        pc1: TopOpeBRepToolCurveTool::make_bspline1_from_pnt2d(&p1)?,
        pc2: TopOpeBRepToolCurveTool::make_bspline1_from_pnt2d(&p2)?,
    })
}

/// n angles evenly between 0.3 and 2.5.
fn angles(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| 0.3 + 2.2 * i as f64 / (n - 1) as f64)
        .collect()
}

/// The approximating curve tool.
fn approx_tool() -> TopOpeBRepToolCurveTool {
    TopOpeBRepToolCurveTool::with_type(TopOpeBRepToolOutCurveType::Approx)
}

#[test]
fn the_geom_tool_defaults_to_bspline1_with_every_curve() {
    let gt = TopOpeBRepToolGeomTool::default();
    assert_eq!(gt.type_c3d(), TopOpeBRepToolOutCurveType::BSpline1);
    assert!(gt.comp_c3d() && gt.comp_pc1() && gt.comp_pc2());
    assert_eq!(
        gt.get_tolerances(),
        (precision::approximation(), precision::p_approximation())
    );
    assert_eq!(gt.nb_pnt_max(), 30);
}

#[test]
fn the_geom_tool_definitions_change_only_their_options() {
    let mut gt = TopOpeBRepToolGeomTool::default();
    gt.define(TopOpeBRepToolOutCurveType::Interpol, false, true, false);
    gt.define_pcurves1(false);
    gt.define_pcurves2(true);
    gt.set_tolerances(1e-3, 1e-4);
    gt.set_nb_pnt_max(12);
    assert_eq!(gt.type_c3d(), TopOpeBRepToolOutCurveType::Interpol);
    assert_eq!(
        (gt.comp_c3d(), gt.comp_pc1(), gt.comp_pc2()),
        (false, false, true)
    );
    let ct = TopOpeBRepToolCurveTool::with_geom_tool(&gt);
    assert_eq!(*ct.get_geom_tool(), gt);
    assert_eq!(ct.get_geom_tool().get_tolerances(), (1e-3, 1e-4));
}

#[test]
fn a_bspline1_passes_through_its_points_at_their_indices() -> Result<(), Failure> {
    let c = circle_polylines(&angles(5), 1.)?;
    let b: Handle<GeomBSplineCurve> = c.c3d.clone().cast();
    let curve = b.get().ok_or_else(|| Failure::new("not a B-spline"))?;
    assert_eq!((curve.degree(), curve.nb_poles()), (1, 5));
    let q = curve.value(3.)?;
    let a = angles(5)[2];
    assert!(q.distance(&Point3::new(a.cos(), a.sin(), 1.)) < 1e-15);
    let b2: Handle<Geom2dBSplineCurve> = c.pc2.clone().cast();
    assert_eq!(b2.get().map(|c| c.nb_poles()), Some(5));
    Ok(())
}

#[test]
fn ellipses_on_cylinders_are_not_projectable() -> Result<(), Failure> {
    let cyl = cylinder_face()?;
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(
        Axis2::new(Point3::new(0., 0., 1.), Direction3::Z)?,
        1.,
    )?)
    .cast();
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
        Point3::new(1., 0., 0.),
        Direction3::Z,
    )))
    .cast();
    let ellipse: Handle<dyn GeomCurve> = Handle::new(GeomEllipse::new(
        Axis2::new(Point3::new(0., 0., 1.), Direction3::Z)?,
        1.5,
        1.,
    )?)
    .cast();
    assert!(!TopOpeBRepToolCurveTool::is_projectable(&cyl, &ellipse)?);
    assert!(TopOpeBRepToolCurveTool::is_projectable(&cyl, &circle)?);
    assert!(TopOpeBRepToolCurveTool::is_projectable(&cyl, &line)?);
    assert!(TopOpeBRepToolCurveTool::is_projectable(
        &plane_face()?,
        &circle
    )?);
    Ok(())
}

#[test]
fn a_pcurve_out_of_the_period_is_shifted_into_the_face() -> Result<(), Failure> {
    let cyl = cylinder_face()?;
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(
        Axis2::new(Point3::new(0., 0., 1.), Direction3::Z)?,
        1.,
    )?)
    .cast();
    let (pc, tol) = TopOpeBRepToolCurveTool::make_pcurve_on_face(&cyl, &circle, -3., -1.)?;
    let mid = pc
        .get()
        .ok_or_else(|| Failure::new("null p-curve"))?
        .value(-2.)?;
    assert!((0. ..=2. * PI).contains(&mid.x()), "u = {}", mid.x());
    assert!((mid.y() - 1.).abs() < 1e-9);
    assert!(tol >= 0.);
    Ok(())
}

#[test]
fn the_section_of_a_plane_and_a_cylinder_is_approximated() -> Result<(), Failure> {
    let mut input = circle_polylines(&angles(12), 1.)?;
    let made = approx_tool().make_curves(1., 12., &mut input, &plane_face()?, &cylinder_face()?)?;
    let made = made.ok_or_else(|| Failure::new("not done"))?;
    let c = made
        .curves
        .c3d
        .get()
        .ok_or_else(|| Failure::new("null curve"))?;
    let mid = c.value(0.5 * (c.first_parameter() + c.last_parameter()))?;
    assert!((mid.x().hypot(mid.y()) - 1.).abs() < 1e-5);
    assert!(!made.curves.pc1.is_null() && !made.curves.pc2.is_null());
    assert!(made.tol_reached3d >= 0. && made.tol_reached2d >= 0.);
    Ok(())
}

#[test]
fn without_the_3d_curve_nothing_is_made() -> Result<(), Failure> {
    let mut ct = approx_tool();
    ct.change_geom_tool().define_curves(false);
    let mut input = circle_polylines(&angles(12), 1.)?;
    assert!(
        ct.make_curves(1., 12., &mut input, &plane_face()?, &cylinder_face()?)?
            .is_none()
    );
    Ok(())
}

#[test]
fn a_pcurve_off_the_face_fails_the_approximation() -> Result<(), Failure> {
    let mut input = circle_polylines(&angles(12), 5.)?;
    assert!(
        approx_tool()
            .make_curves(1., 12., &mut input, &plane_face()?, &cylinder_face()?)?
            .is_none()
    );
    Ok(())
}

#[test]
fn close_poles_of_long_polylines_are_dropped_from_the_inputs() -> Result<(), Failure> {
    let mut a = angles(110);
    // Two poles 2e-6 after others: too close in 3d, but kept by the plane UV unless the
    // 2d tolerance is large.
    a.insert(20, a[19] + 2e-6);
    a.insert(50, a[49] + 2e-6);
    let mut input = circle_polylines(&a, 1.)?;
    let mut ct = approx_tool();
    ct.change_geom_tool().set_tolerances(1e-6, 1e-3);
    let _ = ct.make_curves(1., 112., &mut input, &plane_face()?, &cylinder_face()?)?;
    let b: Handle<GeomBSplineCurve> = input.c3d.clone().cast();
    assert_eq!(b.get().map(|c| c.nb_poles()), Some(110));
    Ok(())
}

#[test]
fn a_shape_that_is_not_a_face_is_refused() -> Result<(), Failure> {
    let edges: Vec<Shape> =
        Explorer::new(&plane_face()?, ShapeEnum::Edge, ShapeEnum::Shape)?.collect();
    let edge = edges
        .first()
        .cloned()
        .ok_or_else(|| Failure::new("no edge"))?;
    let mut input = circle_polylines(&angles(12), 1.)?;
    let error = approx_tool()
        .make_curves(1., 12., &mut input, &plane_face()?, &edge)
        .err();
    assert_eq!(
        error.map(|e| e.exception_type().to_string()),
        Some("Standard_TypeMismatch".to_string())
    );
    Ok(())
}
