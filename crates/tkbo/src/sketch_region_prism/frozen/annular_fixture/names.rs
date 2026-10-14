// SPDX-License-Identifier: GPL-2.0-only
//! These fixed semantic reservations are test input, never enumeration-derived publication names.
use super::*;
use crate::sketch_region::{OriginalMemberIdentity, RegionSourceId};
use crate::sketch_region_prism::{PrismBodyContext, PrismReceipt, contexts::body_contexts, history::all_parents};
use rustycad_tkbrep::{brep::BRepTool, topods::{edge, vertex}};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn authored(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    binding: &AnnularPrismBinding,
) -> PrismPublicationBindings {
    rebind_two_loop_region(request, binding.region()).unwrap_or_else(|_| panic!("current region"));
    let originals = original_names(request, receipt);
    let sources = source_names(binding);
    let directed = body_contexts(&receipt.diagnostic.as_ref().unwrap().body).ok().unwrap()
        .into_iter().map(|actual| PrismDirectedBinding {
            authored_use: directed_name(&actual, binding), actual,
        }).collect();
    rebind_two_loop_region(request, binding.region()).unwrap_or_else(|_| panic!("unchanged region"));
    PrismPublicationBindings {
        operation: AuthoredPrismId(77), selected_region: AuthoredPrismId(301),
        originals, sources, directed,
    }
}
fn original_names(request: &TwoLoopRegionRequest<'_, '_>, receipt: &PrismReceipt) -> Vec<PrismOriginalBinding> {
    let originals = original_rows(receipt);
    let slices: Vec<_> = originals.iter().map(std::slice::from_ref).collect();
    let mapped = map_two_loop_original_parents(request, &slices)
        .unwrap_or_else(|_| panic!("every original has an authored native role"));
    originals.into_iter().zip(mapped.rows).map(|(actual, names)| {
        assert_eq!(names.len(), 1);
        let (member, directed_use) = original_name(names[0]);
        PrismOriginalBinding { actual, member: AuthoredPrismId(member), directed_use: AuthoredPrismId(directed_use) }
    }).collect()
}
fn original_rows(receipt: &PrismReceipt) -> Vec<OriginalMemberIdentity> {
    let mut result = Vec::new();
    for row in receipt.copied.as_ref().unwrap().relations.iter()
        .chain(receipt.input.rejected.iter().flatten()) {
        for parent in all_parents(row) {
            if !result.contains(&parent) { result.push(parent); }
        }
    }
    result
}
fn original_name(key: TwoLoopOriginalUse) -> (u64, u64) {
    let [_, edge, vertex] = loop_names(key.source);
    match key.role {
        TwoLoopMemberRole::Boundary => (edge, 931),
        TwoLoopMemberRole::NativeStart => (vertex, 932),
        TwoLoopMemberRole::NativeEnd => (vertex, 933),
        _ => panic!("unexpected original role {:?}; expected edge or native endpoint", key.role),
    }
}
fn loop_names(source: RegionSourceId) -> [u64; 3] {
    match source.0 {
        101 => [401, 501, 601],
        202 => [402, 502, 602],
        actual => panic!("source {actual}; expected authored outer101 or inner202"),
    }
}
fn source_names(binding: &AnnularPrismBinding) -> Vec<PrismSourceBinding> {
    let mut result = vec![PrismSourceBinding {
        actual: binding.copied_sources().face.clone(), member: AuthoredPrismId(301),
    }];
    for boundary in binding.boundaries() {
        let names = loop_names(boundary.source);
        let vertex = boundary.endpoints.iter().find(|e| e.parameter == 0.).unwrap();
        for (actual, member) in [(&boundary.copied_wire, names[0]), (&boundary.copied, names[1]), (&vertex.context, names[2])] {
            result.push(PrismSourceBinding { actual: actual.occurrence.clone(), member: AuthoredPrismId(member) });
        }
    }
    result
}
fn directed_name(actual: &PrismBodyContext, binding: &AnnularPrismBinding) -> AuthoredPrismId {
    AuthoredPrismId(match actual.occurrence.shape_type().unwrap() {
        ShapeEnum::Solid => 901,
        ShapeEnum::Shell => 902,
        ShapeEnum::Face => 903,
        ShapeEnum::Wire => 904,
        ShapeEnum::Edge => trim_name(actual, binding),
        ShapeEnum::Vertex => endpoint_name(actual),
        kind => panic!("actual kind {kind:?}; expected solid through vertex"),
    })
}
fn trim_name(actual: &PrismBodyContext, binding: &AnnularPrismBinding) -> u64 {
    use crate::sketch_region_prism::surface_charts::PrismTrimRole;
    let observed = snapshot_annular_context(actual).unwrap_or_else(|_| panic!("full fixture context"));
    let rows: Vec<_> = binding.charts().iter().flat_map(|c| &c.trims)
        .filter(|t| t.output == observed).collect();
    assert_eq!(rows.len(), 1);
    match rows[0].role {
        PrismTrimRole::First => 910,
        PrismTrimRole::Last => 911,
        PrismTrimRole::Start => 912,
        PrismTrimRole::End => 913,
    }
}
fn endpoint_name(actual: &PrismBodyContext) -> u64 {
    let edge = edge(actual.ancestors.last().unwrap()).unwrap();
    let vertex = vertex(&actual.occurrence).unwrap();
    let parameter = BRepTool::parameter(&vertex, &edge).unwrap();
    let (start, end) = BRepTool::range(&edge).unwrap();
    assert_ne!(start, end);
    if parameter == start { return 920; }
    assert_eq!(parameter, end);
    921
}
