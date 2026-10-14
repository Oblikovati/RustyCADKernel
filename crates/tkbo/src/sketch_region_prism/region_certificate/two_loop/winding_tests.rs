// SPDX-License-Identifier: GPL-2.0-only
use super::{failure::*, fixture::*, request::*, source_tests, winding, work::*};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
struct WindingProgress {
    stop: bool,
}
impl TwoLoopRegionProgress for WindingProgress {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, Failure> {
        Ok(self.stop)
    }
}
fn options() -> TwoLoopRegionOptions {
    TwoLoopRegionOptions {
        behavior: TwoLoopRegionBehavior::NestedPolynomialV2,
        winding: RootIsolationOptions::new(0.001),
        maximum_retained_units: 4096,
        maximum_poles: 33,
    }
}
#[test]
fn exact_witnesses_have_independent_expected_winding_and_native_reversal() {
    for orientation in [Orientation::Forward, Orientation::Reversed] {
        let fixture = TwoLoopFixture::for_edges(
            cubic(1., [0., 0.]).oriented(orientation),
            cubic(0.5, [0.5, 0.5]).oriented(orientation),
        );
        let proofs = source_tests::capture(&fixture).unwrap_or_else(|e| panic!("{:?}", e.cause));
        let mut progress = WindingProgress { stop: false };
        let mut work = TwoLoopWork::new(77, options(), &mut progress);
        let outer = winding::query(&proofs[0], [0.5, 0.5], &mut work)
            .unwrap_or_else(|e| panic!("{:?}", e.cause));
        let inner = winding::query(&proofs[1], [1., 1.], &mut work)
            .unwrap_or_else(|e| panic!("{:?}", e.cause));
        let expected = if orientation == Orientation::Forward {
            1
        } else {
            -1
        };
        assert_eq!(winding::unit_interior(&outer, &work).ok(), Some(expected));
        assert_eq!(winding::unit_interior(&inner, &work).ok(), Some(expected));
        assert_eq!(
            work.winding_bisections,
            outer.result.work().new_bisections + inner.result.work().new_bisections
        );
    }
}
#[test]
fn invalid_inner_witness_and_terminal_contact_never_prove_strict_interior() {
    let fixture = TwoLoopFixture::nested();
    let proofs = source_tests::capture(&fixture).unwrap_or_else(|e| panic!("{:?}", e.cause));
    let mut progress = WindingProgress { stop: false };
    let mut work = TwoLoopWork::new(77, options(), &mut progress);
    for seed in [[0.25, 0.25], [0.5, 0.5]] {
        let proof =
            winding::query(&proofs[1], seed, &mut work).unwrap_or_else(|e| panic!("{:?}", e.cause));
        assert!(matches!(
            winding::unit_interior(&proof, &work).err().unwrap().cause,
            TwoLoopRegionCause::WindingLocation { .. }
        ));
    }
}
#[test]
fn winding_cancel_and_exhaustion_preserve_typed_refusal_and_attempt_work() {
    let fixture = TwoLoopFixture::nested();
    let proofs = source_tests::capture(&fixture).unwrap_or_else(|e| panic!("{:?}", e.cause));
    let mut progress = WindingProgress { stop: true };
    let mut work = TwoLoopWork::new(77, options(), &mut progress);
    assert!(matches!(
        winding::query(&proofs[0], [0.5, 0.5], &mut work)
            .err()
            .unwrap()
            .cause,
        TwoLoopRegionCause::Cancelled
    ));
    let mut progress = WindingProgress { stop: false };
    let mut requested = options();
    requested.winding.max_bisections = 0;
    let mut work = TwoLoopWork::new(77, requested, &mut progress);
    assert!(matches!(
        winding::query(&proofs[0], [0.5, 0.5], &mut work)
            .err()
            .unwrap()
            .cause,
        TwoLoopRegionCause::Winding { .. }
    ));
    assert_eq!(work.winding_bisections, 0);
}
#[test]
fn owned_winding_evidence_is_send_and_sync() {
    fn assert_owned<T: Send + Sync>() {}
    assert_owned::<winding::TwoLoopWindingProof>();
}
