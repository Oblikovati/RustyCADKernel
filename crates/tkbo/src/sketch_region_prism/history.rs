// SPDX-License-Identifier: GPL-2.0-only
//! Observe reusable sweep lists before the next call, retaining all actual source parents.
use super::{
    contexts::{bind_observed_images, body_contexts},
    copying::mapped,
    outcome::*,
};
use crate::sketch_region::{ArrangementRelation, OriginalMemberIdentity, equal_occurrences};
use crate::sketch_region_candidates::append_original_parents as append_parents;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// Retain actual generated/cap observations; `observe_sweep(&mut maker, &copied_relations)`.
pub(super) fn observe_sweep(
    maker: &mut BRepPrimAPIMakePrism,
    bases: &[ArrangementRelation],
) -> Result<PrismDiagnostic, PrismCause> {
    let body = maker
        .shape()
        .map_err(|f| mapped(PrismPhase::History, f))?
        .clone();
    let mut observed = Vec::new();
    let mut native_retired = Vec::new();
    for base in bases {
        observe_base(maker, base, &mut observed, &mut native_retired)?;
    }
    let (bindings, transport_gaps) = body_bindings(&body, observed)?;
    let relations = output_relations(&body, &bindings)?;
    let mut result = retained_diagnostic(body, relations, native_retired);
    result.bindings = bindings;
    result.transport_gaps = transport_gaps;
    Ok(result)
}
fn retained_diagnostic(
    body: Shape,
    relations: Vec<ArrangementRelation>,
    native_retired: Vec<ArrangementRelation>,
) -> PrismDiagnostic {
    PrismDiagnostic {
        body,
        bindings: Vec::new(),
        transport_gaps: Vec::new(),
        relations,
        native_retired,
        coverage: None,
    }
}

fn output_relations(
    body: &Shape,
    observed: &[PrismImageBinding],
) -> Result<Vec<ArrangementRelation>, PrismCause> {
    let mut relations = Vec::new();
    for output in equal_occurrences(body).map_err(|f| mapped(PrismPhase::History, f))? {
        relations.push(output_relation(&output, observed)?);
    }
    Ok(relations)
}
fn observe_base(
    maker: &mut BRepPrimAPIMakePrism,
    base: &ArrangementRelation,
    observed: &mut Vec<PrismNativeImage>,
    native_retired: &mut Vec<ArrangementRelation>,
) -> Result<(), PrismCause> {
    // Generated() owns one scratch list; cloning its actual values now prevents later overwrite.
    observe_generated(maker, base, observed)?;
    observe_caps(maker, base, observed)?;
    observe_retirement(maker, base, native_retired)
}
fn observe_retirement(
    maker: &mut BRepPrimAPIMakePrism,
    base: &ArrangementRelation,
    native_retired: &mut Vec<ArrangementRelation>,
) -> Result<(), PrismCause> {
    if maker
        .is_deleted(&base.output)
        .map_err(|f| mapped(PrismPhase::History, f))?
    {
        native_retired.push(base.clone());
    }
    Ok(())
}

fn observe_caps(
    maker: &mut BRepPrimAPIMakePrism,
    base: &ArrangementRelation,
    observed: &mut Vec<PrismNativeImage>,
) -> Result<(), PrismCause> {
    let first = maker
        .first_shape_of(&base.output)
        .map_err(|f| mapped(PrismPhase::History, f))?;
    let last = maker
        .last_shape_of(&base.output)
        .map_err(|f| mapped(PrismPhase::History, f))?;
    append_cap_images(base, first, last, observed);
    Ok(())
}
fn all_parents(base: &ArrangementRelation) -> Vec<OriginalMemberIdentity> {
    let mut parents = Vec::new();
    for kind in [&base.preserved, &base.modified, &base.generated] {
        append_parents(&mut parents, kind);
    }
    parents
}
fn generated_relation(output: &Shape, base: &ArrangementRelation) -> ArrangementRelation {
    ArrangementRelation {
        output: output.clone(),
        preserved: Vec::new(),
        modified: Vec::new(),
        generated: all_parents(base),
    }
}
fn output_relation(
    output: &Shape,
    observed: &[PrismImageBinding],
) -> Result<ArrangementRelation, PrismCause> {
    let mut relation = empty_relation(output);
    append_matching(&mut relation, observed);
    if !all_parents(&relation).is_empty() {
        return Ok(relation);
    }
    let kind = output
        .shape_type()
        .map_err(|f| mapped(PrismPhase::History, f))?;
    if matches!(kind, ShapeEnum::Wire | ShapeEnum::Shell | ShapeEnum::Solid) {
        append_descendants(&mut relation, observed)?;
    }
    Ok(relation)
}
fn append_descendants(
    relation: &mut ArrangementRelation,
    observed: &[PrismImageBinding],
) -> Result<(), PrismCause> {
    for child in equal_occurrences(&relation.output)
        .map_err(|f| mapped(PrismPhase::History, f))?
        .iter()
        .skip(1)
    {
        let mut descendant = empty_relation(child);
        append_matching(&mut descendant, observed);
        append_parents(&mut relation.generated, &all_parents(&descendant));
    }
    Ok(())
}
fn append_matching(output: &mut ArrangementRelation, observed: &[PrismImageBinding]) {
    for binding in observed
        .iter()
        .filter(|binding| binding.context.occurrence.is_equal(&output.output))
    {
        let row = &binding.native.returned;
        append_parents(&mut output.preserved, &row.preserved);
        append_parents(&mut output.modified, &row.modified);
        append_parents(&mut output.generated, &row.generated);
    }
}
fn native_image(
    base: &ArrangementRelation,
    role: PrismImageRole,
    returned: ArrangementRelation,
) -> PrismNativeImage {
    PrismNativeImage {
        query: base.output.clone(),
        role,
        returned,
    }
}
fn empty_relation(output: &Shape) -> ArrangementRelation {
    ArrangementRelation {
        output: output.clone(),
        preserved: Vec::new(),
        modified: Vec::new(),
        generated: Vec::new(),
    }
}

fn body_bindings(
    body: &Shape,
    observed: Vec<PrismNativeImage>,
) -> Result<(Vec<PrismImageBinding>, Vec<PrismGap>), PrismCause> {
    let contexts = body_contexts(body)?;
    Ok(bind_observed_images(observed, &contexts))
}
fn observe_generated(
    maker: &mut BRepPrimAPIMakePrism,
    base: &ArrangementRelation,
    observed: &mut Vec<PrismNativeImage>,
) -> Result<(), PrismCause> {
    // Generated owns one scratch list; snapshot actual values before the next query overwrites it.
    let generated = maker
        .generated(&base.output)
        .map_err(|f| mapped(PrismPhase::History, f))?
        .clone();
    for output in &generated {
        observed.push(native_image(
            base,
            PrismImageRole::Generated,
            generated_relation(output, base),
        ));
    }
    Ok(())
}
fn append_cap_images(
    base: &ArrangementRelation,
    first: Shape,
    last: Shape,
    observed: &mut Vec<PrismNativeImage>,
) {
    for (image, role) in [(first, PrismImageRole::First), (last, PrismImageRole::Last)] {
        if image.is_null() {
            continue;
        }
        let returned = crate::sketch_region_candidates::composed_copy_relation(base, &image);
        observed.push(native_image(base, role, returned));
    }
}
