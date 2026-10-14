// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_contact_transport::PairedContactTransaction;
use crate::sketch_region_partition_bindings::binding_sources;
use crate::sketch_region_prism::{
    PrismReceipt,
    region_certificate::tests::{fixture, run},
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions, metric_bounds::BezierRegularityOptions,
};

pub(in crate::sketch_region_prism) struct JoinFixture {
    pub(in crate::sketch_region_prism) bridge: PairedContactTransaction,
    pub(in crate::sketch_region_prism) shapes: Vec<Shape>,
    pub(in crate::sketch_region_prism) receipt: PrismReceipt,
    pub(in crate::sketch_region_prism) authored: PrismPublicationBindings,
    pub(in crate::sketch_region_prism) contact: AuthoredClosureContact,
}
impl JoinFixture {
    pub(in crate::sketch_region_prism) fn new() -> Self {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        let authored = super::super::tests::authored_bindings(&receipt, &shapes[0]);
        let region = receipt.region.as_ref().unwrap().decision.as_ref().unwrap();
        let contact = AuthoredClosureContact {
            name: AuthoredPrismId(900),
            actual: region.contact,
            original: region.original.clone(),
        };
        Self {
            bridge,
            shapes,
            receipt,
            authored,
            contact,
        }
    }
    pub(in crate::sketch_region_prism) fn compose(
        &self,
        cancelled: impl FnMut() -> bool + Send,
    ) -> Result<JoinedPrismProof, PrismProofJoinFailure> {
        compose_prism_proof(
            &self.bridge,
            &binding_sources(&self.shapes),
            &self.receipt,
            &self.authored,
            &self.contact,
            options(),
            cancelled,
        )
    }
    pub(in crate::sketch_region_prism) fn rebind(
        &self,
        joined: &JoinedPrismProof,
    ) -> Result<(), PrismProofJoinFailure> {
        rebind_joined_prism_proof(
            &self.bridge,
            &binding_sources(&self.shapes),
            &self.receipt,
            &self.authored,
            joined,
        )
    }
}
fn options() -> PrismProofJoinOptions {
    PrismProofJoinOptions::v1(PrismEmbeddingOptions::v1(BezierRegularityOptions::v1(
        RootIsolationOptions::new(1e-6),
    )))
}
#[test]
fn complete_entry_owns_v2_and_every_original_operand_without_body_promotion() {
    let fixture = JoinFixture::new();
    let joined = fixture.compose(|| false).ok().unwrap();
    assert_eq!(joined.projection().version, 2);
    assert!(!joined.projection().is_valid_body());
    assert_eq!(joined.raw_vertices().len(), 12);
    assert_eq!(joined.raw_closure_points().len(), 12);
    assert!(fixture.rebind(&joined).is_ok());
    fn send_sync<T: Send + Sync>() {}
    send_sync::<JoinedPrismProof>();
    send_sync::<crate::sketch_region_contact_transport::ContactPointQuerySnapshot>();
}
#[test]
fn every_callback_cancellation_refuses_partial_join_and_preserves_reuse() {
    let fixture = JoinFixture::new();
    let mut polls = 0;
    let baseline = fixture
        .compose(|| {
            polls += 1;
            false
        })
        .ok()
        .unwrap();
    for stop in 1..=polls {
        let mut current = 0;
        assert!(
            fixture
                .compose(|| {
                    current += 1;
                    current == stop
                })
                .is_err()
        );
        assert_eq!(
            fixture.compose(|| false).ok().unwrap().replay_bytes(),
            baseline.replay_bytes()
        );
    }
}
#[test]
fn empty_source_and_census_candidates_refuse_before_callback() {
    let mut fixture = JoinFixture::new();
    let mut polls = 0;
    let refused = compose_prism_proof(
        &fixture.bridge,
        &[],
        &fixture.receipt,
        &fixture.authored,
        &fixture.contact,
        options(),
        || {
            polls += 1;
            false
        },
    );
    assert!(refused.is_err());
    assert_eq!(polls, 0);
    fixture.bridge.census.inputs.segments.clear();
    assert!(
        fixture
            .compose(|| {
                polls += 1;
                false
            })
            .is_err()
    );
    assert_eq!(polls, 0);
    let mut fixture = JoinFixture::new();
    fixture
        .bridge
        .mapped
        .copied
        .as_mut()
        .unwrap()
        .completion
        .as_mut()
        .ok()
        .unwrap()
        .candidates
        .clear();
    assert!(
        fixture
            .compose(|| {
                polls += 1;
                false
            })
            .is_err()
    );
    assert_eq!(polls, 0);
}
fn body_vertex(fixture: &JoinFixture) -> Shape {
    let contexts = crate::sketch_region_prism::contexts::body_contexts(
        &fixture.receipt.diagnostic.as_ref().unwrap().body,
    )
    .ok()
    .unwrap();
    contexts
        .into_iter()
        .find(|row| {
            row.occurrence.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Vertex
        })
        .unwrap()
        .occurrence
}
fn mutate_vertex(shape: &Shape) {
    rustycad_tkbrep::brep::access::vertex_tshape(shape, "owned join callback mutation")
        .unwrap()
        .set_pnt(rustycad_tkmath::gp::Point3::new(0.25, 0., 0.));
}
#[test]
fn last_callback_body_mutation_refuses_and_previous_publication_remains_owned() {
    let fixture = JoinFixture::new();
    let mut polls = 0;
    let old = fixture
        .compose(|| {
            polls += 1;
            false
        })
        .ok()
        .unwrap();
    let bytes = old.replay_bytes().to_vec();
    let target = body_vertex(&fixture);
    let mut current = 0;
    assert!(
        fixture
            .compose(|| {
                current += 1;
                if current == polls {
                    mutate_vertex(&target);
                }
                false
            })
            .is_err()
    );
    assert_eq!(old.replay_bytes(), bytes);
    assert!(fixture.rebind(&old).is_err());
}
#[test]
fn last_callback_original_mutation_cannot_inherit_closure_proof() {
    let fixture = JoinFixture::new();
    let mut polls = 0;
    let old = fixture
        .compose(|| {
            polls += 1;
            false
        })
        .ok()
        .unwrap();
    let source = crate::sketch_region_prism::region_certificate::current_closure_vertices(
        &fixture.bridge,
        &binding_sources(&fixture.shapes),
        &fixture.receipt,
    )
    .unwrap()
    .remove(0)
    .1;
    let mut current = 0;
    assert!(
        fixture
            .compose(|| {
                current += 1;
                if current == polls {
                    mutate_vertex(&source);
                }
                false
            })
            .is_err()
    );
    assert!(fixture.rebind(&old).is_err());
}

pub(super) fn complete_replay() -> Vec<u8> {
    JoinFixture::new()
        .compose(|| false)
        .ok()
        .unwrap()
        .replay_bytes()
        .to_vec()
}

#[test]
fn changed_history_multiplicity_and_authored_names_refuse_final_rebind() {
    let mut fixture = JoinFixture::new();
    let old = fixture.compose(|| false).ok().unwrap();
    let row = fixture
        .receipt
        .diagnostic
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|row| {
            !row.modified.is_empty() || !row.preserved.is_empty() || !row.generated.is_empty()
        })
        .unwrap();
    if let Some(parent) = row.modified.first().cloned() {
        row.modified.push(parent);
    } else if let Some(parent) = row.preserved.first().cloned() {
        row.preserved.push(parent);
    } else {
        row.generated.push(row.generated[0].clone());
    }
    assert!(fixture.rebind(&old).is_err());
    let mut fixture = JoinFixture::new();
    let old = fixture.compose(|| false).ok().unwrap();
    fixture.authored.operation = AuthoredPrismId(9900);
    assert!(fixture.rebind(&old).is_err());
}
#[test]
fn source_coefficients_changed_at_final_callback_refuse_and_keep_owned_replay() {
    use rustycad_tkg3d::geom::GeomBezierCurve;
    let fixture = JoinFixture::new();
    let mut polls = 0;
    let old = fixture
        .compose(|| {
            polls += 1;
            false
        })
        .ok()
        .unwrap();
    let bytes = old.replay_bytes().to_vec();
    let source = rustycad_tkbrep::topods::edge(&fixture.shapes[0]).unwrap();
    let curve = rustycad_tkbrep::brep::BRepTool::curve_with_location(&source)
        .unwrap()
        .curve
        .cast::<GeomBezierCurve>();
    let mut current = 0;
    assert!(
        fixture
            .compose(|| {
                current += 1;
                if current == polls {
                    curve
                        .set_pole(2, rustycad_tkmath::gp::Point3::new(5., 0., 0.))
                        .unwrap();
                }
                false
            })
            .is_err()
    );
    assert_eq!(old.replay_bytes(), bytes);
}

#[test]
fn final_callback_actual_cap_trim_mutation_refuses_owned_join() {
    use rustycad_tkbrep::{brep::BRepBuilder, topods::face};
    let fixture = JoinFixture::new();
    let mut polls = 0;
    let old = fixture
        .compose(|| {
            polls += 1;
            false
        })
        .ok()
        .unwrap();
    let last = fixture
        .receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|row| {
            row.native.role == crate::sketch_region_prism::PrismImageRole::Last
                && row.context.occurrence.shape_type().unwrap()
                    == rustycad_tkg3d::top_abs::ShapeEnum::Face
        })
        .unwrap();
    let face = face(&last.context.occurrence).unwrap();
    let edge = crate::sketch_region_prism::isolation_tests::first_edge(&face);
    let mut current = 0;
    assert!(
        fixture
            .compose(|| {
                current += 1;
                if current == polls {
                    BRepBuilder::default()
                        .range_on_face(&edge, &face, 0., 0.5)
                        .unwrap();
                }
                false
            })
            .is_err()
    );
    assert!(fixture.rebind(&old).is_err());
}
