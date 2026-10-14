// SPDX-License-Identifier: GPL-2.0-only
//! Final callback-free source and body rebind is required before private binding issuance.
use super::super::current_binding::{PrismBindingFailure, rebind_prism_selection};
use super::*;

pub(in crate::sketch_region_prism) fn bind_annular_prism_v2<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    region: Box<TwoLoopRegionCertificate>,
    receipt: &PrismReceipt,
    progress: &mut dyn AnnularBindingProgress,
) -> AnnularBindingResult<'a, AnnularPrismBinding> {
    let mut tracker = ProgressTracker {
        progress,
        stage: AnnularBindingStage::Admission,
    };
    bind(request, region, receipt, &mut tracker)
        .map_err(|cause| attributed(request, receipt, tracker.stage, cause))
}
fn bind<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    region: Box<TwoLoopRegionCertificate>,
    receipt: &PrismReceipt,
    progress: &mut dyn AnnularBindingProgress,
) -> AnnularCheck<'a, AnnularPrismBinding> {
    poll(progress, AnnularBindingStage::Admission, request, receipt)?;
    region_rebind(request, &region)?;
    let operands = capture(request, &region, receipt, progress)?;
    let replay = replay::encode(&region, &operands)?;
    poll(progress, AnnularBindingStage::Commit, request, receipt)?;
    let binding = AnnularPrismBinding {
        region,
        operands,
        replay,
    };
    rebind::check(request, receipt, &binding)?;
    Ok(binding)
}
pub(super) fn capture<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    region: &TwoLoopRegionCertificate,
    receipt: &PrismReceipt,
    progress: &mut dyn AnnularBindingProgress,
) -> AnnularCheck<'a, AnnularPrismOperands> {
    let hierarchy = bounds::inspect(receipt, request.requested.maximum_retained_units)?;
    let metadata_units = metadata_budget::inspect(
        receipt,
        hierarchy.retained_units,
        request.requested.maximum_retained_units,
    )?;
    let reserved = geometry_budget::inspect(request, receipt, metadata_units)?;
    let limits = settings::capture(request, receipt)?;
    selection(request, receipt, progress)?;
    poll(progress, AnnularBindingStage::Copy, request, receipt)?;
    let copy = capture::copy_inputs(request, region, receipt)?;
    let boundaries = boundaries::capture(region, &copy, receipt)?;
    let signed_jacobian = settings::jacobian(&copy, &limits)?;
    poll(progress, AnnularBindingStage::Charts, request, receipt)?;
    let (charts, body_contexts) = chart_capture(request, receipt, &copy, &boundaries, progress)?;
    let parents = parents::capture(request, receipt, &copy, reserved, &charts)?;
    Ok(AnnularPrismOperands {
        copy,
        boundaries,
        charts,
        body_contexts,
        limits,
        signed_jacobian,
        parents,
        hierarchy,
        metadata_units,
        geometry_units: reserved,
    })
}
pub(super) fn region_rebind<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    region: &TwoLoopRegionCertificate,
) -> AnnularCheck<'a> {
    rebind_two_loop_region(request, region)
        .map_err(|cause| Box::new(AnnularBindingCause::Region(cause)))
}
pub(super) fn poll(
    progress: &mut dyn AnnularBindingProgress,
    stage: AnnularBindingStage,
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
) -> AnnularCheck<'static> {
    poll_owned(progress, stage)?;
    preflight(request, receipt)
}
fn poll_owned(
    progress: &mut dyn AnnularBindingProgress,
    stage: AnnularBindingStage,
) -> AnnularCheck<'static> {
    match progress.checkpoint(stage) {
        Ok(false) => Ok(()),
        Ok(true) => Err(Box::new(AnnularBindingCause::Cancelled(stage))),
        Err(cause) => Err(Box::new(AnnularBindingCause::Callback { stage, cause })),
    }
}
pub(super) fn preflight(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
) -> AnnularCheck<'static> {
    let hierarchy = bounds::inspect(receipt, request.requested.maximum_retained_units)?;
    let reserved = metadata_budget::inspect(
        receipt,
        hierarchy.retained_units,
        request.requested.maximum_retained_units,
    )?;
    geometry_budget::inspect(request, receipt, reserved).map(|_| ())
}
fn selection(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    progress: &mut dyn AnnularBindingProgress,
) -> AnnularCheck<'static> {
    let mut callback = None;
    let result = rebind_prism_selection(request.bridge, receipt, &mut || {
        stop(progress, AnnularBindingStage::Admission, &mut callback)
    });
    finish_callback(
        result.map_err(|cause| Box::new(AnnularBindingCause::Chart(Box::new(cause)))),
        callback,
    )
}
fn chart_capture(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    copy: &AnnularCopyCapture,
    boundaries: &[AnnularBoundaryBinding],
    progress: &mut dyn AnnularBindingProgress,
) -> AnnularCheck<'static, (Vec<AnnularBoundChart>, Vec<RegionObservedContext>)> {
    let mut callback = None;
    let result = charts::capture_guarded(
        receipt,
        copy,
        boundaries,
        &mut || stop(progress, AnnularBindingStage::Charts, &mut callback),
        &mut || preflight(request, receipt),
    );
    finish_callback(result, callback)
}
fn stop(
    progress: &mut dyn AnnularBindingProgress,
    stage: AnnularBindingStage,
    callback: &mut Option<Box<AnnularBindingCause<'static>>>,
) -> bool {
    if callback.is_some() {
        return true;
    }
    if let Err(cause) = poll_owned(progress, stage) {
        *callback = Some(cause);
    }
    callback.is_some()
}
fn finish_callback<T>(
    result: AnnularCheck<'static, T>,
    callback: Option<Box<AnnularBindingCause<'static>>>,
) -> AnnularCheck<'static, T> {
    if let Some(callback) = callback {
        return Err(Box::new(AnnularBindingCause::Stopped {
            callback,
            observed: result.err(),
        }));
    }
    result
}
pub(super) struct CallbackFree;
impl AnnularBindingProgress for CallbackFree {
    fn checkpoint(
        &mut self,
        _: AnnularBindingStage,
    ) -> Result<bool, rustycad_tkernel::standard::Failure> {
        Ok(false)
    }
}
struct ProgressTracker<'p> {
    progress: &'p mut dyn AnnularBindingProgress,
    stage: AnnularBindingStage,
}
impl AnnularBindingProgress for ProgressTracker<'_> {
    fn checkpoint(
        &mut self,
        stage: AnnularBindingStage,
    ) -> Result<bool, rustycad_tkernel::standard::Failure> {
        self.stage = stage;
        self.progress.checkpoint(stage)
    }
}
