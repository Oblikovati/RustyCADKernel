//! Behavior tests of BRepCheck_Solid and BRepCheck_Analyzer on cubes built with
//! BRep_Builder (brep_check_support); the probes compare the same checks with OCCT.
mod brep_check_support;

use brep_check_support::{Planar, cube, square};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Face, Shape, Shell, Solid};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tktopalgo::brep_check::{
    BRepCheckAnalyzer, BRepCheckResult, BRepCheckSolid, BRepCheckStatus,
};

use BRepCheckStatus::*;

/// A closed cube shell of `size` at `origin`, its faces reversed for a hole.
fn box_shell(origin: f64, size: f64, hole: bool) -> Shell {
    let points: Vec<(f64, f64, f64)> = brep_check_support::CUBE_POINTS
        .iter()
        .map(|&(x, y, z)| (origin + size * x, origin + size * y, origin + size * z))
        .collect();
    let mut planar = Planar::new(&points);
    for f in brep_check_support::CUBE_FACES {
        planar.face(&[&f], None, &[], 0.0);
    }
    planar.finish();
    let faces: Vec<Face> = planar
        .faces
        .iter()
        .map(|f| {
            if hole {
                Face::unchecked(f.reversed())
            } else {
                f.clone()
            }
        })
        .collect();
    let shell = planar.shell(&faces);
    shell.set_closed(true).unwrap();
    shell
}

fn solid(shells: &[Shell]) -> Solid {
    let b = BRepBuilder::default();
    let mut solid = Solid::default();
    b.make_solid(&mut solid);
    for s in shells {
        b.add(&mut solid, s).unwrap();
    }
    solid
}

fn statuses(checker: &BRepCheckSolid) -> Vec<BRepCheckStatus> {
    checker.status().unwrap().iter().copied().collect()
}

#[test]
fn solid_checks_its_shells() {
    assert_eq!(
        statuses(&BRepCheckSolid::new(&solid(&[box_shell(0.0, 1.0, false)])).unwrap()),
        [NoError]
    );
    let holed = solid(&[box_shell(0.0, 4.0, false), box_shell(1.0, 1.0, true)]);
    assert_eq!(statuses(&BRepCheckSolid::new(&holed).unwrap()), [NoError]);
    let apart = solid(&[box_shell(0.0, 1.0, false), box_shell(3.0, 1.0, false)]);
    assert_eq!(
        statuses(&BRepCheckSolid::new(&apart).unwrap()),
        [EnclosedRegion, SubshapeNotInShape]
    );
    let nested = solid(&[box_shell(0.0, 4.0, false), box_shell(1.0, 1.0, false)]);
    assert_eq!(
        statuses(&BRepCheckSolid::new(&nested).unwrap()),
        [EnclosedRegion]
    );
    let checker = BRepCheckSolid::new(&holed).unwrap();
    // InContext checks nothing: the solid has no context.
    let shell: Shape = box_shell(0.0, 1.0, false).into_shape();
    checker.in_context(&shell).unwrap();
    checker.init_context_iterator().unwrap();
    assert!(!checker.more_shape_in_context() && !checker.is_status_on_shape(&shell));
}

#[test]
fn solid_with_a_detached_vertex_or_a_shared_face() {
    let shell = box_shell(0.0, 1.0, false);
    let v = brep_check_support::vertex(rustycad_tkmath::gp::Point3::new(5.0, 5.0, 5.0), 1e-7);
    let b = BRepBuilder::default();
    let mut s = solid(std::slice::from_ref(&shell));
    b.add(&mut s, &v).unwrap();
    assert_eq!(
        statuses(&BRepCheckSolid::new(&s).unwrap()),
        [BadOrientationOfSubshape]
    );
    let shared = solid(&[shell.clone(), shell]);
    assert_eq!(
        statuses(&BRepCheckSolid::new(&shared).unwrap()).first(),
        Some(&InvalidImbricationOfShells)
    );
}

#[test]
fn analyzer_validates_solids_faces_and_compounds() {
    let valid = solid(&[box_shell(0.0, 1.0, false)]);
    let analyzer = BRepCheckAnalyzer::new(&valid, true, false, false).unwrap();
    assert!(analyzer.is_valid().unwrap());
    assert!(!analyzer.is_parallel() && !analyzer.is_exact_method());
    let open = cube(&[0, 1, 2, 3, 4]);
    let open_solid = solid(&[open.shell(&open.faces)]);
    assert!(
        !BRepCheckAnalyzer::new(&open_solid, true, true, true)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let shifted = square(1.0, &[], &[(0, 1)], 1e-3);
    let analyzer = BRepCheckAnalyzer::new(&shifted.faces[0], true, false, false).unwrap();
    assert!(!analyzer.is_valid().unwrap());
    // Without the geometric controls the shifted pcurve is not checked.
    assert!(
        BRepCheckAnalyzer::new(&shifted.faces[0], false, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let b = BRepBuilder::default();
    let mut compound = Compound::default();
    b.make_compound(&mut compound);
    b.add(&mut compound, &valid).unwrap();
    b.add(
        &mut compound,
        &shifted.faces[0].oriented(Orientation::Reversed),
    )
    .unwrap();
    let mut analyzer = BRepCheckAnalyzer::new(&compound, true, false, false).unwrap();
    assert!(!analyzer.is_valid().unwrap());
    assert!(analyzer.is_valid_shape(&valid).unwrap());
    assert!(analyzer.result(&compound).unwrap().is_null());
    assert!(analyzer.result(&valid).unwrap().is_some());
    let foreign: Shape = square(1.0, &[], &[], 0.0).faces[0].clone().into_shape();
    assert!(analyzer.result(&foreign).is_err());
    analyzer.set_parallel(true);
    analyzer.set_exact_method(true);
    assert!(analyzer.is_parallel() && analyzer.is_exact_method());
    assert_eq!(
        BRepCheckAnalyzer::new(&Shape::default(), true, false, false)
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );
}
