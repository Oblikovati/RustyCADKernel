//! Geom2d_Conic members and the specific members of each conic.
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dConic, Geom2dCurve, Geom2dEllipse, Geom2dGeometry, Geom2dHyperbola,
    Geom2dParabola,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{
    Axis2d, Axis22d, Circle2, Direction2, Ellipse2, Hyperbola2, Parabola2, Point2, Transform2,
};
use std::f64::consts::PI;

fn origin_axis() -> Axis2d {
    Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap())
}

fn frame() -> Axis22d {
    Axis22d::new(
        Point2::new(1.0, 2.0),
        Direction2::new(0.0, 1.0).unwrap(),
        true,
    )
    .unwrap()
}

fn near(a: Point2, x: f64, y: f64) {
    EXPECT_NEAR(a.x(), x, 1e-14);
    EXPECT_NEAR(a.y(), y, 1e-14);
}

#[test]
fn conic_placement_members() {
    let c: Handle<dyn Geom2dConic> = Handle::new(Geom2dCircle::new(frame(), 2.0).unwrap()).cast();
    assert_eq!(c.location(), Point2::new(1.0, 2.0));
    assert_eq!(c.x_axis().direction(), Direction2::new(0.0, 1.0).unwrap());
    // A direct frame turns X counterclockwise: Y = (-1, 0).
    near(c.y_axis().direction().xy().into(), -1.0, 0.0);
    near(c.value(0.0).unwrap(), 1.0, 4.0);
    c.set_location(Point2::new(0.0, 0.0));
    c.set_x_axis(origin_axis()).unwrap();
    near(c.value(PI / 2.0).unwrap(), 0.0, 2.0);
    c.reverse().unwrap();
    // The reversed frame is indirect: the point of parameter PI/2 is now below the X axis.
    near(c.value(PI / 2.0).unwrap(), 0.0, -2.0);
    c.set_y_axis(Axis2d::new(
        Point2::new(0.0, 0.0),
        Direction2::new(0.0, 1.0).unwrap(),
    ))
    .unwrap();
    near(c.position().y_direction().xy().into(), 0.0, 1.0);
    c.set_axis(frame());
    assert_eq!(c.position(), frame());
    assert_eq!((c.continuity(), c.is_cn(9).unwrap()), (Shape::CN, true));
}

#[test]
fn circle_members() {
    let circle = Geom2dCircle::from_circ2d(Circle2::new(frame(), 3.0).unwrap());
    assert_eq!(
        (circle.radius(), circle.eccentricity().unwrap()),
        (3.0, 0.0)
    );
    assert_eq!(
        circle.set_radius(-1.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert!(Geom2dCircle::new(frame(), -1.0).is_err());
    assert!(Geom2dCircle::from_axis(origin_axis(), -1.0, true).is_err());
    circle.set_radius(1.5).unwrap();
    let gp = circle.circ2d().unwrap();
    assert_eq!((gp.radius(), gp.axis()), (1.5, frame()));
    circle.set_circ2d(Circle2::new(frame(), 4.0).unwrap());
    assert_eq!(circle.radius(), 4.0);
    assert_eq!(
        (
            circle.reversed_parameter(1.0).unwrap(),
            circle.period().unwrap()
        ),
        (2.0 * PI - 1.0, 2.0 * PI)
    );
    assert!(circle.is_closed().unwrap() && circle.is_periodic());
    assert_eq!(
        circle.eval_dn(0.0, 0).unwrap_err().exception_type(),
        "Geom2d_UndefinedDerivative"
    );
    EXPECT_NEAR(circle.eval_dn(0.3, 5).unwrap().magnitude(), 4.0, 1e-14);
    circle
        .transform(Transform2::scaling(Point2::new(1.0, 2.0), -0.5))
        .unwrap();
    assert_eq!(circle.radius(), 2.0);
}

#[test]
fn ellipse_members() {
    let e = Geom2dEllipse::from_axis(origin_axis(), 5.0, 3.0, true).unwrap();
    assert_eq!((e.major_radius(), e.minor_radius()), (5.0, 3.0));
    assert_eq!(
        (e.focal(), e.eccentricity().unwrap(), e.parameter()),
        (8.0, 0.8, 9.0 / 5.0)
    );
    assert_eq!(
        (e.focus1(), e.focus2()),
        (Point2::new(4.0, 0.0), Point2::new(-4.0, 0.0))
    );
    // The directrices are at a / e = 6.25 from the center.
    near(e.directrix1().unwrap().location(), 6.25, 0.0);
    near(e.directrix2().unwrap().location(), -6.25, 0.0);
    assert_eq!(
        e.set_major_radius(2.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        e.set_minor_radius(6.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert!(e.set_minor_radius(-1.0).is_err());
    e.set_major_radius(6.0).unwrap();
    e.set_minor_radius(1.0).unwrap();
    let gp = e.elips2d().unwrap();
    assert_eq!((gp.major_radius(), gp.minor_radius()), (6.0, 1.0));
    e.set_elips2d(Ellipse2::new(frame(), 2.0, 2.0).unwrap());
    assert_eq!(e.eccentricity().unwrap(), 0.0);
    assert!(Geom2dEllipse::new(frame(), 1.0, 2.0).is_err());
    let flat = Geom2dEllipse::from_elips2d(Ellipse2::new(frame(), 0.0, 0.0).unwrap());
    assert_eq!((flat.eccentricity().unwrap(), flat.parameter()), (0.0, 0.0));
    let d3 = e.eval_d3(0.0).unwrap();
    near(d3.point, 1.0, 4.0);
    assert_eq!(e.eval_dn(0.0, 3).unwrap(), d3.d3);
    assert_eq!(
        e.eval_dn(0.0, -1).unwrap_err().exception_type(),
        "Geom2d_UndefinedDerivative"
    );
    assert_eq!(
        (e.reversed_parameter(0.5).unwrap(), e.last_parameter()),
        (2.0 * PI - 0.5, 2.0 * PI)
    );
}

#[test]
fn hyperbola_members() {
    let h = Geom2dHyperbola::from_axis(origin_axis(), 3.0, 4.0, true).unwrap();
    assert_eq!(
        (h.focal(), h.eccentricity().unwrap(), h.parameter().unwrap()),
        (10.0, 5.0 / 3.0, 16.0 / 3.0)
    );
    assert_eq!(
        (h.focus1(), h.focus2()),
        (Point2::new(5.0, 0.0), Point2::new(-5.0, 0.0))
    );
    // Native gp_Hypr2d::Asymptote1/2 build Y + Y * (+-b/a), so both directions lie on the
    // Y axis: Y * (1 + 4/3) and Y * (1 - 4/3). The directrices sit at a / e = 1.8.
    near(h.asymptote1().unwrap().direction().xy().into(), 0.0, 1.0);
    near(h.asymptote2().unwrap().direction().xy().into(), 0.0, -1.0);
    near(h.directrix1().unwrap().location(), 1.8, 0.0);
    // Native gp_Hypr2d::Directrix2 is Focus1 + X * (p / e) = 5 + 3.2.
    near(h.directrix2().unwrap().location(), 8.2, 0.0);
    assert_eq!(h.conjugate_branch1().unwrap().major_radius(), 4.0);
    assert_eq!(h.conjugate_branch2().unwrap().minor_radius(), 3.0);
    near(
        h.other_branch().unwrap().axis().x_direction().xy().into(),
        -1.0,
        0.0,
    );
    assert_eq!(
        (h.first_parameter(), h.is_closed().unwrap(), h.is_periodic()),
        (-2e100, false, false)
    );
    assert_eq!(h.reversed_parameter(2.0).unwrap(), -2.0);
    // At u = 0 the main branch is at its vertex (a, 0).
    near(h.value(0.0).unwrap(), 3.0, 0.0);
    assert_eq!(
        h.set_major_radius(-1.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert!(h.set_minor_radius(-1.0).is_err());
    h.set_hypr2d(Hyperbola2::new(frame(), 0.0, 1.0).unwrap());
    assert_eq!(
        h.eccentricity().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    assert_eq!(
        h.parameter().unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    h.set_major_radius(2.0).unwrap();
    h.set_minor_radius(2.0).unwrap();
    assert_eq!(h.hypr2d().unwrap().major_radius(), 2.0);
    assert!(Geom2dHyperbola::new(frame(), -1.0, 1.0).is_err());
    let g = Geom2dHyperbola::from_hypr2d(Hyperbola2::new(frame(), 1.0, 1.0).unwrap());
    assert_eq!(g.eval_dn(0.5, 2).unwrap(), g.eval_d2(0.5).unwrap().d2);
    assert_eq!(
        g.eval_dn(0.5, 0).unwrap_err().exception_type(),
        "Geom2d_UndefinedDerivative"
    );
    assert_eq!(g.eval_d3(0.5).unwrap().d3, g.eval_dn(0.5, 3).unwrap());
}

#[test]
fn parabola_members() {
    let p = Geom2dParabola::from_axis(origin_axis(), 2.0, true).unwrap();
    assert_eq!(
        (p.focal(), p.parameter(), p.eccentricity().unwrap()),
        (2.0, 4.0, 1.0)
    );
    assert_eq!(p.focus(), Point2::new(2.0, 0.0));
    near(p.directrix().unwrap().location(), -2.0, 0.0);
    assert_eq!(
        p.set_focal(-1.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    p.set_focal(1.0).unwrap();
    assert_eq!(p.parab2d().unwrap().focal(), 1.0);
    p.set_parab2d(Parabola2::new(frame(), 3.0).unwrap());
    assert_eq!(p.focal(), 3.0);
    let d = Geom2dParabola::from_directrix(origin_axis(), Point2::new(0.0, 4.0)).unwrap();
    assert_eq!((d.focal(), d.location()), (2.0, Point2::new(0.0, 2.0)));
    assert!(Geom2dParabola::new(frame(), -1.0).is_err());
    let scale = Transform2::scaling(Point2::new(0.0, 0.0), -2.0);
    assert_eq!(
        (
            d.transformed_parameter(1.5, scale),
            d.parametric_transformation(scale)
        ),
        (3.0, 2.0)
    );
    assert_eq!(d.transformed_parameter(-2e100, scale), -2e100);
    // D3 of a parabola is null; the point of parameter u = 2 f t sits at (t^2 f... ) = (u^2/(4f), u).
    let d3 = d.eval_d3(2.0).unwrap();
    assert_eq!((d3.d3.x(), d3.d3.y()), (0.0, 0.0));
    near(d.value(0.0).unwrap(), 0.0, 2.0);
    assert_eq!(
        d.eval_dn(1.0, 0).unwrap_err().exception_type(),
        "Geom2d_UndefinedDerivative"
    );
    assert_eq!(d.eval_dn(1.0, 2).unwrap(), d.eval_d2(1.0).unwrap().d2);
    d.transform(scale).unwrap();
    assert_eq!(d.focal(), 4.0);
    assert_eq!(d.reversed_parameter(1.0).unwrap(), -1.0);
}

#[test]
fn copies_are_independent() {
    let e = Handle::new(Geom2dEllipse::from_axis(origin_axis(), 5.0, 3.0, true).unwrap());
    let copy: Handle<Geom2dEllipse> = e.copy().unwrap().cast();
    copy.set_major_radius(7.0).unwrap();
    assert_eq!((e.major_radius(), copy.major_radius()), (5.0, 7.0));
    let h = Handle::new(Geom2dHyperbola::from_axis(origin_axis(), 1.0, 2.0, false).unwrap());
    assert_eq!(
        h.copy().unwrap().cast::<Geom2dHyperbola>().minor_radius(),
        2.0
    );
    let p = Handle::new(Geom2dParabola::from_axis(origin_axis(), 1.0, false).unwrap());
    assert_eq!(p.copy().unwrap().cast::<Geom2dParabola>().focal(), 1.0);
}
