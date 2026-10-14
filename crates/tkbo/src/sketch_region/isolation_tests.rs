// SPDX-License-Identifier: GPL-2.0-only
//! Shared incidence and reachable polygon storage are tested by actual mutations, not copy flags.
use super::{arrangement::*, copy_inputs::*, outcome::*, tests::*};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::edge,
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    poly::Polygon3D,
    toploc::Location,
};

#[test]
fn reversed_aliases_copy_together_with_all_distinct_caller_parent_ids() {
    let original = closed_cubic();
    let reversed = original.oriented(Orientation::Reversed);
    let inputs = alias_inputs(&original, &reversed);
    let copied = copy_sources(&inputs).unwrap_or_else(|_| panic!("expected aggregate copy"));
    assert!(copied.roots[0].1.is_same(&copied.roots[1].1));
    assert!(!copied.roots[0].1.is_same(&original));
    assert_eq!(
        copied.arguments.last().unwrap().orientation(),
        Orientation::Reversed
    );
    let ArrangementCompletion::Arranged(output) = arrange_owned_sources(&inputs, &mut |_| false)
    else {
        panic!("expected stage")
    };
    assert_alias_parent_ids(&output);
}
fn alias_inputs<'a>(
    original: &'a rustycad_tkbrep::topods::Shape,
    reversed: &'a rustycad_tkbrep::topods::Shape,
) -> [ArrangementSource<'a>; 2] {
    [
        ArrangementSource {
            identity: RegionSourceId(9),
            shape: original,
        },
        ArrangementSource {
            identity: RegionSourceId(2),
            shape: reversed,
        },
    ]
}
fn assert_alias_parent_ids(output: &OwnedArrangement) {
    let mut identities: Vec<_> = output
        .relations
        .last()
        .unwrap()
        .generated
        .iter()
        .map(|key| key.source)
        .collect();
    identities.dedup();
    assert_eq!(identities, [RegionSourceId(2), RegionSourceId(9)]);
}
fn translated(original: &rustycad_tkbrep::topods::Shape, x: f64) -> rustycad_tkbrep::topods::Shape {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(x, 4., 5.));
    original.located(&Location::new(transform), true).unwrap()
}
#[test]
fn two_locations_of_same_topology_preserve_distinct_world_placements() {
    let original = closed_cubic();
    let first = translated(&original, 3.);
    let second = translated(&original, 8.);
    let inputs = [
        ArrangementSource {
            identity: RegionSourceId(1),
            shape: &first,
        },
        ArrangementSource {
            identity: RegionSourceId(2),
            shape: &second,
        },
    ];
    let copied = copy_sources(&inputs).unwrap_or_else(|_| panic!("expected locatedcopy"));
    for (index, source) in [&first, &second].iter().enumerate() {
        let expected = world_start(source);
        assert_eq!(world_start(&copied.roots[index].1), expected);
    }
}
fn world_start(shape: &rustycad_tkbrep::topods::Shape) -> Point3 {
    let curve = BRepTool::curve(&edge(shape).unwrap()).unwrap().0;
    curve.value(0.).unwrap()
}
pub(crate) fn add_polygon(source: &rustycad_tkbrep::topods::Shape) -> Handle<Polygon3D> {
    let vertices = [Point3::new(0., 0., 0.), Point3::new(1., 2., 0.)];
    let points = Array1::from_shared(&vertices, 1);
    let polygon = Handle::new(Polygon3D::from_nodes(&points));
    BRepBuilder::default()
        .update_edge_polygon3d(&edge(source).unwrap(), &polygon, &Location::default())
        .unwrap();
    polygon
}
#[test]
fn copied_polygon_nodes_and_deflection_are_independent_in_both_directions() {
    let source = closed_cubic();
    let original = add_polygon(&source);
    let copied = copy_sources(&sources(&source)).unwrap_or_else(|_| panic!("expected meshcopy"));
    let cloned = BRepTool::polygon3d(&edge(&copied.roots[0].1).unwrap())
        .unwrap()
        .0;
    assert_ne!(original.object_address(), cloned.object_address());
    original
        .change_nodes(|nodes| nodes.set_value(1, Point3::new(9., 0., 0.)))
        .unwrap();
    assert_eq!(*cloned.nodes().value(1).unwrap(), Point3::new(0., 0., 0.));
    cloned.set_deflection(0.25);
    assert_eq!(original.deflection(), 0.);
}
#[test]
fn published_output_survives_failure_cancellation_and_shared_source_reuse() {
    let source = closed_cubic();
    let old = arranged(&source);
    let reversed = source.oriented(Orientation::Reversed);
    let result =
        arrange_with_executor(&sources(&reversed), &mut |_| false, &mut FailingMappedStage);
    assert!(matches!(result, ArrangementCompletion::Failed(_)));
    let result = arrange_owned_sources(&sources(&reversed), &mut |phase| {
        phase == ArrangementPhase::History
    });
    assert!(matches!(result, ArrangementCompletion::Cancelled(_)));
    assert_eq!(
        curve_of(&old.arrangement).pole(2).unwrap(),
        Point3::new(4., 0., 0.)
    );
    assert_eq!(curve_of(&source).pole(2).unwrap(), Point3::new(4., 0., 0.));
}

#[test]
fn distinct_nonintersecting_sources_run_same_pipeline_and_keep_both_parent_sets() {
    let original = closed_cubic();
    let first = translated(&original, 3.);
    let second = translated(&original, 8.);
    let inputs = alias_inputs(&first, &second);
    let ArrangementCompletion::Arranged(output) = arrange_owned_sources(&inputs, &mut |_| false)
    else {
        panic!("expected multiple-source stage")
    };
    assert_alias_parent_ids(&output);
    assert_eq!(
        curve_of(&original).pole(2).unwrap(),
        Point3::new(4., 0., 0.)
    );
}

#[test]
fn copied_then_unchanged_member_is_modified_from_original_not_preserved() {
    let original = closed_cubic();
    let output = arranged(&original);
    assert!(
        output
            .relations
            .iter()
            .all(|relation| relation.preserved.is_empty())
    );
    let retained = only_edge_relation(&output);
    assert!(!retained.output.is_same(&original));
    assert!(
        retained
            .modified
            .iter()
            .any(|key| key.topology_address == original.tshape().object_address().addr())
    );
}

fn only_edge_relation(output: &OwnedArrangement) -> &ArrangementRelation {
    let retained: Vec<_> = output
        .relations
        .iter()
        .filter(|relation| {
            relation.output.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Edge
        })
        .collect();
    assert_eq!(retained.len(), 1);
    retained[0]
}
