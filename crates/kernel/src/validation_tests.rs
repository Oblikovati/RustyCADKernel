// SPDX-License-Identifier: GPL-2.0-only
//! `validate_body` through the facade: valid primitives with their Euler genus, and one
//! named finding in the right stage for each deliberately broken body.
use crate::invalid_shapes::{
    flipped_face, non_manifold, open_sheet, open_shell, over_tolerance_edge, over_tolerance_vertex,
    overlapping_boxes,
};
use crate::test_shapes::{annulus_prism, box_solid, cone, cylinder, sphere, torus};
use crate::validation::{
    Finding, FindingKind, Severity, StageVerdict, ValidationReport, ValidationStage, validate_body,
};
use StageVerdict::{Failed, NotRun, Passed};
use rustycad_tkbo::profile_extrusion::{DisplayTolerance, Resolution, ToleranceCap};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;

fn resolution(budget: f64) -> Resolution {
    let display = DisplayTolerance {
        chordal: 1e-3,
        angular: 0.1,
    };
    Resolution::new(1e-6, 1e-7, 1e-9, display, ToleranceCap::Report(budget))
}

fn validate(shape: &Shape) -> ValidationReport {
    validate_body(shape, &resolution(1e-3)).unwrap()
}

fn verdicts(report: &ValidationReport) -> [StageVerdict; 4] {
    report.stages.each_ref().map(|stage| stage.verdict)
}

fn defects(report: &ValidationReport, stage: ValidationStage) -> Vec<Finding> {
    let findings = report.stage(stage).findings.iter().cloned();
    findings
        .filter(|finding| finding.severity == Severity::Defect)
        .collect()
}

/// The edges of `face`, for checking that findings name them.
fn edges_of(face: &Shape) -> Vec<Shape> {
    let mut edges = Vec::new();
    let mut explorer = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        edges.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    edges
}

/// Asserts the four topology defects of `kind` each name an edge of `face`.
fn assert_edges_named(root: &Shape, face: &Shape, findings: &[Finding], kind: &FindingKind) {
    assert_eq!(findings.len(), 4, "{findings:?}");
    let edges = edges_of(face);
    for finding in findings {
        assert_eq!(&finding.kind, kind);
        let named = finding.subject.resolve(root).unwrap();
        assert!(edges.iter().any(|edge| edge.is_same(&named)), "{finding:?}");
    }
}

#[test]
fn primitives_pass_every_stage_with_their_genus() {
    let cases = [
        (box_solid(1.0, 2.0, 3.0), 0),
        (cylinder(1.0, 2.0), 0),
        (sphere(1.5), 0),
        (cone(2.0, 0.5, 3.0), 0),
        (torus(3.0, 1.0), 1),
        (annulus_prism(2.0, 0.75, 1.5), 1),
    ];
    for (index, (shape, genus)) in cases.iter().enumerate() {
        let report = validate(shape);
        assert_eq!(verdicts(&report), [Passed; 4], "case {index}: {report:#?}");
        assert_eq!(report.census.len(), 1);
        assert_eq!(report.census[0].genus, Some(*genus), "case {index}");
    }
}

#[test]
fn box_census_is_the_cube_euler_characteristic() {
    let census = validate(&box_solid(1.0, 1.0, 1.0)).census[0];
    let counts = (
        census.vertices,
        census.edges,
        census.faces,
        census.loops,
        census.shells,
    );
    assert_eq!(counts, (8, 12, 6, 6, 1));
}

#[test]
fn open_shell_fails_topology_with_free_edges() {
    let (solid, missing) = open_shell();
    let report = validate(&solid);
    assert_eq!(verdicts(&report), [Failed, NotRun, NotRun, NotRun]);
    let free: Vec<Finding> = defects(&report, ValidationStage::Topology)
        .into_iter()
        .filter(|finding| finding.kind == FindingKind::FreeEdge)
        .collect();
    assert_edges_named(&solid, &missing, &free, &FindingKind::FreeEdge);
    assert!(report.census.is_empty());
}

#[test]
fn an_open_sheet_keeps_its_boundary_edges() {
    let report = validate(&open_sheet());
    assert_eq!(verdicts(&report), [Passed; 4], "{report:#?}");
    assert!(report.census.is_empty());
}

#[test]
fn repeated_face_fails_topology_with_non_manifold_edges() {
    let (solid, repeated) = non_manifold();
    let report = validate(&solid);
    assert_eq!(verdicts(&report)[1..], [NotRun; 3]);
    let kind = FindingKind::NonManifoldEdge { uses: 3 };
    let found: Vec<Finding> = defects(&report, ValidationStage::Topology)
        .into_iter()
        .filter(|finding| finding.kind == kind)
        .collect();
    assert_edges_named(&solid, &repeated, &found, &kind);
}

#[test]
fn flipped_face_fails_topology_with_inconsistent_orientation() {
    let (solid, flipped) = flipped_face();
    let report = validate(&solid);
    assert_eq!(verdicts(&report)[1..], [NotRun; 3]);
    let kind = FindingKind::InconsistentOrientation;
    let found: Vec<Finding> = defects(&report, ValidationStage::Topology)
        .into_iter()
        .filter(|finding| finding.kind == kind)
        .collect();
    assert_edges_named(&solid, &flipped, &found, &kind);
}

#[test]
fn overlapping_solids_fail_self_intersection_with_named_interferences() {
    let compound = overlapping_boxes();
    let report = validate(&compound);
    assert_eq!(
        verdicts(&report),
        [Passed, Passed, Failed, NotRun],
        "{report:#?}"
    );
    let found = defects(&report, ValidationStage::SelfIntersection);
    assert!(!found.is_empty());
    for finding in &found {
        assert_eq!(finding.kind, FindingKind::Interference);
        assert!(finding.subject.resolve(&compound).is_some(), "{finding:?}");
        assert!(
            finding.context.unwrap().resolve(&compound).is_some(),
            "{finding:?}"
        );
    }
}

#[test]
fn over_tolerance_vertex_fails_the_tolerance_stage_only() {
    let (solid, vertex) = over_tolerance_vertex(0.5);
    let report = validate(&solid);
    assert_eq!(verdicts(&report), [Passed, Passed, Passed, Failed]);
    let found = defects(&report, ValidationStage::Tolerance);
    assert_eq!(found.len(), 1, "{found:?}");
    let kind = FindingKind::ToleranceAboveBudget {
        tolerance: 0.5,
        budget: 1e-3,
    };
    assert_eq!(found[0].kind, kind);
    assert!(found[0].subject.resolve(&solid).unwrap().is_same(&vertex));
}

#[test]
fn edge_above_its_vertices_breaks_tolerance_nesting() {
    let (solid, edge) = over_tolerance_edge(1e-4);
    let report = validate(&solid);
    assert_eq!(verdicts(&report)[..3], [Passed; 3], "{report:#?}");
    let found = defects(&report, ValidationStage::Tolerance);
    assert_eq!(found.len(), 2, "{found:?}");
    for finding in &found {
        assert!(matches!(finding.kind, FindingKind::ToleranceOrder { inner, .. } if inner == 1e-4));
        assert!(finding.subject.resolve(&solid).unwrap().is_same(&edge));
        assert_eq!(finding.context.unwrap().kind, ShapeEnum::Vertex);
    }
}

#[test]
fn reports_are_identical_across_threads() {
    let compute = || [validate(&torus(3.0, 1.0)), validate(&open_shell().0)];
    let reference = compute();
    let handles: Vec<_> = (0..4).map(|_| std::thread::spawn(compute)).collect();
    for handle in handles {
        assert_eq!(handle.join().unwrap(), reference);
    }
}
