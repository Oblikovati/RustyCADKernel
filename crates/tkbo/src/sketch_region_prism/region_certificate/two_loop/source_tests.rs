// SPDX-License-Identifier: GPL-2.0-only
use super::{failure::*, fixture::*, outcome::*, request::*, sources, work::*};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
struct SourceProgress;
impl TwoLoopRegionProgress for SourceProgress {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, Failure> {
        Ok(false)
    }
}
pub(super) fn capture(fixture: &TwoLoopFixture) -> TwoLoopCheck<'_, Vec<TwoLoopSourceProof>> {
    let originals = fixture.sources();
    let identities = TwoLoopRegionIdentities {
        operation: 77,
        outer: OUTER,
        inner: INNER,
        selected: 301,
        candidates: Vec::new(),
    };
    let requested = TwoLoopRegionOptions {
        behavior: TwoLoopRegionBehavior::NestedPolynomialV2,
        winding: RootIsolationOptions::new(0.001),
        maximum_retained_units: 4096,
        maximum_poles: 33,
    };
    let request = TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources: &originals,
        identities: &identities,
        inner_interior: [1., 1.],
        material_seed: [0.25, 0.25],
        requested,
    };
    let mut progress = SourceProgress;
    sources::capture(
        &request,
        &mut TwoLoopWork::new(77, requested, &mut progress),
    )
}
#[test]
fn both_actual_sources_preserve_native_closure_roles_and_exact_stage_proofs() {
    let fixture = TwoLoopFixture::nested();
    let proofs = capture(&fixture).unwrap_or_else(|e| panic!("source proof: {:?}", e.cause));
    assert_eq!(proofs.len(), 2);
    for proof in &proofs {
        assert_eq!(proof.stages.len(), 3);
        assert_eq!(
            proof
                .closure
                .iter()
                .map(|p| p.parameter)
                .collect::<Vec<_>>(),
            [0., 1.]
        );
        assert_eq!(
            proof
                .copied_closure
                .iter()
                .map(|p| p.parameter)
                .collect::<Vec<_>>(),
            [0., 1.]
        );
    }
    assert_eq!(proofs[1].closure[0].local_point, [0.5, 0.5, 0.]);
}
#[test]
fn reversed_actual_edges_keep_native_endpoint_roles_and_recorded_orientation() {
    use rustycad_tkg3d::top_abs::Orientation;
    let fixture = TwoLoopFixture::for_edges(
        cubic(1., [0., 0.]).oriented(Orientation::Reversed),
        cubic(0.5, [0.5, 0.5]).oriented(Orientation::Reversed),
    );
    let proofs =
        capture(&fixture).unwrap_or_else(|e| panic!("reversed source proof: {:?}", e.cause));
    assert!(
        proofs
            .iter()
            .all(|p| p.orientation == Orientation::Reversed)
    );
    assert!(
        proofs
            .iter()
            .all(|p| p.closure[0].parameter == 0. && p.closure[1].parameter == 1.)
    );
}
#[test]
fn same_identity_curve_and_vertex_mutations_cannot_reuse_source_proofs() {
    use rustycad_tkbrep::{
        brep::BRepBuilder,
        topods::{ShapeIterator, edge, vertex},
    };
    let fixture = TwoLoopFixture::nested();
    let before = capture(&fixture).unwrap_or_else(|e| panic!("source proof: {:?}", e.cause));
    let endpoint = ShapeIterator::new(&fixture.inner, true, true)
        .value()
        .unwrap()
        .clone();
    BRepBuilder::default()
        .update_vertex_point(
            &vertex(&endpoint).unwrap(),
            rustycad_tkmath::gp::Point3::new(0.5, 0.5, 1.),
            1e-7,
        )
        .unwrap();
    assert!(matches!(
        capture(&fixture).err().unwrap().cause,
        TwoLoopRegionCause::ResidualMismatch(_)
    ));
    assert_eq!(before[1].closure[0].local_point, [0.5, 0.5, 0.]);
    let fixture = TwoLoopFixture::nested();
    crate::sketch_region_prism::isolation_tests::set_pole(&edge(&fixture.outer).unwrap(), 5.);
    assert!(capture(&fixture).is_err());
}
