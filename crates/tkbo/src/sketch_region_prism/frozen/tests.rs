// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    tests::{cubic_receipt, progress},
    *,
};
use super::*;
#[test]
fn missing_authored_bindings_never_get_invented_ids() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let bindings = PrismPublicationBindings {
        operation: AuthoredPrismId(700),
        selected_region: AuthoredPrismId(701),
        originals: vec![],
        sources: vec![],
        directed: vec![],
    };
    let result = freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress);
    assert!(matches!(result, PrismFreezeOutcome::Incomplete(_)));
}
struct NamedFreezeProgress;
impl FreezeProgress for NamedFreezeProgress {
    fn checkpoint(&mut self, _: FreezePhase) -> Result<bool, rustycad_tkernel::standard::Failure> {
        Ok(false)
    }
}

fn authored_bindings(
    receipt: &PrismReceipt,
    source: &rustycad_tkbrep::topods::Shape,
) -> PrismPublicationBindings {
    let mut bindings = PrismPublicationBindings {
        operation: AuthoredPrismId(700),
        selected_region: AuthoredPrismId(701),
        originals: Vec::new(),
        sources: Vec::new(),
        directed: Vec::new(),
    };
    let copied = receipt.copied.as_ref().unwrap();
    for row in &copied.relations {
        bind_authored_source(&mut bindings, row, source);
    }
    for context in super::super::contexts::body_contexts(&receipt.diagnostic.as_ref().unwrap().body)
        .ok()
        .unwrap()
    {
        let authored_use = authored_use(&context);
        bindings.directed.push(PrismDirectedBinding {
            actual: context,
            authored_use,
        });
    }
    bindings
}
fn bind_authored_source(
    bindings: &mut PrismPublicationBindings,
    row: &crate::sketch_region::ArrangementRelation,
    source: &rustycad_tkbrep::topods::Shape,
) {
    let kind = row.output.shape_type().unwrap();
    let member = authored_member(kind);
    bindings.sources.push(PrismSourceBinding {
        actual: crate::sketch_region::PartitionOccurrenceIdentity::capture(&row.output),
        member,
    });
    for original in super::super::history::all_parents(row) {
        if bindings.originals.iter().any(|b| b.actual == original) {
            continue;
        }
        let endpoint = original.orientation != rustycad_tkg3d::top_abs::Orientation::Forward;
        // Caller-authored source shape is a single closed cubic: its edge and endpoint roles are declared.
        let is_edge = original.topology_address == source.tshape().object_address().addr();
        let (member, use_id) = if is_edge {
            (703, 711)
        } else {
            (704, if endpoint { 713 } else { 712 })
        };
        bindings.originals.push(PrismOriginalBinding {
            actual: original,
            member: AuthoredPrismId(member),
            directed_use: AuthoredPrismId(use_id),
        });
    }
}
fn authored_member(kind: rustycad_tkg3d::top_abs::ShapeEnum) -> AuthoredPrismId {
    use rustycad_tkg3d::top_abs::ShapeEnum::*;
    AuthoredPrismId(match kind {
        Face => 701,
        Wire => 702,
        Edge => 703,
        Vertex => 704,
        _ => panic!("unsupported authored fixture member"),
    })
}
fn authored_use(context: &PrismBodyContext) -> AuthoredPrismId {
    use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum::*};
    let reverse = context.occurrence.orientation() == Orientation::Reversed;
    AuthoredPrismId(match context.occurrence.shape_type().unwrap() {
        Solid => 720,
        Shell => 721,
        Face => 722,
        Wire => 710,
        Edge => {
            if reverse {
                713
            } else {
                712
            }
        }
        Vertex => {
            if reverse {
                715
            } else {
                714
            }
        }
        _ => panic!("unsupported authored fixture use"),
    })
}
#[test]
fn complete_owned_capture_keeps_independent_incidence_counts() {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let result = freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress);
    let PrismFreezeOutcome::Captured(capture) = result else {
        match result {
            PrismFreezeOutcome::Incomplete(f) | PrismFreezeOutcome::Failed(f) => panic!("{f:?}"),
            _ => panic!("cancelled"),
        }
    };
    assert_eq!(capture.graph.counts, [2, 3, 6, 3, 3, 1, 1]);
    assert_eq!(capture.faces().len(), 3);
    assert!(!capture.is_valid_body());
}

fn authored_receipt() -> (rustycad_tkbrep::topods::Shape, PrismReceipt) {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let batch = crate::sketch_region_candidates::tests::candidates(std::slice::from_ref(&source));
    let receipt = retain_selected_prism(
        batch,
        super::super::tests::request([1., 1.]),
        &mut progress(None, None),
    );
    (source, receipt)
}
