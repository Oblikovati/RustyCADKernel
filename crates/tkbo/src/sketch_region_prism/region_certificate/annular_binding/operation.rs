// SPDX-License-Identifier: GPL-2.0-only
//! Final callback-free source and body rebind is required before private binding issuance.
use super::super::current_binding::{PrismBindingFailure, rebind_prism_selection};
use super::*;

pub(in crate::sketch_region_prism) fn bind_annular_prism_v2<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    region: Box<TwoLoopRegionCertificate>,
    receipt: &PrismReceipt,
    progress: &mut dyn AnnularBindingProgress,
) -> AnnularCheck<'a, AnnularPrismBinding> {
    poll(progress, AnnularBindingStage::Admission)?;
    region_rebind(request, &region)?;
    let operands = capture(request, &region, receipt, progress)?;
    let replay = replay::encode(&region, &operands)?;
    poll(progress, AnnularBindingStage::Commit)?;
    let binding = AnnularPrismBinding {
        region,
        operands,
        replay,
    };
    rebind::rebind_annular_prism_v2(request, receipt, &binding)?;
    Ok(binding)
}
pub(super) fn capture<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    region: &TwoLoopRegionCertificate,
    receipt: &PrismReceipt,
    progress: &mut dyn AnnularBindingProgress,
) -> AnnularCheck<'a, AnnularPrismOperands> {
    let limits = settings::capture(request, receipt)?;
    selection(request, receipt, progress)?;
    poll(progress, AnnularBindingStage::Copy)?;
    let copy = capture::copy_inputs(request, region, receipt)?;
    let boundaries = boundaries::capture(region, &copy, receipt)?;
    let signed_jacobian = settings::jacobian(&copy, &limits)?;
    poll(progress, AnnularBindingStage::Charts)?;
    let (charts, body_contexts) = chart_capture(receipt, &copy, &boundaries, progress)?;
    Ok(AnnularPrismOperands {
        copy,
        boundaries,
        charts,
        body_contexts,
        limits,
        signed_jacobian,
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
) -> AnnularCheck<'static> {
    match progress.checkpoint(stage) {
        Ok(false) => Ok(()),
        Ok(true) => Err(Box::new(AnnularBindingCause::Cancelled(stage))),
        Err(cause) => Err(Box::new(AnnularBindingCause::Callback { stage, cause })),
    }
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
    receipt: &PrismReceipt,
    copy: &AnnularCopyCapture,
    boundaries: &[AnnularBoundaryBinding],
    progress: &mut dyn AnnularBindingProgress,
) -> AnnularCheck<'static, (Vec<AnnularBoundChart>, Vec<RegionObservedContext>)> {
    let mut callback = None;
    let result = charts::capture(receipt, copy, boundaries, &mut || {
        stop(progress, AnnularBindingStage::Charts, &mut callback)
    });
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
    if let Err(cause) = poll(progress, stage) {
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
