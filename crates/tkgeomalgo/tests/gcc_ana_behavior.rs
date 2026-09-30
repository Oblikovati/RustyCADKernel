//! Behavior of the GccAna solvers beyond the upstream suite (the bits of every
//! solver are compared with OCCT by the numeric probe).
use rustycad_tkgeomalgo::gcc_ana::{
    Circ2d2TanOn, Circ2d2TanRad, Circ2d3Tan, Circ2dBisec, Circ2dTanCen, Circ2dTanOnRad,
    CircLin2dBisec, CircPnt2dBisec, Lin2d2Tan, Lin2dBisec, Lin2dTanObl, LinPnt2dBisec, Pnt2dBisec,
};
use rustycad_tkgeomalgo::gcc_ent::{self, Position, QualifiedCirc};
use rustycad_tkgeomalgo::gcc_int::IType;
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Line2, Point2};

const TOL: f64 = 1.0e-7;

fn circle(x: f64, y: f64, r: f64) -> Circle2 {
    let axis = Axis2d::new(Point2::new(x, y), Direction2::new(1.0, 0.0).unwrap());
    Circle2::from_x_axis(axis, r, true).unwrap()
}

fn line(x: f64, y: f64, dx: f64, dy: f64) -> Line2 {
    Line2::new(Point2::new(x, y), Direction2::new(dx, dy).unwrap())
}

fn unqualified(c: Circle2) -> QualifiedCirc {
    gcc_ent::unqualified_circ(c)
}

#[test]
fn two_apart_circles_have_four_common_tangents() {
    let lin = Lin2d2Tan::tangent_to_circles(
        unqualified(circle(0.0, 0.0, 1.0)),
        unqualified(circle(5.0, 0.0, 1.0)),
        TOL,
    )
    .unwrap();
    assert_eq!(lin.nb_solutions(), 4);
    for i in 1..=4 {
        let (_, _, p1) = lin.tangency1(i).unwrap();
        assert!((p1.distance(Point2::new(0.0, 0.0)) - 1.0).abs() < 1e-12);
    }
    assert_eq!(
        lin.this_solution(5).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn enclosed_circles_are_rejected_by_line_solvers() {
    let enclosed = gcc_ent::enclosed_circ(circle(0.0, 0.0, 1.0));
    let error = Lin2d2Tan::tangent_through_point(enclosed, Point2::new(3.0, 0.0), TOL).unwrap_err();
    assert_eq!(error.exception_type(), "GccEnt_BadQualifier");
    let error =
        Lin2dTanObl::tangent_to_circle(enclosed, line(0.0, 0.0, 1.0, 0.0), 0.3).unwrap_err();
    assert_eq!(error.exception_type(), "GccEnt_BadQualifier");
}

#[test]
fn radius_solvers_refuse_negative_radii() {
    let c = unqualified(circle(0.0, 0.0, 1.0));
    let error = Circ2d2TanRad::circ_circ(c, c, -1.0, TOL).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NegativeValue");
    let error = Circ2dTanOnRad::circ_on_line(c, line(0.0, 0.0, 1.0, 0.0), -1.0, TOL).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NegativeValue");
}

#[test]
fn circles_of_given_radius_through_two_points() {
    let circ =
        Circ2d2TanRad::pnt_pnt(Point2::new(-1.0, 0.0), Point2::new(1.0, 0.0), 2.0, TOL).unwrap();
    assert_eq!(circ.nb_solutions(), 2);
    for i in 1..=2 {
        let solution = circ.this_solution(i).unwrap();
        assert_eq!(solution.radius(), 2.0);
        assert_eq!(
            circ.which_qualifier(i).unwrap(),
            (Position::NoQualifier, Position::NoQualifier)
        );
    }
}

#[test]
fn centred_circles_and_their_tangency() {
    let circ = Circ2dTanCen::tangent_to_circle(
        unqualified(circle(0.0, 0.0, 1.0)),
        Point2::new(3.0, 0.0),
        TOL,
    )
    .unwrap();
    let radii: Vec<f64> = (1..=circ.nb_solutions())
        .map(|i| circ.this_solution(i).unwrap().radius())
        .collect();
    assert_eq!(radii, [2.0, 4.0]);
    let concentric =
        Circ2dTanCen::tangent_to_circle(unqualified(circle(0.0, 0.0, 1.0)), Point2::default(), TOL)
            .unwrap();
    assert!(concentric.is_the_same1(1).unwrap());
    assert_eq!(
        concentric.tangency1(1).unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}

#[test]
fn circles_on_a_locus_report_their_centre_parameter() {
    let on = Circ2d2TanOn::pnt_pnt_on_line(
        Point2::new(-1.0, 1.0),
        Point2::new(1.0, 1.0),
        line(0.0, 0.0, 1.0, 0.0),
        TOL,
    )
    .unwrap();
    assert_eq!(on.nb_solutions(), 1);
    let (parameter, _) = on.center_on3(1).unwrap();
    assert_eq!(parameter, 0.0);
    // ThisSolution reads the array without testing NbSolutions, as natively.
    assert_eq!(on.this_solution(2).unwrap().radius(), f64::MAX);
    let rad =
        Circ2dTanOnRad::pnt_on_line(Point2::new(0.0, 1.0), line(0.0, 0.0, 1.0, 0.0), 1.0, TOL)
            .unwrap();
    assert_eq!(
        rad.this_solution(1).unwrap().location(),
        Point2::new(0.0, 0.0)
    );
}

#[test]
fn three_tangent_circles_include_the_inscribed_one() {
    let circ = Circ2d3Tan::lin_lin_lin(
        gcc_ent::unqualified_lin(line(0.0, 0.0, 1.0, 0.0)),
        gcc_ent::unqualified_lin(line(0.0, 0.0, 0.0, 1.0)),
        gcc_ent::unqualified_lin(line(4.0, 0.0, -1.0, 1.0)),
        TOL,
    )
    .unwrap();
    assert!(circ.nb_solutions() >= 1);
    assert!(circ.this_solution(1).unwrap().radius() > 0.0);
}

#[test]
fn bisectors_of_points_lines_and_circles() {
    assert!(
        !Pnt2dBisec::new(Point2::default(), Point2::default())
            .unwrap()
            .has_solution()
    );
    let lines = Lin2dBisec::new(line(0.0, 0.0, 1.0, 0.0), line(0.0, 0.0, 0.0, 1.0)).unwrap();
    assert_eq!(lines.nb_solutions().unwrap(), 2);
    let parabola = LinPnt2dBisec::new(line(0.0, 0.0, 1.0, 0.0), Point2::new(0.0, 2.0)).unwrap();
    assert_eq!(parabola.this_solution().unwrap().arc_type(), IType::Par);
    let circ_line = CircLin2dBisec::new(circle(0.0, 3.0, 1.0), line(0.0, 0.0, 1.0, 0.0));
    assert_eq!(circ_line.this_solution(1).unwrap().arc_type(), IType::Par);
    let outside = CircPnt2dBisec::new(circle(0.0, 0.0, 1.0), Point2::new(3.0, 0.0));
    assert_eq!(outside.nb_solutions(), 2);
    assert_eq!(outside.this_solution(2).unwrap().arc_type(), IType::Hpr);
    let apart = Circ2dBisec::new(circle(0.0, 0.0, 1.0), circle(5.0, 0.0, 2.0));
    assert_eq!(apart.nb_solutions().unwrap(), 4);
}
