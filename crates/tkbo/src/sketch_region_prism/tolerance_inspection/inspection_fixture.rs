// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    PrismImageRole, PrismReceipt,
    region_certificate::tests::{fixture, run},
};
use super::*;
use crate::sketch_region_contact_transport::PairedContactTransaction;
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::{
    brep::access::{edge_tshape, face_tshape, vertex_tshape},
    topods::Shape,
};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) struct InspectionFixture {
    pub bridge: PairedContactTransaction,
    pub sources: Vec<Shape>,
    pub receipt: PrismReceipt,
}
impl InspectionFixture {
    pub(super) fn new() -> Self {
        let (bridge, sources) = fixture();
        let receipt = run(&bridge, &sources);
        Self {
            bridge,
            sources,
            receipt,
        }
    }
    pub(super) fn inspect(
        &self,
        request: PrismToleranceRequest,
        progress: &mut impl ToleranceInspectionProgress,
    ) -> Result<PrismToleranceInspection, ToleranceInspectionFailure> {
        inspect_prism_tolerances(
            &self.bridge,
            &binding_sources(&self.sources),
            &self.receipt,
            request,
            progress,
        )
    }
    pub(super) fn captured(&self, request: PrismToleranceRequest) -> PrismToleranceInspection {
        self.inspect(request, &mut ContinueInspection)
            .ok()
            .expect("authored evidence capture")
    }
    pub(super) fn actual(&self, kind: ShapeEnum) -> Shape {
        self.receipt
            .diagnostic
            .as_ref()
            .unwrap()
            .bindings
            .iter()
            .find(|row| {
                row.native.role == PrismImageRole::Last
                    && row.context.occurrence.shape_type().ok() == Some(kind)
            })
            .unwrap()
            .context
            .occurrence
            .clone()
    }
}
pub(super) struct ContinueInspection;
impl ToleranceInspectionProgress for ContinueInspection {
    fn cancelled(&mut self, _: ToleranceInspectionPhase) -> bool {
        false
    }
}
pub(super) fn request(cap: StoredToleranceCap, budget: f64) -> PrismToleranceRequest {
    PrismToleranceRequest::new(cap, budget).unwrap()
}
pub(super) fn ordinary_request() -> PrismToleranceRequest {
    request(StoredToleranceCap::refuse(1e-6).unwrap(), 0.)
}
pub(super) fn set_raw(actual: &Shape, value: f64) {
    match actual.shape_type().unwrap() {
        ShapeEnum::Vertex => vertex_tshape(actual, "tolerance fixture")
            .unwrap()
            .set_tolerance(value),
        ShapeEnum::Edge => edge_tshape(actual, "tolerance fixture")
            .unwrap()
            .set_tolerance(value),
        ShapeEnum::Face => face_tshape(actual, "tolerance fixture")
            .unwrap()
            .set_tolerance(value),
        _ => panic!("expected vertex/edge/face metadata fixture"),
    }
}
