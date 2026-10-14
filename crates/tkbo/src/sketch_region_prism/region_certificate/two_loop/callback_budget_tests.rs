// SPDX-License-Identifier: GPL-2.0-only
//! Review regression: rank callbacks occur after initial source admission but before pair recapture.
use super::{failure::*, fixture::*, fixture_bindings, operation::*, outcome::*, request::*};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, edge},
};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkmath::gp::Point3;
struct ReplaceDuringRank {
    edge: Edge,
    sources_polls: usize,
}
impl TwoLoopRegionProgress for ReplaceDuringRank {
    fn checkpoint(&mut self, stage: TwoLoopRegionStage) -> Result<bool, Failure> {
        if stage != TwoLoopRegionStage::Sources {
            return Ok(false);
        }
        self.sources_polls += 1;
        if self.sources_polls == 2 {
            replace_with_five_poles(&self.edge)?;
        }
        Ok(false)
    }
}
fn replace_with_five_poles(edge: &Edge) -> Result<(), Failure> {
    let points =
        [[0., 0.], [4., 0.], [2., 2.], [0., 4.], [0., 0.]].map(|p| Point3::new(p[0], p[1], 0.));
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles)?).cast();
    BRepBuilder::default().update_edge_curve(edge, &curve, 1e-7)
}
#[test]
fn rank_callback_curve_growth_is_admitted_before_legacy_recapture() {
    let fixture = TwoLoopFixture::nested();
    let sources = fixture.sources();
    let identities = fixture_bindings::identities(&fixture);
    let request = rank_request(&fixture, &sources, &identities);
    let mut progress = ReplaceDuringRank {
        edge: edge(&fixture.outer).unwrap(),
        sources_polls: 0,
    };
    let TwoLoopRegionOutcome::Incomplete(failure) =
        inspect_two_loop_region(&request, &mut progress)
    else {
        panic!("rank mutation; expected preadmission limit");
    };
    check_rank_refusal(&progress, &failure);
}

fn check_rank_refusal(progress: &ReplaceDuringRank, failure: &TwoLoopRegionFailure<'_>) {
    assert!(progress.sources_polls >= 2);
    assert!(matches!(
        failure.cause,
        TwoLoopRegionCause::Limit {
            operand: "polynomial poles",
            requested: 4,
            measured: 5
        }
    ));
    assert!(failure.entities.contains(&TwoLoopEntity::Source(OUTER)));
}

fn rank_request<'a, 'b>(
    fixture: &'a TwoLoopFixture,
    sources: &'b [crate::sketch_region::ArrangementSource<'a>],
    identities: &'b TwoLoopRegionIdentities,
) -> TwoLoopRegionRequest<'a, 'b> {
    let mut options = fixture_bindings::options();
    options.maximum_poles = 4;
    TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources,
        identities,
        inner_interior: [1., 1.],
        material_seed: [0.25, 0.25],
        requested: options,
    }
}
