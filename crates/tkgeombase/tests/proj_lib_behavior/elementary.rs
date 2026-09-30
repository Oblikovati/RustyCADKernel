//! The elementary projectors, the static Project overloads, PrjFunc and PrjResolve.
use super::support::{ax2, dir, kind, xoy};
use rustycad_tkernel::precision;
use rustycad_tkg3d::geom::{GeomLine, GeomSphericalSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::proj_lib::{
    self as projlib, Cone, Cylinder, Plane, PrjFunc, PrjResolve, PrjResolveBox, Project, Projector,
    Sphere, Torus,
};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis3, Circle3, Cone3, Cylinder3, Ellipse3, Line2, Line3, Plane3, Point2, Point3, Sphere3,
    Torus3,
};
use rustycad_tkmath::math::{Matrix, Vector};
use std::f64::consts::PI;

#[test]
fn projector_default_is_an_undone_bspline_with_null_curves() {
    let projector = Projector::new();
    assert_eq!(projector.get_type(), CurveType::BSplineCurve);
    assert!(!projector.is_done() && !projector.is_periodic());
    assert!(projector.bspline().is_null() && projector.bezier().is_null());
    assert_eq!(kind(projector.line()), "Standard_NoSuchObject");
}

#[test]
fn base_project_sets_other_curve() {
    let mut projector = Projector::new();
    projector.project_circle(&Circle3::default()).unwrap();
    assert_eq!(projector.get_type(), CurveType::OtherCurve);
    projector.done();
    projector.set_periodic();
    assert!(projector.is_done() && projector.is_periodic());
}

#[test]
fn plane_projects_conics_in_plane_coordinates() {
    let plane = Plane3::from_position(xoy());
    let center = Point3::new(1.0, 2.0, 3.0);
    let circle = Circle3::new(ax2(center, dir(0.0, 0.0, 1.0), dir(1.0, 0.0, 0.0)), 1.5).unwrap();
    let circle2d = projlib::project_circle_on_plane(&plane, &circle).unwrap();
    assert_eq!(circle2d.location(), Point2::new(1.0, 2.0));
    assert_eq!(circle2d.radius(), 1.5);
    assert_eq!(
        projlib::project_point_on_plane(&plane, center),
        Point2::new(1.0, 2.0)
    );
    // A line normal to the plane has no 2D direction (gp_Dir2d of a null vector).
    let normal = Line3::new(center, dir(0.0, 0.0, 1.0));
    assert_eq!(
        kind(projlib::project_line_on_plane(&plane, &normal)),
        "Standard_ConstructionError"
    );
}

#[test]
fn cylinder_projects_generatrices_and_parallels_only() {
    let cylinder = Cylinder3::new(xoy(), 2.0).unwrap();
    let generatrix = Line3::new(Point3::new(0.0, -2.0, 1.0), dir(0.0, 0.0, -1.0));
    let line: Line2 = projlib::project_line_on_cylinder(&cylinder, &generatrix).unwrap();
    assert!((line.location().x() - 1.5 * PI).abs() < 1e-15 && line.direction().y() == -1.0);
    let oblique = Line3::new(Point3::new(2.0, 0.0, 0.0), dir(0.0, 1.0, 1.0));
    assert_eq!(
        kind(projlib::project_line_on_cylinder(&cylinder, &oblique)),
        "Standard_NoSuchObject"
    );
    // The ellipse is left to the approximation: nothing is projected.
    let ellipse = Ellipse3::new(
        ax2(Point3::default(), dir(0.0, 1.0, 1.0), dir(1.0, 0.0, 0.0)),
        3.0,
        2.0,
    )
    .unwrap();
    let projection = Cylinder::with_ellipse(&cylinder, &ellipse);
    assert!(!projection.is_done());
    assert_eq!(projection.get_type(), CurveType::OtherCurve);
}

#[test]
fn cone_rejected_circle_keeps_the_line_type() {
    let cone = Cone3::new(xoy(), PI / 6.0, 5.0).unwrap();
    let tilted = Circle3::new(
        ax2(
            Point3::new(0.0, 0.0, 10.0),
            dir(1.0, 0.0, 0.0),
            dir(0.0, 1.0, 0.0),
        ),
        2.0,
    )
    .unwrap();
    let projection = Cone::with_circle(&cone, &tilted).unwrap();
    assert!(!projection.is_done());
    assert_eq!(projection.get_type(), CurveType::Line);
    // The static overload returns the default line instead of raising.
    assert_eq!(
        projlib::project_circle_on_cone(&cone, &tilted).unwrap(),
        Line2::default()
    );
}

#[test]
fn cone_line_through_the_apex_uses_the_next_point() {
    let cone = Cone3::new(xoy(), 0.5, 1.0).unwrap();
    let direction = dir(0.5_f64.sin(), 0.0, 0.5_f64.cos());
    let through_apex = Line3::new(cone.apex(), direction);
    let projection = Cone::with_line(&cone, &through_apex).unwrap();
    assert!(projection.is_done());
    let line = projection.line().unwrap();
    assert_eq!(line.direction().y(), 1.0);
    // The origin is moved back by the unit step taken along the line.
    let (u, v) =
        rustycad_tkmath::elslib::parameters(&cone, cone.apex().translated(direction.into()));
    assert_eq!(line.location(), Point2::new(u, v - 1.0));
}

#[test]
fn sphere_neither_iso_keeps_its_type_and_the_default_line() {
    let sphere = Sphere3::new(xoy(), 2.0).unwrap();
    let small = Circle3::new(
        ax2(
            Point3::new(0.5, 0.25, 3.0),
            dir(0.0, 1.0, 1.0),
            dir(1.0, 0.0, 0.0),
        ),
        1.5,
    )
    .unwrap();
    let projection = Sphere::with_circle(&sphere, &small).unwrap();
    assert!(!projection.is_done());
    assert_eq!(projection.get_type(), CurveType::OtherCurve);
    let mut undone = Sphere::with_sphere(&sphere);
    assert_eq!(kind(undone.set_in_bounds(0.0)), "StdFail_NotDone");
}

#[test]
fn sphere_meridian_is_moved_into_bounds() {
    let sphere = Sphere3::new(xoy(), 2.0).unwrap();
    let meridian = Circle3::new(
        ax2(Point3::default(), dir(0.0, 1.0, 0.0), dir(1.0, 0.0, 0.0)),
        2.0,
    )
    .unwrap();
    let mut projection = Sphere::with_circle(&sphere, &meridian).unwrap();
    assert!(projection.is_done());
    projection.set_in_bounds(2.5).unwrap();
    let point = rustycad_tkmath::elclib::value(2.5, &projection.line().unwrap());
    assert!(point.y().abs() <= PI / 2.0 + 1e-7 && (0.0..2.0 * PI).contains(&point.x()));
}

#[test]
fn torus_u_iso_with_the_centre_on_the_axis_raises() {
    let torus = Torus3::new(xoy(), 3.0, 1.0).unwrap();
    let on_axis = Circle3::new(
        ax2(
            Point3::new(0.0, 0.0, 2.0),
            dir(0.0, 1.0, 0.0),
            dir(1.0, 0.0, 0.0),
        ),
        1.0,
    )
    .unwrap();
    assert_eq!(
        kind(Torus::with_circle(&torus, &on_axis)),
        "gp_VectorWithNullMagnitude"
    );
    // A circle off the torus is still taken as an iso line.
    let off = Circle3::new(
        ax2(
            Point3::new(9.0, 0.0, 0.0),
            dir(0.0, 1.0, 0.0),
            dir(1.0, 0.0, 0.0),
        ),
        1.0,
    )
    .unwrap();
    assert!(Torus::with_circle(&torus, &off).unwrap().is_done());
}

#[test]
fn frames_translate_line_results_by_periods() {
    let cylinder = Cylinder3::new(xoy(), 2.0).unwrap();
    let generatrix = Line3::new(Point3::new(2.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let mut projection = Cylinder::with_line(&cylinder, &generatrix);
    projection.u_frame(0.0, 1.0, 5.0, 2.0 * PI);
    assert!((projection.line().unwrap().location().x() - 2.0 * PI).abs() < 1e-15);
    projection.v_frame(0.0, 1.0, 10.0, 1.0);
    assert_eq!(projection.line().unwrap().location().y(), 10.0);
}

#[test]
fn prj_func_rejects_a_bad_fixed_index_and_evaluates() {
    let line = GeomLine::from_point(Point3::new(0.0, 0.0, 5.0), dir(1.0, 0.0, 0.0));
    let curve = GeomAdaptorCurve::with_curve(&rustycad_tkernel::standard::Handle::new(line).cast())
        .unwrap();
    let sphere = GeomSphericalSurface::new(Axis3::default(), 2.0).unwrap();
    let surface =
        GeomAdaptorSurface::with_surface(&rustycad_tkernel::standard::Handle::new(sphere).cast())
            .unwrap();
    assert_eq!(
        kind(PrjFunc::new(&curve, 0.0, &surface, 4)),
        "Standard_ConstructionError"
    );
    let mut function = PrjFunc::new(&curve, 0.0, &surface, 1).unwrap();
    assert_eq!((function.nb_variables(), function.nb_equations()), (2, 2));
    let mut x = Vector::with_bounds(1, 2).unwrap();
    (x[1], x[2]) = (0.0, PI / 2.0);
    let mut f = Vector::with_bounds(1, 2).unwrap();
    let mut d = Matrix::with_bounds(1, 2, 1, 2).unwrap();
    assert!(function.values(&x, &mut f, &mut d).unwrap());
    assert_eq!(function.solution(), Point2::new(0.0, PI / 2.0));
    assert_eq!(
        kind(PrjResolve::new(&curve, &surface, 0)),
        "Standard_ConstructionError"
    );
    let mut resolve = PrjResolve::new(&curve, &surface, 1).unwrap();
    assert_eq!(kind(resolve.solution()), "StdFail_NotDone");
    let bounds = PrjResolveBox {
        tol: Point2::new(1e-9, 1e-9),
        inf: Point2::new(0.0, -PI / 2.0),
        sup: Point2::new(2.0 * PI, PI / 2.0),
    };
    resolve
        .perform(0.0, 0.2, 1.4, bounds, 1e-10, false)
        .unwrap();
    assert!(resolve.is_done());
    assert!((resolve.solution().unwrap().y() - PI / 2.0).abs() < precision::p_confusion());
}

#[test]
fn is_ana_surf_names_the_five_elementary_kinds() {
    assert!(projlib::is_ana_surf(&super::support::sphere(1.0)).unwrap());
    assert!(!projlib::is_ana_surf(&super::support::extrusion()).unwrap());
    assert!(!projlib::is_ana_surf(&super::support::flat_patch()).unwrap());
}

#[test]
fn plane_projector_is_periodic_for_closed_conics() {
    let plane = Plane3::from_position(xoy());
    let ellipse = Ellipse3::new(
        ax2(Point3::default(), dir(0.0, 0.0, 1.0), dir(1.0, 0.0, 0.0)),
        3.0,
        1.0,
    )
    .unwrap();
    let projection = Plane::with_ellipse(&plane, &ellipse).unwrap();
    assert!(projection.is_done() && projection.is_periodic());
    assert_eq!(projection.ellipse().unwrap().major_radius(), 3.0);
}
