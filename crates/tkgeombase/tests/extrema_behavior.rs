//! The native quirks of the classic Extrema package that the port keeps
//! (docs/tkgeombase-progress.md, Preserved native behaviour); the numeric probe
//! compares the same paths with OCCT (tools/verify_tkgeombase_numeric.py).
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomCircle, GeomCurve, GeomLine, GeomPlane, GeomSphericalSurface,
    GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurface, GeomAdaptorSurfaceOfRevolution,
};
use rustycad_tkgeombase::extrema::{
    ExtAlgo, ExtCS, ExtElCS, ExtElSS, ExtFlag, ExtPElC2d, ExtPRevS, ExtPS, ExtSS, FuncExtCS,
    FuncExtSS, FuncPSNorm, GenExtPS, GlobOptFuncCQuadric, GlobOptFuncConicS, Sampling,
};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Axis22d, Circle2, Cone3, Direction2, Direction3, Ellipse2, Line3, Point2,
    Point3, Sphere3,
};
use rustycad_tkmath::math::Vector;
use std::f64::consts::PI;

fn kind(failure: Failure) -> &'static str {
    failure.exception_type()
}

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

fn frame(origin: Point3) -> Axis3 {
    Axis3::new(origin, dir(0.0, 0.0, 1.0), dir(1.0, 0.0, 0.0)).unwrap()
}

fn plane_at(z: f64) -> GeomAdaptorSurface {
    let plane: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::new(frame(Point3::new(0.0, 0.0, z)))).cast();
    GeomAdaptorSurface::with_bounds(&plane, (-3.0, 3.0, -3.0, 3.0)).unwrap()
}

fn sphere(radius: f64) -> GeomAdaptorSurface {
    let sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(frame(Point3::default()), radius).unwrap()).cast();
    GeomAdaptorSurface::with_surface(&sphere).unwrap()
}

fn line(origin: Point3, direction: Direction3, range: (f64, f64)) -> GeomAdaptorCurve {
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(origin, direction))).cast();
    GeomAdaptorCurve::with_range(&line, range.0, range.1).unwrap()
}

fn vector(values: &[f64]) -> Vector<'static> {
    let mut vector = Vector::with_bounds(1, values.len() as i32).unwrap();
    for (index, value) in values.iter().enumerate() {
        vector[index as i32 + 1] = *value;
    }
    vector
}

#[test]
fn ext_cs_parallel_case_has_a_distance_but_no_point() {
    let curve = line(Point3::new(0.0, 0.0, 1.0), dir(1.0, 0.0, 0.0), (-5.0, 5.0));
    let surface = plane_at(0.0);
    let extrema = ExtCS::with_curve(&curve, &surface, 1.0e-9, 1.0e-9).unwrap();
    assert!(extrema.is_parallel().unwrap());
    assert_eq!(extrema.nb_ext().unwrap(), 1);
    assert_eq!(extrema.square_distance(1).unwrap(), 1.0);
    assert_eq!(kind(extrema.points(1).unwrap_err()), "Standard_OutOfRange");
}

#[test]
fn ext_ss_keeps_the_parallel_flag_of_an_earlier_perform() {
    let (s2, planes, ball) = (plane_at(0.0), plane_at(2.0), sphere(1.0));
    let mut extrema = ExtSS::new();
    extrema
        .initialize(&s2, [-3.0, 3.0, -3.0, 3.0], 1.0e-9)
        .unwrap();
    extrema
        .perform(&planes, [-3.0, 3.0, -3.0, 3.0], 1.0e-9)
        .unwrap();
    assert!(extrema.is_parallel().unwrap());
    assert_eq!(kind(extrema.points(1).unwrap_err()), "Standard_OutOfRange");
    // The generic search does not reset the flag.
    extrema
        .perform(&ball, [0.0, 2.0 * PI, -PI / 2.0, PI / 2.0], 1.0e-9)
        .unwrap();
    assert!(extrema.is_parallel().unwrap());
}

#[test]
fn func_ext_ss_saves_every_state() {
    let (s1, s2) = (plane_at(0.0), plane_at(1.0));
    let mut function = FuncExtSS::with_surfaces(&s1, &s2);
    let mut f = vector(&[0.0; 4]);
    for _ in 0..2 {
        function
            .value(&vector(&[0.5, 0.5, 0.5, 0.5]), &mut f)
            .unwrap();
        function.get_state_number().unwrap();
    }
    assert_eq!(function.nb_ext(), 2);
}

#[test]
fn func_ext_cs_merges_states_by_curve_parameter_only() {
    let curve = line(Point3::new(0.0, 0.0, 1.0), dir(1.0, 0.0, 0.0), (-5.0, 5.0));
    let surface = plane_at(0.0);
    let mut function = FuncExtCS::with_curve_surface(&curve, &surface);
    let mut f = vector(&[0.0; 3]);
    for uv in [[0.5, 0.5, 0.5], [0.5, 1.5, -1.0]] {
        function.value(&vector(&uv), &mut f).unwrap();
        function.get_state_number().unwrap();
    }
    assert_eq!(function.nb_ext(), 1);
}

#[test]
fn func_ps_norm_merges_states_within_p_confusion_in_uv() {
    let surface = plane_at(0.0);
    let mut function = FuncPSNorm::with_point_surface(Point3::new(0.0, 0.0, 1.0), &surface);
    let mut f = vector(&[0.0; 2]);
    for uv in [[0.5, 0.5], [0.5, 0.5 + 1.0e-10], [0.5, 0.6]] {
        function.value(&vector(&uv), &mut f).unwrap();
        function.get_state_number().unwrap();
    }
    assert_eq!(function.nb_ext(), 2);
    assert_eq!(
        kind(FuncPSNorm::new().point(1).unwrap_err()),
        "Standard_TypeMismatch"
    );
}

#[test]
fn glob_opt_func_c_quadric_on_another_surface_returns_u_bounds() {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for (i, j, z) in [(1, 1, 0.0), (1, 2, 1.0), (2, 1, 0.5), (2, 2, 0.0)] {
        poles[(i, j)] = Point3::new(f64::from(i), f64::from(j), z);
    }
    let bezier: Handle<dyn GeomSurface> =
        Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast();
    let surface = GeomAdaptorSurface::with_surface(&bezier).unwrap();
    let curve = line(Point3::default(), dir(0.0, 0.0, 1.0), (-1.0, 1.0));
    let function = GlobOptFuncCQuadric::with_surface(&curve, &surface).unwrap();
    let mut value = 0.0;
    assert!(!function.value(&vector(&[0.0]), &mut value).unwrap());
    let mut uv = vector(&[9.0, 9.0]);
    function
        .quadric_parameters(&vector(&[0.0]), &mut uv)
        .unwrap();
    // (Uf, Ul), not (Uf, Vf).
    assert_eq!((uv[1], uv[2]), (0.0, 1.0));
}

#[test]
fn glob_opt_func_conic_s_without_conic_is_a_null_object() {
    let surface = plane_at(0.0);
    let function = GlobOptFuncConicS::new(&surface).unwrap();
    let mut value = 0.0;
    let failure = function
        .value(&vector(&[0.0, 0.0]), &mut value)
        .unwrap_err();
    assert_eq!(kind(failure), "Standard_NullObject");
}

#[test]
fn gen_ext_ps_needs_two_samples_in_each_direction() {
    let surface = sphere(1.0);
    let sampling = Sampling {
        nb_u: 1,
        nb_v: 10,
        tol_u: 1.0e-9,
        tol_v: 1.0e-9,
    };
    let failure = GenExtPS::new().initialize(&surface, sampling).unwrap_err();
    assert_eq!(kind(failure), "Standard_OutOfRange");
}

#[test]
fn ext_ps_trimmed_distances_are_never_computed() {
    let surface = sphere(2.0);
    let extrema = ExtPS::with_surface(
        Point3::new(0.5, 0.5, 3.0),
        &surface,
        1.0e-9,
        1.0e-9,
        ExtFlag::MinMax,
        ExtAlgo::Grad,
    )
    .unwrap();
    let (distances, points) = extrema.trimmed_square_distances();
    assert_eq!(distances, [0.0; 4]);
    assert_eq!(points, [Point3::default(); 4]);
    assert_eq!(
        kind(ExtPS::new().square_distance(0).unwrap_err()),
        "Standard_OutOfRange"
    );
}

#[test]
fn ext_p_rev_s_reinitialized_on_the_same_handle_takes_the_grid_search() {
    let circle: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(3.0, 0.0, 0.0), dir(0.0, 1.0, 0.0)).unwrap(),
            1.0,
        )
        .unwrap(),
    )
    .cast();
    let basis: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_curve(&circle).unwrap()).cast();
    let axis = Axis1::new(Point3::default(), dir(0.0, 0.0, 1.0));
    let revolution =
        Handle::new(GeomAdaptorSurfaceOfRevolution::with_curve_and_axis(&basis, axis).unwrap());
    let point = Point3::new(1.0, 2.0, 0.5);
    let range = ((0.0, 2.0 * PI), (0.0, 2.0 * PI));
    let mut extrema =
        ExtPRevS::with_range(point, &revolution, range.0, range.1, (1.0e-9, 1.0e-9)).unwrap();
    let analytic = extrema.nb_ext().unwrap();
    extrema
        .initialize(&revolution, range.0, range.1, (1.0e-9, 1.0e-9))
        .unwrap();
    extrema.perform(point).unwrap();
    let surface: &dyn Adaptor3dSurface = &*revolution;
    let sampling = Sampling {
        nb_u: 32,
        nb_v: 32,
        tol_u: 1.0e-9,
        tol_v: 1.0e-9,
    };
    let grid = GenExtPS::with_range(
        point,
        surface,
        sampling,
        range.0,
        range.1,
        (ExtFlag::MinMax, ExtAlgo::Grad),
    )
    .unwrap();
    assert_eq!(extrema.nb_ext().unwrap(), grid.nb_ext().unwrap());
    for n in 1..=grid.nb_ext().unwrap() {
        assert_eq!(
            extrema.square_distance(n).unwrap(),
            grid.square_distance(n).unwrap()
        );
    }
    assert!(analytic > 0);
}

#[test]
fn ext_el_ss_sphere_pairs_are_done_then_not_implemented() {
    let ball = Sphere3::new(frame(Point3::default()), 1.0).unwrap();
    let mut extrema = ExtElSS::new();
    let failure = extrema.perform_spheres(&ball, &ball).unwrap_err();
    assert_eq!(kind(failure), "Standard_NotImplemented");
    assert!(extrema.is_done());
    assert_eq!(extrema.nb_ext().unwrap(), 0);
}

#[test]
fn ext_el_cs_line_cone_is_not_implemented() {
    let cone = Cone3::new(frame(Point3::default()), 0.5, 1.0).unwrap();
    let failure =
        ExtElCS::line_cone(Line3::new(Point3::default(), dir(1.0, 0.0, 0.0)), cone).unwrap_err();
    assert_eq!(kind(failure), "Standard_NotImplemented");
}

#[test]
fn ext_p_el_c2d_marks_both_circle_extrema_as_minima_and_no_ellipse_extremum() {
    let position =
        Axis22d::new(Point2::default(), Direction2::new(1.0, 0.0).unwrap(), true).unwrap();
    let point = Point2::new(0.5, 0.25);
    let circle = ExtPElC2d::with_circle(
        point,
        Circle2::new(position, 2.0).unwrap(),
        1.0e-9,
        0.0,
        2.0 * PI,
    )
    .unwrap();
    assert_eq!(circle.nb_ext().unwrap(), 2);
    assert!(circle.is_min(1).unwrap() && circle.is_min(2).unwrap());
    let ellipse = ExtPElC2d::with_ellipse(
        point,
        Ellipse2::new(position, 3.0, 1.0).unwrap(),
        1.0e-9,
        0.0,
        2.0 * PI,
    )
    .unwrap();
    assert!(ellipse.nb_ext().unwrap() > 0);
    for n in 1..=ellipse.nb_ext().unwrap() {
        assert!(!ellipse.is_min(n).unwrap());
    }
}
