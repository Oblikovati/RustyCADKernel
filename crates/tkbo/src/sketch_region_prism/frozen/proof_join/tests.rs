// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_partition_bindings::binding_sources;
use crate::sketch_region_prism::region_certificate::tests::{fixture, run};
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions, metric_bounds::BezierRegularityOptions,
};

fn with_full_linked(
    check: impl FnOnce(
        &PrismEmbeddingEvidence,
        &BoundProjection<'_>,
        &FrozenPrismProjection,
        &AuthoredClosureContact,
        bool,
        &JoinedClosureOperands,
    ),
) {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let authored = super::super::tests::authored_bindings(&receipt, &shapes[0]);
    let options =
        PrismEmbeddingOptions::v1(BezierRegularityOptions::v1(RootIsolationOptions::new(1e-6)));
    let proof = inspect_prism_embedding(
        &bridge,
        &binding_sources(&shapes),
        &receipt,
        options,
        || false,
    )
    .ok()
    .unwrap();
    // The semantic ID is authored independently; actual report indices are lookup evidence only.
    let contact = AuthoredClosureContact {
        name: AuthoredPrismId(900),
        actual: proof.binding().region.contact,
        original: proof.binding().region.original.clone(),
    };
    let frozen = super::super::tests::captured(super::super::freeze_prism(
        &receipt,
        &authored,
        &mut super::super::tests::NamedFreezeProgress,
    ));
    let bound = bind_projection(&receipt, &authored).unwrap();
    let selected_first =
        replay_sources::first_is_selected(&bridge, &proof.binding().region).unwrap();
    let closure = closure::capture(
        &bridge,
        &binding_sources(&shapes),
        &receipt,
        &proof,
        &authored,
        &contact,
    )
    .ok()
    .unwrap();
    check(&proof, &bound, &frozen, &contact, selected_first, &closure);
}
#[test]
fn same_actual_body_has_three_named_faces_six_trim_uses_and_all_parent_rows() {
    with_linked(|proof, bound, frozen, contact| {
        let links = links::link_proof(proof, bound, frozen, contact).unwrap();
        assert_eq!(links.contact, AuthoredPrismId(900));
        assert_eq!(links.original.member, AuthoredPrismId(703));
        assert_eq!(links.faces.len(), 3);
        assert_eq!(
            links
                .faces
                .iter()
                .map(|face| face.trims.len())
                .sum::<usize>(),
            6
        );
        assert_eq!(links.parents.len(), frozen.uses().len());
        assert!(links.faces.iter().all(|face| !face.parents.is_empty()));
        assert!(!frozen.is_valid_body());
    });
}
#[test]
fn missing_colliding_or_wrong_actual_contact_never_reuses_proof() {
    with_linked(|proof, bound, frozen, contact| {
        for name in [0, 700, 701, 703, 712] {
            let wrong = AuthoredClosureContact {
                name: AuthoredPrismId(name),
                actual: contact.actual,
                original: contact.original.clone(),
            };
            assert!(links::link_proof(proof, bound, frozen, &wrong).is_err());
        }
        let mut wrong = AuthoredClosureContact {
            name: contact.name,
            actual: contact.actual,
            original: contact.original.clone(),
        };
        wrong.actual.candidate_root += 1;
        assert!(links::link_proof(proof, bound, frozen, &wrong).is_err());
    });
}
#[test]
fn changed_frozen_chart_or_extrusion_cannot_borrow_same_identity_keys() {
    with_linked(|proof, bound, frozen, contact| {
        let mut changed = frozen.clone();
        changed.limits.extrusion[2] = 3.;
        assert!(links::link_proof(proof, bound, &changed, contact).is_err());
        let mut changed = frozen.clone();
        changed.faces[0].trims[0].interval[1] = 0.5;
        assert!(links::link_proof(proof, bound, &changed, contact).is_err());
        let mut changed = frozen.clone();
        changed.faces[0].trims[0].source = AuthoredPrismId(999);
        assert!(links::link_proof(proof, bound, &changed, contact).is_err());
    });
}
#[test]
fn complete_original_parent_multiset_preserves_duplicates_before_set_projection() {
    with_linked(|proof, bound, _, _| {
        let original = &proof.binding().region.original;
        let keys =
            parents::original_multiset(bound.bindings, &[original.clone(), original.clone()])
                .unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], keys[1]);
    });
}

fn with_linked(
    check: impl FnOnce(
        &PrismEmbeddingEvidence,
        &BoundProjection<'_>,
        &FrozenPrismProjection,
        &AuthoredClosureContact,
    ),
) {
    with_full_linked(|proof, bound, projection, contact, _, _| {
        check(proof, bound, projection, contact)
    });
}
fn operand_replay() -> Vec<u8> {
    let mut result = Vec::new();
    with_full_linked(
        |proof, bound, projection, contact, selected_first, closure| {
            let links = links::link_proof(proof, bound, projection, contact).unwrap();
            assert_eq!(links.raw_vertices.len(), 12);
            result =
                replay::joined_bytes(proof, bound, projection, &links, selected_first, closure)
                    .unwrap();
            assert!(result.len() > projection.replay_bytes().len());
        },
    );
    result
}
#[test]
fn complete_named_operand_replay_matches_independent_operations() {
    assert_eq!(operand_replay(), operand_replay());
}
#[test]
fn joined_raw_vertex_rows_retain_local_points_and_every_directed_owner() {
    with_linked(|proof, bound, projection, contact| {
        let links = links::link_proof(proof, bound, projection, contact).unwrap();
        assert_eq!(links.raw_vertices.len(), 12);
        for row in &links.raw_vertices {
            assert!(projection.nodes().iter().any(|node| node.key == row.vertex));
            assert!(
                projection
                    .uses()
                    .iter()
                    .any(|use_| use_.key == row.occurrence)
            );
            assert!(row.local_point.iter().all(|value| value.is_finite()));
        }
        let mut before = Vec::new();
        raw_vertices::append(&links.raw_vertices, &mut before);
        let vertex = bound
            .contexts
            .iter()
            .find(|context| {
                context.occurrence.shape_type().unwrap()
                    == rustycad_tkg3d::top_abs::ShapeEnum::Vertex
            })
            .unwrap();
        rustycad_tkbrep::brep::access::vertex_tshape(&vertex.occurrence, "owned join raw edit")
            .unwrap()
            .set_pnt(rustycad_tkmath::gp::Point3::new(1., 0., 0.));
        let fresh = raw_vertices::capture(bound, projection).unwrap();
        let mut after = Vec::new();
        raw_vertices::append(&fresh, &mut after);
        assert_ne!(before, after);
        let mut retained = Vec::new();
        raw_vertices::append(&links.raw_vertices, &mut retained);
        assert_eq!(before, retained);
    });
}
#[test]
fn complete_proof_owned_operand_types_are_send_sync() {
    fn check<T: Send + Sync>() {}
    check::<JoinedPrismFace>();
    check::<JoinedPrismTrim>();
    check::<JoinedParentDispositions>();
    check::<JoinedVertexOperand>();
}
fn complete_replay() -> Vec<u8> {
    super::operation_tests::complete_replay()
}
#[test]
fn joined_math_byte_receipt() {
    let bytes = complete_replay();
    println!(
        "same-body-prism-proof-v1:{}",
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}
#[test]
fn complete_joined_replay_matches_threads_and_two_processes() {
    let serial = complete_replay();
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(complete_replay))
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), serial);
    }
    for _ in 0..2 {
        assert_eq!(child_replay(), serial);
    }
}
fn child_replay() -> Vec<u8> {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "sketch_region_prism::frozen::proof_join::tests::joined_math_byte_receipt",
            "--nocapture",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let prefix = "same-body-prism-proof-v1:";
    let rows: Vec<_> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .collect();
    assert_eq!(rows.len(), 1);
    rows[0]
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn extra_or_missing_frozen_faces_uses_and_parent_occurrences_refuse_join() {
    with_linked(|proof, bound, frozen, contact| {
        let mut changed = frozen.clone();
        changed.faces.push(changed.faces[0].clone());
        assert!(links::link_proof(proof, bound, &changed, contact).is_err());
        let mut changed = frozen.clone();
        changed.faces.pop();
        assert!(links::link_proof(proof, bound, &changed, contact).is_err());
        let mut changed = frozen.clone();
        changed.uses.pop();
        assert!(links::link_proof(proof, bound, &changed, contact).is_err());
        let mut changed = frozen.clone();
        let trim = &mut changed.faces[0].trims[0];
        trim.parents.push(trim.parents[0].clone());
        assert!(links::link_proof(proof, bound, &changed, contact).is_err());
    });
}

#[test]
fn original_closure_query_and_all_semantic_raw_uses_are_owned_and_replayed() {
    with_full_linked(|proof, bound, _, contact, _, closure| {
        assert_eq!(closure.rows.len(), 12);
        for stage in [
            "original",
            "census",
            "partition",
            "selected",
            "complement",
            "copied",
        ] {
            assert_eq!(
                closure.rows.iter().filter(|row| row.stage == stage).count(),
                2
            );
        }
        assert!(closure.rows.iter().all(|row| row.contact == contact.name));
        assert_eq!(closure.query.original, proof.binding().region.original);
        assert!(
            closure
                .query
                .bounds
                .iter()
                .all(|bound| bound.compare_f64(0.).unwrap().is_eq())
        );
        let mut before = Vec::new();
        replay_closure::append(closure, bound.bindings, &mut before).unwrap();
        let mut changed = JoinedClosureOperands {
            rows: closure.rows.clone(),
            query: closure.query.clone(),
        };
        changed.rows[0].local_point[0] = 1.;
        let mut after = Vec::new();
        replay_closure::append(&changed, bound.bindings, &mut after).unwrap();
        assert_ne!(before, after);
        changed.rows = closure.rows.clone();
        changed.query.native_interval[1] = 2.;
        after.clear();
        replay_closure::append(&changed, bound.bindings, &mut after).unwrap();
        assert_ne!(before, after);
    });
}

#[test]
fn owned_original_singleton_query_retains_authoritative_parameter_mapping() {
    with_full_linked(|_, _, _, _, _, closure| {
        use rustycad_tkmath::predicates::roots::metric_bounds::*;
        let snapshot = &closure.query;
        let query = crate::sketch_region_source_conformance::with_captured_point_comparator(
            snapshot.source_query(),
            [0.; 3],
            snapshot.representation.placement,
            0.,
        );
        assert_eq!(query.maps, snapshot.maps);
        assert_eq!(query.residual.options, snapshot.roots);
        assert_eq!(query.residual.interval, snapshot.interval);
        let report =
            certify_transported_residual_on_bounds(query, snapshot.bounds.clone(), || false);
        assert!(
            matches!(report.decision,Ok(ResidualDecision::Within{proven_squared_upper}) if proven_squared_upper.compare_f64(0.).unwrap().is_eq())
        );
    });
}
