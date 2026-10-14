// SPDX-License-Identifier: GPL-2.0-only
//! Actual child paths supply context orientation; native returned images remain separate keys.
use super::{copying::mapped, outcome::*};
use rustycad_tkbrep::topods::Shape;
pub(super) fn body_contexts(body: &Shape) -> Result<Vec<PrismBodyContext>, PrismCause> {
    crate::sketch_region::strict_region_contexts(body).map_err(|cause| match cause {
        crate::sketch_region::RegionContextFailure::Native(failure) => {
            mapped(PrismPhase::History, failure)
        }
        crate::sketch_region::RegionContextFailure::RepeatedAncestor { member } => {
            PrismCause::CyclicBodyContext { member }
        }
    })
}
/// Retain source-backed image/context transport; `bind_observed_images(rows,contexts)`.
pub(super) fn bind_observed_images(
    observed: Vec<PrismNativeImage>,
    contexts: &[PrismBodyContext],
) -> (Vec<PrismImageBinding>, Vec<PrismGap>) {
    let mut bindings = Vec::new();
    let mut gaps = Vec::new();
    for image in observed {
        let before = bindings.len();
        bind_image(&image, contexts, &mut bindings, &mut gaps);
        if before == bindings.len() {
            gaps.push(PrismGap::MissingImageContext(image));
        }
    }
    (bindings, gaps)
}
fn bind_image(
    image: &PrismNativeImage,
    contexts: &[PrismBodyContext],
    bindings: &mut Vec<PrismImageBinding>,
    gaps: &mut Vec<PrismGap>,
) {
    for context in contexts
        .iter()
        .filter(|context| context.occurrence.is_same(&image.returned.output))
    {
        record_binding(image, context, contexts, bindings, gaps);
    }
}

fn verified_binding(
    image: &PrismNativeImage,
    context: &PrismBodyContext,
) -> Option<PrismImageBinding> {
    // NumLinearRegularSweep Add applies observed containment orientation independently of cached images.
    let transported = image
        .returned
        .output
        .oriented(context.occurrence.orientation());
    if !transported.is_equal(&context.occurrence) {
        return None;
    }
    Some(PrismImageBinding {
        native: image.clone(),
        context: context.clone(),
        transported,
    })
}

fn conflicting_paths(
    context: &PrismBodyContext,
    contexts: &[PrismBodyContext],
) -> Vec<PrismBodyContext> {
    contexts
        .iter()
        .filter(|candidate| {
            candidate.child_slots == context.child_slots
                && (!candidate.occurrence.is_equal(&context.occurrence)
                    || !same_ancestry(&candidate.ancestors, &context.ancestors))
        })
        .cloned()
        .collect()
}
fn same_ancestry(first: &[Shape], second: &[Shape]) -> bool {
    first.len() == second.len()
        && first
            .iter()
            .zip(second)
            .all(|(left, right)| left.is_equal(right))
}

fn record_binding(
    image: &PrismNativeImage,
    context: &PrismBodyContext,
    contexts: &[PrismBodyContext],
    bindings: &mut Vec<PrismImageBinding>,
    gaps: &mut Vec<PrismGap>,
) {
    let conflicting = conflicting_paths(context, contexts);
    if !conflicting.is_empty() {
        gaps.push(PrismGap::AmbiguousContextPath {
            image: image.clone(),
            contexts: conflicting,
        });
        return;
    }
    record_verified(image, context, bindings, gaps);
}

fn record_verified(
    image: &PrismNativeImage,
    context: &PrismBodyContext,
    bindings: &mut Vec<PrismImageBinding>,
    gaps: &mut Vec<PrismGap>,
) {
    match verified_binding(image, context) {
        Some(binding) => bindings.push(binding),
        None => gaps.push(PrismGap::UnverifiedImageContext {
            image: image.clone(),
            context: context.clone(),
        }),
    }
}
