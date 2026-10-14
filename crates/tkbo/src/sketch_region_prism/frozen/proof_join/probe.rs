// SPDX-License-Identifier: GPL-2.0-only
//! Initial raw operands precede callbacks; a rounded world point cannot hide later mutation.
use super::*;
use crate::sketch_region::{ArrangementSource, PartitionOccurrenceIdentity};
use crate::sketch_region_contact_transport::PairedContactTransaction;
use crate::sketch_region_prism::{
    PrismReceipt,
    region_certificate::{ClosurePointStage, current_closure_vertices},
};

pub(super) struct InitialRawProbe {
    contexts: Vec<(ClosurePointStage, PartitionOccurrenceIdentity)>,
    bytes: Vec<u8>,
}
pub(super) fn capture(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    bindings: &PrismPublicationBindings,
) -> Result<InitialRawProbe, PrismProofJoinCause> {
    admit_shapes(bridge, sources)?;
    let vertices = current_closure_vertices(bridge, sources, receipt)
        .map_err(|cause| PrismProofJoinCause::ClosureRegion(Box::new(cause)))?;
    let mut contexts = Vec::new();
    let mut bytes = Vec::new();
    for (stage, shape) in vertices {
        let key = PartitionOccurrenceIdentity::capture(&shape);
        replay_sources::occurrence(&key, &mut bytes).map_err(link)?;
        encoding::numbers(raw_vertices::local_point(&shape).map_err(link)?, &mut bytes);
        contexts.push((stage, key));
    }
    let bound = bind_projection(receipt, bindings).map_err(link)?;
    raw_vertices::append(
        &raw_vertices::capture_bound(&bound).map_err(link)?,
        &mut bytes,
    );
    Ok(InitialRawProbe { contexts, bytes })
}
fn admit_shapes(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
) -> Result<(), PrismProofJoinCause> {
    let batch = bridge
        .mapped
        .copied
        .as_ref()
        .and_then(|value| value.completion.as_ref().ok())
        .ok_or_else(|| link(missing("current paired closure candidates")))?;
    for (name, actual, expected) in [
        ("one actual closure source", sources.len(), 1),
        (
            "one actual closure census segment",
            bridge.census.inputs.segments.len(),
            1,
        ),
        ("two actual closure candidates", batch.candidates.len(), 2),
    ] {
        if actual != expected {
            return Err(link(PrismFreezeCause::Count {
                entity: name,
                actual,
                expected,
            }));
        }
    }
    Ok(())
}
pub(super) fn same(
    initial: &InitialRawProbe,
    current: &InitialRawProbe,
) -> Result<(), PrismProofJoinCause> {
    if initial.contexts != current.contexts || initial.bytes != current.bytes {
        return Err(PrismProofJoinCause::Stale {
            representation: "raw source/body vertex operands changed during callbacks",
        });
    }
    Ok(())
}
fn link(cause: PrismFreezeCause) -> PrismProofJoinCause {
    PrismProofJoinCause::Link(Box::new(cause))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::{
        gp::{Point3, Transform3, Vector3},
        toploc::Location,
    };
    #[test]
    fn upstream_raw_probe_detects_change_hidden_by_world_rounding() {
        let (bridge, mut shapes) = crate::sketch_region_prism::region_certificate::tests::fixture();
        let receipt = crate::sketch_region_prism::region_certificate::tests::run(&bridge, &shapes);
        let bindings = super::super::super::tests::authored_bindings(&receipt, &shapes[0]);
        let mut affine = Transform3::default();
        affine.set_translation(Vector3::new(9007199254740992., 0., 0.));
        shapes[0] = shapes[0].located(&Location::new(affine), true).unwrap();
        let sources = crate::sketch_region_partition_bindings::binding_sources(&shapes);
        let vertices = current_closure_vertices(&bridge, &sources, &receipt).unwrap();
        let original = &vertices[0].1;
        let before = capture(&bridge, &sources, &receipt, &bindings)
            .ok()
            .unwrap();
        let world = rustycad_tkbrep::brep::BRepTool::pnt(
            &rustycad_tkbrep::topods::vertex(original).unwrap(),
        )
        .unwrap();
        rustycad_tkbrep::brep::access::vertex_tshape(original, "owned upstream rounding trap")
            .unwrap()
            .set_pnt(Point3::new(1., 0., 0.));
        let rounded = rustycad_tkbrep::brep::BRepTool::pnt(
            &rustycad_tkbrep::topods::vertex(original).unwrap(),
        )
        .unwrap();
        assert_eq!(world.x(), rounded.x());
        let after = capture(&bridge, &sources, &receipt, &bindings)
            .ok()
            .unwrap();
        assert!(same(&before, &after).is_err());
    }
}
