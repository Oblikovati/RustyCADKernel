// SPDX-License-Identifier: GPL-2.0-only
//! Bind every directed body edge use to one already-proved actual face trim and its parents.
use super::super::{
    PrismBodyContext,
    region_certificate::current_binding::{BoundPrismChart, CurrentPrismBinding},
    surface_charts::CapturedPrismTrim,
};
use super::{census_contexts::*, census_types::*};
use crate::sketch_region::PartitionOccurrenceIdentity;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture_trim_census(
    contexts: &[PrismBodyContext],
    binding: &CurrentPrismBinding,
) -> CensusCheck<Vec<ToleranceTrimCensus>> {
    check_face_coverage(contexts, binding)?;
    let mut trims = Vec::new();
    for context in contexts {
        if context.occurrence.shape_type()? == ShapeEnum::Edge {
            let key = capture_context(context);
            trims.push(attach_context(
                bind_trim(context, binding, key.clone()),
                &key,
            )?);
        }
    }
    check_all_trims(binding, &trims)?;
    Ok(trims)
}

fn check_face_coverage(
    contexts: &[PrismBodyContext],
    binding: &CurrentPrismBinding,
) -> CensusCheck<()> {
    for context in contexts {
        if context.occurrence.shape_type()? != ShapeEnum::Face {
            continue;
        }
        attach_context(check_face(context, binding), &capture_context(context))?;
    }
    Ok(())
}

fn check_face(context: &PrismBodyContext, binding: &CurrentPrismBinding) -> CensusCheck<()> {
    let key = PartitionOccurrenceIdentity::capture(&context.occurrence);
    let matches = binding
        .charts
        .iter()
        .filter(|chart| chart.output == key)
        .count();
    count("current chart for body face", matches, 1)
}

fn bind_trim(
    context: &PrismBodyContext,
    binding: &CurrentPrismBinding,
    key: ToleranceUseContext,
) -> CensusCheck<ToleranceTrimCensus> {
    let face = parent_face(context)?;
    let edge = PartitionOccurrenceIdentity::capture(&context.occurrence);
    let chart = bound_chart(binding, &face)?;
    let trim = bound_trim(chart, &edge)?;
    Ok(ToleranceTrimCensus {
        face,
        edge,
        context: key,
        source: trim.source.clone(),
        parents: trim.parents.clone(),
        role: trim.role,
    })
}

fn bound_chart<'a>(
    binding: &'a CurrentPrismBinding,
    face: &PartitionOccurrenceIdentity,
) -> CensusCheck<&'a BoundPrismChart> {
    let charts: Vec<_> = binding
        .charts
        .iter()
        .filter(|chart| &chart.output == face)
        .collect();
    count("bound face chart for directed trim", charts.len(), 1)?;
    Ok(charts[0])
}

fn bound_trim<'a>(
    chart: &'a BoundPrismChart,
    edge: &PartitionOccurrenceIdentity,
) -> CensusCheck<&'a CapturedPrismTrim> {
    let trims: Vec<_> = chart
        .captured
        .trims
        .iter()
        .filter(|trim| &trim.output == edge)
        .collect();
    count("bound chart trim for directed edge", trims.len(), 1)?;
    Ok(trims[0])
}

fn parent_face(context: &PrismBodyContext) -> CensusCheck<PartitionOccurrenceIdentity> {
    let mut faces = Vec::new();
    for parent in &context.ancestors {
        if parent.shape_type()? == ShapeEnum::Face {
            faces.push(parent);
        }
    }
    count("face ancestor of directed edge", faces.len(), 1)?;
    Ok(PartitionOccurrenceIdentity::capture(faces[0]))
}

fn check_all_trims(
    binding: &CurrentPrismBinding,
    actual: &[ToleranceTrimCensus],
) -> CensusCheck<()> {
    for chart in &binding.charts {
        for trim in &chart.captured.trims {
            let count_actual = actual
                .iter()
                .filter(|row| row.face == chart.output && row.edge == trim.output)
                .count();
            count("body occurrence for captured trim", count_actual, 1)?;
        }
    }
    Ok(())
}
