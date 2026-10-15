// SPDX-License-Identifier: GPL-2.0-only
//! Publication bindings: originals by certified two-loop role, sources by authored boundary role,
//! directed output uses by sweep kind, trim role and native endpoint role.
use super::*;
use crate::sketch_region::OriginalMemberIdentity;
use crate::sketch_region_prism::region_certificate::annular_binding::{
    AnnularNativeEndpoint, snapshot_annular_context,
};
use crate::sketch_region_prism::region_certificate::two_loop::{
    TwoLoopMemberRole, TwoLoopOriginalUse, map_two_loop_original_parents, rebind_two_loop_region,
};
use crate::sketch_region_prism::surface_charts::PrismTrimRole;
use crate::sketch_region_prism::{PrismBodyContext, contexts::body_contexts, history::all_parents};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{edge, vertex},
};

/// Issue the publication bindings over a current region; `issuer.publication(region, receipt, binding)`.
pub(super) fn publication<'a>(
    issuer: &mut AnnularNameIssuer,
    region: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    binding: &AnnularPrismBinding,
) -> NameCheck<'a, PrismPublicationBindings> {
    rebind_two_loop_region(region, binding.region()).map_err(AnnularNameCause::Region)?;
    let originals = originals(issuer, region, receipt)?;
    let sources = sources(issuer, binding)?;
    let directed = directed(issuer, receipt, binding)?;
    rebind_two_loop_region(region, binding.region()).map_err(AnnularNameCause::Region)?;
    Ok(PrismPublicationBindings {
        operation: issuer.operation()?,
        selected_region: issuer.annulus()?,
        originals,
        sources,
        directed,
    })
}
fn originals<'a>(
    issuer: &mut AnnularNameIssuer,
    region: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
) -> NameCheck<'a, Vec<PrismOriginalBinding>> {
    let originals = original_rows(receipt)?;
    let slices: Vec<_> = originals.iter().map(std::slice::from_ref).collect();
    let mapped =
        map_two_loop_original_parents(region, &slices).map_err(AnnularNameCause::Region)?;
    let mut result = Vec::new();
    for (actual, uses) in originals.into_iter().zip(mapped.rows) {
        count("one certified role per original", uses.len(), 1)?;
        let (member, directed_use) = original_names(issuer, uses[0])?;
        result.push(PrismOriginalBinding {
            actual,
            member,
            directed_use,
        });
    }
    Ok(result)
}
/// Every distinct original parent of copied and rejected relations, in relation order.
fn original_rows<'a>(receipt: &PrismReceipt) -> NameCheck<'a, Vec<OriginalMemberIdentity>> {
    let copied = receipt.copied.as_ref().ok_or(AnnularNameCause::Missing {
        required: "copied selected relations",
    })?;
    let rejected = receipt.input.rejected.iter().flatten();
    let mut result = Vec::new();
    for row in copied.relations.iter().chain(rejected) {
        for parent in all_parents(row) {
            if !result.contains(&parent) {
                result.push(parent);
            }
        }
    }
    Ok(result)
}
fn original_names<'a>(
    issuer: &mut AnnularNameIssuer,
    key: TwoLoopOriginalUse,
) -> NameCheck<'a, (AuthoredPrismId, AuthoredPrismId)> {
    let (kind, variant) = match key.role {
        TwoLoopMemberRole::Boundary => (ShapeEnum::Edge, 1),
        TwoLoopMemberRole::NativeStart => (ShapeEnum::Vertex, 2),
        TwoLoopMemberRole::NativeEnd => (ShapeEnum::Vertex, 3),
        role => return Err(AnnularNameCause::OriginalRole { role }),
    };
    let member = issuer.boundary_member(key.source, kind)?;
    let directed = issuer.use_role(AnnularNameCategory::OriginalUse, kind, variant)?;
    Ok((member, directed))
}
fn sources<'a>(
    issuer: &mut AnnularNameIssuer,
    binding: &AnnularPrismBinding,
) -> NameCheck<'a, Vec<PrismSourceBinding>> {
    let mut result = vec![PrismSourceBinding {
        actual: binding.copied_sources().face.clone(),
        member: issuer.annulus()?,
    }];
    for boundary in binding.boundaries() {
        let start = native_start(&boundary.endpoints)?;
        for (actual, kind) in [
            (&boundary.copied_wire, ShapeEnum::Wire),
            (&boundary.copied, ShapeEnum::Edge),
            (&start.context, ShapeEnum::Vertex),
        ] {
            result.push(PrismSourceBinding {
                actual: actual.occurrence.clone(),
                member: issuer.boundary_member(boundary.source, kind)?,
            });
        }
    }
    Ok(result)
}
fn native_start<'a>(
    endpoints: &[AnnularNativeEndpoint],
) -> NameCheck<'a, &AnnularNativeEndpoint> {
    let found: Vec<_> = endpoints.iter().filter(|e| e.parameter == 0.).collect();
    count("one native start endpoint at parameter 0", found.len(), 1)?;
    Ok(found[0])
}
fn directed<'a>(
    issuer: &mut AnnularNameIssuer,
    receipt: &PrismReceipt,
    binding: &AnnularPrismBinding,
) -> NameCheck<'a, Vec<PrismDirectedBinding>> {
    let diagnostic = receipt
        .diagnostic
        .as_ref()
        .ok_or(AnnularNameCause::Missing {
            required: "retained sweep body",
        })?;
    let contexts = body_contexts(&diagnostic.body).map_err(AnnularNameCause::Body)?;
    let mut result = Vec::new();
    for actual in contexts {
        let authored_use = directed_name(issuer, &actual, binding)?;
        result.push(PrismDirectedBinding {
            actual,
            authored_use,
        });
    }
    Ok(result)
}
fn directed_name<'a>(
    issuer: &mut AnnularNameIssuer,
    actual: &PrismBodyContext,
    binding: &AnnularPrismBinding,
) -> NameCheck<'a, AuthoredPrismId> {
    let kind = actual
        .occurrence
        .shape_type()
        .map_err(AnnularNameCause::Native)?;
    let variant = match kind {
        ShapeEnum::Edge => trim_role(actual, binding)?,
        ShapeEnum::Vertex => endpoint_role(actual)?,
        _ => 0,
    };
    issuer.use_role(AnnularNameCategory::OutputUse, kind, variant)
}
fn trim_role<'a>(actual: &PrismBodyContext, binding: &AnnularPrismBinding) -> NameCheck<'a, u8> {
    let observed = snapshot_annular_context(actual).map_err(AnnularNameCause::Binding)?;
    let rows: Vec<_> = binding
        .charts()
        .iter()
        .flat_map(|c| &c.trims)
        .filter(|t| t.output == observed)
        .collect();
    count("one bound trim per output edge use", rows.len(), 1)?;
    Ok(match rows[0].role {
        PrismTrimRole::First => 1,
        PrismTrimRole::Last => 2,
        PrismTrimRole::Start => 3,
        PrismTrimRole::End => 4,
    })
}
/// The exact native range endpoint the vertex lies at: 1 for start, 2 for end.
fn endpoint_role<'a>(actual: &PrismBodyContext) -> NameCheck<'a, u8> {
    let parent = actual.ancestors.last().ok_or(AnnularNameCause::Missing {
        required: "edge ancestor of an output vertex use",
    })?;
    let edge = edge(parent).map_err(AnnularNameCause::Native)?;
    let vertex = vertex(&actual.occurrence).map_err(AnnularNameCause::Native)?;
    let parameter = BRepTool::parameter(&vertex, &edge).map_err(AnnularNameCause::Native)?;
    let (start, end) = BRepTool::range(&edge).map_err(AnnularNameCause::Native)?;
    match (parameter == start, parameter == end) {
        (true, false) => Ok(1),
        (false, true) => Ok(2),
        _ => Err(AnnularNameCause::EndpointRole {
            parameter,
            range: [start, end],
        }),
    }
}
