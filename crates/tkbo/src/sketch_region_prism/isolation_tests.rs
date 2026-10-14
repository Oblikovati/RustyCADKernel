// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region::equal_occurrences;
use crate::sketch_region_candidates::tests::candidates;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face, Shape, edge, face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dBezierCurve;
use rustycad_tkg3d::{geom::GeomBezierCurve, top_abs::ShapeEnum};
use rustycad_tkmath::{
    gp::{Direction3, Point2, Point3},
    poly::{Triangle, Triangulation, mesh_purpose},
};
pub(super) fn attached_mesh(face: &Face) -> Handle<Triangulation> {
    let mesh = Handle::new(Triangulation::with_sizes(3, 1, true, true).unwrap());
    fill_mesh(mesh.get().unwrap());
    BRepBuilder::default()
        .update_face_triangulation(face, &mesh, true)
        .unwrap();
    mesh
}
fn mesh(face: &Face) -> Handle<Triangulation> {
    let (mesh, _) = BRepTool::triangulation(face, mesh_purpose::NONE).unwrap();
    assert!(!mesh.is_null());
    mesh
}
fn mesh_cap(body: &Shape) -> Face {
    equal_occurrences(body)
        .unwrap()
        .iter()
        .filter(|s| s.shape_type().unwrap() == ShapeEnum::Face)
        .map(|s| face(s).unwrap())
        .find(|f| {
            !BRepTool::triangulation(f, mesh_purpose::NONE)
                .unwrap()
                .0
                .is_null()
        })
        .expect("copied base triangulation must survive on actual first cap")
}
pub(super) fn first_edge(face: &Face) -> Edge {
    let found = equal_occurrences(face)
        .unwrap()
        .into_iter()
        .find(|s| s.shape_type().unwrap() == ShapeEnum::Edge)
        .unwrap();
    edge(&found).unwrap()
}
pub(super) fn pole(edge: &Edge) -> Point3 {
    let (curve, _, _) = BRepTool::curve(edge).unwrap();
    curve
        .cast::<GeomBezierCurve>()
        .get()
        .unwrap()
        .pole(2)
        .unwrap()
}
pub(super) fn set_pole(edge: &Edge, coordinate: f64) {
    let (curve, _, _) = BRepTool::curve(edge).unwrap();
    curve
        .cast::<GeomBezierCurve>()
        .get()
        .unwrap()
        .set_pole(2, Point3::new(coordinate, 0., 0.))
        .unwrap();
}
fn change_mesh(mesh: &Handle<Triangulation>, coordinate: f64) {
    let mesh = mesh.get().unwrap();
    mesh.set_node(2, Point3::new(coordinate, 0., 0.)).unwrap();
    mesh.set_uv_node(2, Point2::new(coordinate, 2.)).unwrap();
    mesh.set_normal(2, Direction3::new(1., 0., 0.).unwrap())
        .unwrap();
    mesh.set_triangle(1, Triangle::new(3, 2, 1)).unwrap();
    mesh.set_deflection(coordinate);
    mesh.unset_cached_min_max();
}
pub(super) fn assert_original_mesh(mesh: &Handle<Triangulation>) {
    let mesh = mesh.get().unwrap();
    assert_eq!(mesh.node(2).unwrap(), Point3::new(1., 0., 0.));
    assert_eq!(mesh.uv_node(2).unwrap(), Point2::new(1., 0.));
    assert_eq!(
        mesh.normal(2).unwrap(),
        Direction3::new(0., 0., 1.).unwrap()
    );
    assert_eq!(mesh.triangle(1).unwrap(), Triangle::new(1, 2, 3));
    assert_eq!(mesh.deflection(), 0.);
    assert!(mesh.has_cached_min_max());
}
#[test]
fn diagnostic_prism_isolated_from_source_and_candidate_geometry_mesh_flags() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let batch = candidates(std::slice::from_ref(&source));
    let selected = &batch.candidates[selected_index(&batch)].face;
    let candidate_face = selected.clone();
    let candidate_mesh = attached_mesh(&candidate_face);
    let candidate_edge = first_edge(&candidate_face);
    let result = retain_selected_prism(batch, request([1., 1.]), &mut progress(None, None));
    assert_bidirectional_isolation(
        &source,
        &candidate_face,
        &candidate_edge,
        &candidate_mesh,
        &result,
    );
}
fn assert_bidirectional_isolation(
    source: &Shape,
    candidate: &Face,
    original: &Edge,
    original_mesh: &Handle<Triangulation>,
    result: &PrismReceipt,
) {
    let cap = mesh_cap(&result.diagnostic.as_ref().unwrap().body);
    let copied = first_edge(&cap);
    let copied_mesh = mesh(&cap);
    assert_independent_storage(&copied, original, &copied_mesh, original_mesh);
    mutate_candidate(source, candidate, original, original_mesh, &cap);
    assert_original_mesh(&copied_mesh);
    assert_eq!(pole(&copied), Point3::new(4., 0., 0.));
    assert_ne!(cap.closed().unwrap(), candidate.closed().unwrap());
    set_pole(&copied, 12.);
    change_mesh(&copied_mesh, 12.);
    assert_eq!(pole(original), Point3::new(9., 0., 0.));
    assert_original_changed_node(original_mesh);
}
#[test]
fn previous_diagnostic_survives_failure_cancel_recovery_on_same_source() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let first = retain_selected_prism(
        candidates(std::slice::from_ref(&source)),
        request([1., 1.]),
        &mut progress(None, None),
    );
    for (stop, fail) in [
        (Some(PrismPhase::History), None),
        (None, Some(PrismPhase::Copy)),
        (None, None),
    ] {
        let next = retain_selected_prism(
            candidates(std::slice::from_ref(&source)),
            request([1., 1.]),
            &mut progress(stop, fail),
        );
        assert_recovery_step(&next, &first);
    }
}

fn selected_index(batch: &crate::sketch_region_candidates::CandidateBatch) -> usize {
    use crate::sketch_region_candidates::{
        CandidateClassification, SeedSelection, classify_candidates,
    };
    let result = classify_candidates(batch, [1., 1.], request([1., 1.]).roots, || false)
        .unwrap_or_else(|_| panic!("expected stored classification"));
    let CandidateClassification::Incomplete {
        selection: SeedSelection::Unique { candidate },
        ..
    } = result
    else {
        panic!("expected unique diagnostic selection");
    };
    candidate
}

fn bezier_edge(body: &Shape) -> Edge {
    let shapes = equal_occurrences(body).unwrap();
    shapes
        .iter()
        .filter(|shape| shape.shape_type().unwrap() == ShapeEnum::Edge)
        .map(|shape| edge(shape).unwrap())
        .find(|edge| {
            !BRepTool::curve(edge)
                .unwrap()
                .0
                .cast::<GeomBezierCurve>()
                .is_null()
        })
        .expect("expected actual cubic boundary, not an extrusion line")
}
fn pcurve(edge: &Edge, face: &Face) -> Handle<Geom2dBezierCurve> {
    let curve = BRepTool::curve_on_face(edge, face).unwrap();
    assert!(curve.is_stored);
    let bezier = curve.pcurve.cast::<Geom2dBezierCurve>();
    assert!(!bezier.is_null());
    bezier
}
#[test]
fn copied_prism_pcurve_mutations_do_not_reach_candidate_or_previous_output() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let batch = candidates(&[source]);
    let candidate = batch.candidates[selected_index(&batch)].face.clone();
    attached_mesh(&candidate);
    let original = pcurve(&first_edge(&candidate), &candidate);
    let before = original.get().unwrap().pole(2).unwrap();
    let result = retain_selected_prism(batch, request([1., 1.]), &mut progress(None, None));
    let cap = mesh_cap(&result.diagnostic.as_ref().unwrap().body);
    let copied = pcurve(&first_edge(&cap), &cap);
    assert_pcurve_aliases(&original, &copied, before);
}

fn fill_mesh(mesh: &Triangulation) {
    for (index, point) in [
        Point3::default(),
        Point3::new(1., 0., 0.),
        Point3::new(0., 1., 0.),
    ]
    .into_iter()
    .enumerate()
    {
        fill_node(mesh, index as i32 + 1, point);
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    mesh.update_cached_min_max();
}
fn fill_node(mesh: &Triangulation, index: i32, point: Point3) {
    mesh.set_node(index, point).unwrap();
    mesh.set_uv_node(index, Point2::new(point.x(), point.y()))
        .unwrap();
    mesh.set_normal(index, Direction3::new(0., 0., 1.).unwrap())
        .unwrap();
}
fn mutate_candidate(
    source: &Shape,
    candidate: &Face,
    original: &Edge,
    original_mesh: &Handle<Triangulation>,
    cap: &Face,
) {
    set_pole(&edge(source).unwrap(), 8.);
    set_pole(original, 9.);
    candidate.set_closed(!cap.closed().unwrap()).unwrap();
    change_mesh(original_mesh, 9.);
}
fn assert_original_changed_node(original_mesh: &Handle<Triangulation>) {
    let actual = original_mesh.get().unwrap().node(2).unwrap();
    assert_eq!(actual, Point3::new(9., 0., 0.));
    assert_eq!(original_mesh.get().unwrap().deflection(), 9.);
}
fn assert_previous_pole(previous: &PrismReceipt) {
    let original = bezier_edge(&previous.diagnostic.as_ref().unwrap().body);
    assert_eq!(pole(&original).x(), 4.);
    assert!(matches!(previous.completion, PrismCompletion::Incomplete));
}
fn assert_pcurve_aliases(
    original: &Handle<Geom2dBezierCurve>,
    copied: &Handle<Geom2dBezierCurve>,
    before: Point2,
) {
    update_pcurve(original, 9.);
    assert_eq!(copied.get().unwrap().pole(2).unwrap(), before);
    update_pcurve(copied, 12.);
    assert_eq!(
        original.get().unwrap().pole(2).unwrap(),
        Point2::new(9., 0.)
    );
}

#[test]
fn failed_cancelled_requests_leave_original_candidate_representations_unchanged() {
    for phase in [
        PrismPhase::Copy,
        PrismPhase::Sweep,
        PrismPhase::History,
        PrismPhase::Validation,
    ] {
        for fail in [false, true] {
            assert_failed_request_isolation(phase, fail);
        }
    }
}
fn assert_failed_request_isolation(phase: PrismPhase, fail: bool) {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let batch = candidates(std::slice::from_ref(&source));
    let candidate = batch.candidates[selected_index(&batch)].face.clone();
    let edge = first_edge(&candidate);
    let curve = pcurve(&edge, &candidate);
    let before = curve.get().unwrap().pole(2).unwrap();
    let closed = candidate.closed().unwrap();
    let original_mesh = attached_mesh(&candidate);
    let mut observer = failed_progress(phase, fail);
    let result = retain_selected_prism(batch, request([1., 1.]), &mut observer);
    assert_failed_original(&original_mesh, &candidate, closed, &edge, &curve, before);
    set_pole(&rustycad_tkbrep::topods::edge(&source).unwrap(), 8.);
    assert_eq!(pole(&edge), Point3::new(4., 0., 0.));
    assert!(matches!(
        result.completion,
        PrismCompletion::Failed(_) | PrismCompletion::Cancelled(_)
    ));
}

fn assert_recovery_step(next: &PrismReceipt, first: &PrismReceipt) {
    assert!(next.limits.achieved_error.is_none());
    assert_previous_pole(first);
    assert!(first.diagnostic.is_some());
}

fn update_pcurve(curve: &Handle<Geom2dBezierCurve>, coordinate: f64) {
    curve
        .get()
        .unwrap()
        .set_pole(2, Point2::new(coordinate, 0.))
        .unwrap();
    assert_eq!(curve.get().unwrap().pole(2).unwrap().x(), coordinate);
    assert!(!curve.is_null());
}

fn assert_failed_original(
    mesh: &Handle<Triangulation>,
    face: &Face,
    closed: bool,
    edge: &Edge,
    curve: &Handle<Geom2dBezierCurve>,
    before: Point2,
) {
    assert_original_mesh(mesh);
    assert_eq!(face.closed().unwrap(), closed);
    assert_eq!(pole(edge), Point3::new(4., 0., 0.));
    assert_eq!(curve.get().unwrap().pole(2).unwrap(), before);
}

fn failed_progress(phase: PrismPhase, fail: bool) -> NamedPrismProgress {
    let stop = (!fail).then_some(phase);
    let failure = fail.then_some(phase);
    progress(stop, failure)
}
fn assert_independent_storage(
    copied: &Edge,
    original: &Edge,
    copied_mesh: &Handle<Triangulation>,
    original_mesh: &Handle<Triangulation>,
) {
    assert!(!copied.is_same(original));
    assert_ne!(copied_mesh.object_address(), original_mesh.object_address());
    assert_eq!(
        copied_mesh.get().unwrap().nb_nodes(),
        original_mesh.get().unwrap().nb_nodes()
    );
}
