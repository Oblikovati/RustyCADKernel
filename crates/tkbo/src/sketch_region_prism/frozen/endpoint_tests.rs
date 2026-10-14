// SPDX-License-Identifier: GPL-2.0-only
//! Endpoint evidence must use current actual vertex values, including commit callback changes.
use super::{tests::*, *};
use rustycad_tkbrep::{brep::access::vertex_tshape, topods::Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
struct ChangeVertexAtCheckpoint {
    phase: FreezePhase,
    actual: Shape,
}
impl FreezeProgress for ChangeVertexAtCheckpoint {
    fn checkpoint(&mut self, phase: FreezePhase) -> Result<bool, Failure> {
        if phase == self.phase {
            vertex_tshape(&self.actual, "owned endpoint mutation")?
                .set_pnt(Point3::new(1., 0., 0.));
        }
        Ok(false)
    }
}
fn actual_vertex(receipt: &super::super::PrismReceipt) -> Shape {
    super::super::contexts::body_contexts(&receipt.diagnostic.as_ref().unwrap().body)
        .ok()
        .unwrap()
        .into_iter()
        .find(|c| c.occurrence.shape_type().ok() == Some(ShapeEnum::Vertex))
        .unwrap()
        .occurrence
}
fn refuse_moved_endpoint(phase: FreezePhase) {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let before = captured(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress));
    let bytes = before.replay_bytes();
    let mut callback = ChangeVertexAtCheckpoint {
        phase,
        actual: actual_vertex(&receipt),
    };
    let vertex = super::error_context_tests::actual_key(&receipt, &bindings, &callback.actual);
    let result = freeze_prism(&receipt, &bindings, &mut callback);
    super::error_context_tests::assert_endpoint_context(result, &before, vertex);
    assert_eq!(before.replay_bytes(), bytes);
}
#[test]
fn actual_vertex_changes_fail_fresh_endpoint_evidence() {
    for phase in [FreezePhase::Incidence, FreezePhase::Commit] {
        refuse_moved_endpoint(phase);
    }
}
