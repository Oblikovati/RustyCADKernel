// SPDX-License-Identifier: GPL-2.0-only
//! A repeated history query does not create a new directed context or justify a replacement face.
use super::{tests::*, *};
use rustycad_tkbrep::{brep::BRepBuilder, topods::ShapeIterator};
use rustycad_tkmath::gp::Plane3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

#[test]
fn duplicate_or_replaced_contact_context_cannot_reuse_complete_closure_status() {
    for duplicate in [false, true] {
        let (mut bridge, shapes) = fixture();
        let contexts = &mut bridge.inspection.contacts[0]
            .bounded_construction
            .as_mut()
            .unwrap()
            .directed_contexts;
        let first = contexts[0].clone();
        if duplicate {
            contexts.push(first);
        } else {
            contexts[0].0 = usize::MAX;
        }
        assert_four_present(&run(&bridge, &shapes));
    }
}
fn replace_face(bridge: &mut PairedContactTransaction) {
    let copy = bridge.mapped.copied.as_mut().unwrap();
    let batch = copy.completion.as_mut().ok().unwrap();
    let candidate = &mut batch.candidates[0];
    let old = candidate.face.clone();
    let new = replacement_face(&old);
    for row in &mut candidate.relations {
        if row.output.is_equal(&old) {
            row.output = new.clone().into_shape();
        }
    }
    candidate.face = new;
}
fn replacement_face(old: &rustycad_tkbrep::topods::Face) -> rustycad_tkbrep::topods::Face {
    let mut new = BRepBuilderAPIMakeFace::from_plane(Plane3::default())
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    let wire = ShapeIterator::new(old, true, true).value().unwrap().clone();
    BRepBuilder::default().add(&mut new, &wire).unwrap();
    rustycad_tkbrep::topods::face(&new).unwrap()
}

#[test]
fn new_face_with_identical_boundary_and_parents_cannot_inherit_generated_context() {
    let (mut bridge, shapes) = fixture();
    replace_face(&mut bridge);
    let receipt = run(&bridge, &shapes);
    assert_four_present(&receipt);
    assert!(matches!(
        receipt.region.unwrap().decision,
        Err(RegionCertificateCause::Hypothesis {
            name: "one captured generated face identity",
            actual: 0,
            ..
        })
    ));
}
#[test]
fn duplicated_generated_member_identity_is_explicitly_unproved() {
    let (mut bridge, shapes) = fixture();
    let copy = bridge.mapped.copied.as_mut().unwrap();
    let crate::sketch_region_candidates::GeneratedTopologyObservation::Observed(areas) =
        &mut copy.generated_topology
    else {
        panic!("expected generated topology");
    };
    areas[0].members[1].occurrence = areas[0].members[0].occurrence.clone();
    assert_four_present(&run(&bridge, &shapes));
}

fn mutate_wire(bridge: &mut PairedContactTransaction, mode: usize) {
    let copy = bridge.mapped.copied.as_mut().unwrap();
    let candidate = &mut copy.completion.as_mut().ok().unwrap().candidates[0];
    let wire = ShapeIterator::new(&candidate.face, true, true)
        .value()
        .unwrap()
        .clone();
    let mut root = candidate.face.clone().into_shape();
    let builder = BRepBuilder::default();
    if mode != 1 {
        builder.remove(&mut root, &wire).unwrap();
    }
    if mode == 1 {
        builder.add(&mut root, &wire).unwrap();
    }
    if mode == 2 {
        builder.add(&mut root, &replacement_wire(&wire)).unwrap();
    }
}
fn replacement_wire(old: &rustycad_tkbrep::topods::Shape) -> rustycad_tkbrep::topods::Shape {
    let mut replacement = rustycad_tkbrep::topods::Wire::default();
    let builder = BRepBuilder::default();
    builder.make_wire(&mut replacement);
    let edge = ShapeIterator::new(old, true, true).value().unwrap().clone();
    let mut root = replacement.into_shape();
    builder.add(&mut root, &edge).unwrap();
    root
}
#[test]
fn missing_extra_or_reparented_actual_wire_leaves_all_region_gaps() {
    for mode in 0..3 {
        let (mut bridge, shapes) = fixture();
        mutate_wire(&mut bridge, mode);
        assert_four_present(&run(&bridge, &shapes));
    }
}
#[test]
fn changed_stored_pcurve_trim_does_not_reuse_full_range_classification() {
    let (bridge, shapes) = fixture();
    let candidate = &borrowed_batch(&bridge).unwrap().candidates[0];
    let edge = rustycad_tkbrep::topods::edge(&candidate.wires[0].segments[0].edge).unwrap();
    BRepBuilder::default()
        .range_on_face(&edge, &candidate.face, 0., 0.5)
        .unwrap();
    assert_four_present(&run(&bridge, &shapes));
}

#[test]
fn duplicate_descendant_omitting_current_identity_cannot_reuse_containment() {
    let (mut bridge, shapes) = fixture();
    let copy = bridge.mapped.copied.as_mut().unwrap();
    let crate::sketch_region_candidates::GeneratedTopologyObservation::Observed(areas) =
        &mut copy.generated_topology
    else {
        panic!("expected generated topology");
    };
    let descendants = &mut areas[0].members[0].descendants;
    assert!(descendants.len() > 1);
    assert_ne!(descendants[0], descendants[1]);
    descendants[1] = descendants[0].clone();
    assert!(containment::generated_universe(&bridge).is_err());
    assert_four_present(&run(&bridge, &shapes));
}
#[test]
fn moved_actual_shared_vertex_cannot_reuse_retained_construction_bounds() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    let face = &borrowed_batch(&bridge).unwrap().candidates[0].face;
    let members = crate::sketch_region::equal_occurrences(face).unwrap();
    let shape = members
        .iter()
        .find(|row| row.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Vertex)
        .unwrap();
    let vertex = rustycad_tkbrep::topods::vertex(shape).unwrap();
    BRepBuilder::default()
        .update_vertex_point(&vertex, rustycad_tkmath::gp::Point3::new(0., 0., 1.), 1e-7)
        .unwrap();
    retain_certificate(
        &bridge,
        &crate::sketch_region_partition_bindings::binding_sources(&shapes),
        &mut receipt,
    );
    assert_four_present(&receipt);
}
