// SPDX-License-Identifier: GPL-2.0-only
//! `extrude` against closed-form volumes, independent adaptive volume integration, the
//! Euler-Poincare relation and complete one-to-one history.
use super::fixtures::*;
use rustycad_tkbo::profile_extrusion::*;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkbrep::{topexp, toptools::IndexedMapOfShape};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

static_assertions::assert_impl_all!(OperationResult: Send, Sync);
static_assertions::assert_impl_all!(PlanarRegionSet: Send, Sync);
static_assertions::assert_impl_all!(OperationDecline: Send, Sync);

pub fn options() -> ExtrudeOptions {
    ExtrudeOptions { resolution: resolution() }
}

pub fn extruded(profile: &SketchProfile, region: usize, direction: [f64; 3]) -> OperationResult {
    let set = build_planar_regions(profile, resolution()).unwrap();
    extrude(&set.regions()[region], direction, &options()).unwrap()
}

/// Adaptive Gauss-Kronrod volume of the native solid (no tessellation).
fn integrated_volume(result: &OperationResult) -> f64 {
    let mut props = GProps::new();
    BRepGProp::volume_properties_eps(result.solid(), &mut props, 1e-10, true, false).unwrap();
    props.mass()
}

fn assert_volume(result: &OperationResult, expected: f64) {
    assert!(result.volume().encloses(expected), "{:?} vs {expected}", result.volume());
    let integrated = integrated_volume(result);
    assert!((integrated - expected).abs() < 1e-7 * expected.max(1.0), "{integrated} vs {expected}");
}

/// Inner wires (rings) of all faces.
fn rings(result: &OperationResult) -> usize {
    result.faces().iter().map(|face| wire_count(face) - 1).sum()
}

fn wire_count(face: &Face) -> usize {
    let mut wires = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(face, ShapeEnum::Wire, &mut wires).unwrap();
    wires.extent()
}

/// Euler-Poincare for one closed shell: V - E + F - R = 2 (1 - genus).
fn assert_euler(result: &OperationResult, genus: i64) {
    let (v, e, f) = (result.vertices().len(), result.edges().len(), result.faces().len());
    let r = rings(result);
    assert_eq!(v as i64 - e as i64 + f as i64 - r as i64, 2 * (1 - genus), "V{v} E{e} F{f} R{r}");
}

/// History oracle: every profile curve has exactly one side face, both caps come from the
/// region, and every result entity has exactly one parent.
fn assert_history(result: &OperationResult, region: &PlanarRegion) {
    let history = result.history();
    for (curve, _) in region.edges() {
        let sides = history.of(HistorySource::Curve(*curve));
        let sides = sides.iter().filter(|record| record.role == HistoryRole::SideFace);
        assert_eq!(sides.count(), 1, "{curve:?}");
    }
    let caps: Vec<_> = history.of(HistorySource::Region(region.id())).iter().map(|r| r.role).collect();
    assert_eq!(caps, vec![HistoryRole::StartCap, HistoryRole::EndCap]);
    let entities = (0..result.faces().len()).map(ResultEntity::Face)
        .chain((0..result.edges().len()).map(ResultEntity::Edge))
        .chain((0..result.vertices().len()).map(ResultEntity::Vertex));
    for entity in entities {
        let parents = history.records.iter().filter(|record| record.result == entity).count();
        assert_eq!(parents, 1, "{entity:?}");
    }
    assert!(history.deleted.is_empty());
    assert!(history.records.windows(2).all(|pair| pair[0] < pair[1]));
}

fn cap_heights(result: &OperationResult, role: HistoryRole) -> Vec<f64> {
    let record = result.history().records.iter().find(|record| record.role == role).unwrap();
    let ResultEntity::Face(index) = record.result else { panic!("{record:?}") };
    let mut vertices = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&result.faces()[index], ShapeEnum::Vertex, &mut vertices).unwrap();
    (1..=vertices.extent())
        .map(|i| BRepTool::pnt(&rustycad_tkbrep::topods::vertex(vertices.find_key(i).unwrap()).unwrap()).unwrap().z())
        .collect()
}

fn region_of(profile: &SketchProfile, index: usize) -> PlanarRegion {
    build_planar_regions(profile, resolution()).unwrap().regions()[index].clone()
}

#[test]
fn rectangle_extrudes_to_a_box() {
    let profile = rectangle();
    let result = extruded(&profile, 0, [0., 0., 4.]);
    assert_volume(&result, 24.0);
    assert_eq!((result.vertices().len(), result.edges().len(), result.faces().len()), (8, 12, 6));
    assert_euler(&result, 0);
    assert_history(&result, &region_of(&profile, 0));
    assert!(cap_heights(&result, HistoryRole::StartCap).iter().all(|z| *z == 0.0));
    assert!(cap_heights(&result, HistoryRole::EndCap).iter().all(|z| (*z - 4.0).abs() < 1e-15));
    assert!(result.validation().is_valid(), "{:?}", result.validation());
    assert_eq!(result.self_intersection_certificate(), SelfIntersectionEvidence::AffineSweepOfCertifiedRegion);
    assert!(result.achieved_tolerance().max() <= 1e-6);
}

#[test]
fn circle_extrudes_to_a_cylinder_with_seam_history() {
    let profile = circle();
    let result = extruded(&profile, 0, [0., 0., 2.]);
    assert_volume(&result, PI * 1.5 * 1.5 * 2.0);
    assert_eq!((result.vertices().len(), result.edges().len(), result.faces().len()), (2, 3, 3));
    assert_euler(&result, 0);
    assert_history(&result, &region_of(&profile, 0));
    let roles: Vec<_> = result.history().of(HistorySource::Curve(ProfileCurveId(7))).iter().map(|r| r.role).collect();
    assert!(roles.contains(&HistoryRole::LateralEdge) && roles.contains(&HistoryRole::StartVertex));
}

#[test]
fn annulus_ring_extrudes_to_a_tube_of_genus_one() {
    let profile = annulus();
    let result = extruded(&profile, 0, [0., 0., 1.5]);
    assert_volume(&result, 3.0 * PI * 1.5);
    assert_euler(&result, 1);
    assert_eq!(rings(&result), 2);
    assert_history(&result, &region_of(&profile, 0));
}

#[test]
fn slot_extrudes_with_one_side_face_per_curve() {
    let profile = slot();
    let result = extruded(&profile, 0, [0., 0., 0.5]);
    assert_volume(&result, (8.0 + PI) * 0.5);
    assert_eq!(result.faces().len(), 6);
    assert_euler(&result, 0);
    assert_history(&result, &region_of(&profile, 0));
}

#[test]
fn each_of_two_disjoint_regions_extrudes_independently() {
    let profile = two_rectangles();
    let first = extruded(&profile, 0, [0., 0., 1.]);
    let second = extruded(&profile, 1, [0., 0., -2.]);
    assert_volume(&first, 1.0);
    assert_volume(&second, 4.0);
    assert!(cap_heights(&second, HistoryRole::EndCap).iter().all(|z| (*z + 2.0).abs() < 1e-15));
}

#[test]
fn oblique_extrusion_on_a_tilted_plane_has_area_times_height() {
    let mut profile = slot();
    profile.plane = ProfilePlane { origin: [1., 2., 3.], x_direction: [0., 0., 1.], normal: [1., 0., 0.] };
    let result = extruded(&profile, 0, [2., 5., -1.]);
    assert_volume(&result, (8.0 + PI) * 2.0);
}

#[test]
fn extrusion_is_deterministic_and_keeps_the_region_unchanged() {
    let region = region_of(&annulus(), 0);
    let before = BRepTool::max_tolerance(region.face(), ShapeEnum::Edge).unwrap();
    let first = extrude(&region, [0., 0., 1.], &options()).unwrap();
    let second = extrude(&region, [0., 0., 1.], &options()).unwrap();
    assert_eq!(first.history(), second.history());
    assert_eq!(BRepTool::max_tolerance(region.face(), ShapeEnum::Edge).unwrap(), before);
    let mut inputs = IndexedMapOfShape::default();
    topexp::map_shapes(region.face(), &mut inputs, true, true);
    let shared = (1..=inputs.extent()).filter(|i| shares(&first, inputs.find_key(*i).unwrap())).count();
    assert_eq!(shared, 0, "the solid shares input topology");
    let detached = first.detached_solid().unwrap();
    assert!(!detached.is_same(first.solid()));
}

fn shares(result: &OperationResult, input: &Shape) -> bool {
    let mut all = IndexedMapOfShape::default();
    topexp::map_shapes(result.solid(), &mut all, true, true);
    all.contains(input)
}

#[test]
fn invalid_directions_are_declined_by_name() {
    let region = region_of(&rectangle(), 0);
    for direction in [[1., 1., 0.], [0., 0., 0.], [f64::NAN, 0., 1.]] {
        let decline = extrude(&region, direction, &options()).unwrap_err();
        assert_eq!((decline.operation, decline.stage), (OperationKind::Extrude, OperationStage::Admission));
        assert!(matches!(decline.cause, DeclineCause::InvalidDirection { .. }), "{decline}");
    }
}

#[test]
fn over_budget_solid_is_refused_and_report_cap_records_a_finding() {
    let region = region_of(&rectangle(), 0);
    let mut strict = options();
    strict.resolution.cap = ToleranceCap::Refuse(1e-9);
    let decline = extrude(&region, [0., 0., 1.], &strict).unwrap_err();
    assert_eq!((decline.stage, decline.class()), (OperationStage::Tolerance, DeclineClass::OverBudget));
    strict.resolution.cap = ToleranceCap::Report(1e-9);
    let result = extrude(&region, [0., 0., 1.], &strict).unwrap();
    assert!(matches!(result.findings(), [Finding::ToleranceAboveReportLimit { limit, .. }] if *limit == 1e-9));
}

#[test]
fn results_are_equal_across_threads() {
    let profile = slot();
    let local = extruded(&profile, 0, [0., 0., 1.]);
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let profile = profile.clone();
            std::thread::spawn(move || extruded(&profile, 0, [0., 0., 1.]))
        })
        .collect();
    for thread in threads {
        let remote = thread.join().unwrap();
        assert_eq!(remote.history(), local.history());
        assert_eq!(remote.volume(), local.volume());
        assert_eq!(remote.achieved_tolerance(), local.achieved_tolerance());
    }
}

/// ADR 0082: the result's validation is the public `validate_body` report of its solid:
/// the same four stages, modes, census and budget, every stage passed.
#[test]
fn operation_validation_is_the_public_body_report() {
    use rustycad_tkbo::body_check::{CheckMode, StageVerdict, ValidationStage, validate_body};
    for (profile, genus) in [(rectangle(), 0), (circle(), 0), (annulus(), 1)] {
        let result = extruded(&profile, 0, [0., 0., 1.]);
        let report: &ValidationReport = result.validation();
        assert_eq!(report, &validate_body(result.solid(), &resolution()).unwrap());
        assert!(report.is_valid(), "{report:?}");
        let modes = report.stages.iter().map(|stage| stage.mode).collect::<Vec<_>>();
        assert_eq!(modes[0], CheckMode::Exact);
        assert_eq!(modes[2], CheckMode::NativeSearch);
        assert_eq!(report.stage(ValidationStage::Tolerance).verdict, StageVerdict::Passed);
        assert_eq!(report.census[0].genus, Some(genus));
        assert_eq!(report.tolerance_budget, Some(1e-6));
    }
}

/// A report cap is a finding, not a validation defect: the body check runs without a budget.
#[test]
fn report_cap_keeps_a_valid_report_without_budget() {
    let region = region_of(&rectangle(), 0);
    let mut lenient = options();
    lenient.resolution.cap = ToleranceCap::Report(1e-9);
    let result = extrude(&region, [0., 0., 1.], &lenient).unwrap();
    assert!(result.validation().is_valid());
    assert_eq!(result.validation().tolerance_budget, None);
    assert_eq!(result.findings().len(), 1);
}
