//! Behavior of the GccEnt and GccInt ports (no upstream suite; the bits are compared
//! with OCCT by the numeric probe).
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkgeomalgo::gcc_ent::{self, Position, QualifiedCirc, QualifiedLin};
use rustycad_tkgeomalgo::gcc_int::{BCirc, BElips, BHyper, BLine, BParab, Bisec, IType};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};

#[test]
fn position_names_round_trip_case_insensitively() {
    for position in Position::ALL {
        let name = gcc_ent::position_to_string(position);
        assert_eq!(
            gcc_ent::try_position_from_string(&name.to_lowercase()),
            Some(position)
        );
    }
    assert_eq!(gcc_ent::try_position_from_string("inside"), None);
    assert_eq!(
        gcc_ent::position_from_string("inside"),
        Position::Unqualified
    );
    let mut printed: Vec<u8> = Vec::new();
    gcc_ent::print(Position::Enclosed, &mut printed).unwrap();
    assert_eq!(printed, b"ENCLOSED");
    assert_eq!(
        gcc_ent::bad_qualifier("q").exception_type(),
        "GccEnt_BadQualifier"
    );
}

#[test]
fn qualified_entities_report_their_qualifier() {
    let circle = Circle2::default();
    let checks: [(QualifiedCirc, [bool; 4]); 4] = [
        (
            gcc_ent::unqualified_circ(circle),
            [true, false, false, false],
        ),
        (gcc_ent::enclosing(circle), [false, true, false, false]),
        (gcc_ent::enclosed_circ(circle), [false, false, true, false]),
        (gcc_ent::outside_circ(circle), [false, false, false, true]),
    ];
    for (qualified, flags) in checks {
        let read = [
            qualified.is_unqualified(),
            qualified.is_enclosing(),
            qualified.is_enclosed(),
            qualified.is_outside(),
        ];
        assert_eq!(read, flags);
    }
    let line = Line2::default();
    let lines: [QualifiedLin; 3] = [
        gcc_ent::unqualified_lin(line),
        gcc_ent::enclosed_lin(line),
        gcc_ent::outside_lin(line),
    ];
    let qualifiers: Vec<Position> = lines.iter().map(QualifiedLin::qualifier).collect();
    assert_eq!(
        qualifiers,
        [Position::Unqualified, Position::Enclosed, Position::Outside]
    );
    assert_eq!(lines[0].qualified(), line);
}

#[test]
fn each_bisector_answers_only_its_own_accessor() {
    let bisectors: [Handle<dyn Bisec>; 5] = [
        Handle::new(BLine::new(Line2::default())).cast(),
        Handle::new(BCirc::new(Circle2::default())).cast(),
        Handle::new(BElips::new(Ellipse2::default())).cast(),
        Handle::new(BHyper::new(Hyperbola2::default())).cast(),
        Handle::new(BParab::new(Parabola2::default())).cast(),
    ];
    let kinds = [IType::Lin, IType::Cir, IType::Ell, IType::Hpr, IType::Par];
    for (bisector, kind) in bisectors.iter().zip(kinds) {
        assert_eq!(bisector.arc_type(), kind);
        assert!(bisector.is_kind(<dyn Bisec as RuntimeTyped>::static_type()));
        let answers = [
            bisector.line().is_ok(),
            bisector.circle().is_ok(),
            bisector.ellipse().is_ok(),
            bisector.hyperbola().is_ok(),
            bisector.parabola().is_ok(),
        ];
        assert_eq!(answers.iter().filter(|&&ok| ok).count(), 1);
        let error = bisector.point().unwrap_err();
        assert_eq!(error.exception_type(), "Standard_NotImplemented");
    }
}
