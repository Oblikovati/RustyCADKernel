// SPDX-License-Identifier: GPL-2.0-only
//! One original-native-role mapping batch preserves all copy/sweep relation classes and multiplicity.
use super::*;
use crate::sketch_region::{ArrangementRelation, OriginalMemberIdentity};
struct RawCopy<'a> {
    key: TwoLoopMemberKey,
    selected: &'a ArrangementRelation,
    copied: &'a ArrangementRelation,
}
struct RawImage<'a> {
    key: TwoLoopMemberKey,
    identity: AnnularChartIdentity,
    returned: &'a ArrangementRelation,
}

pub(super) fn capture<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    copy: &AnnularCopyCapture,
) -> AnnularCheck<'a, AnnularParentCapture> {
    let diagnostic = receipt
        .diagnostic
        .as_ref()
        .expect("checked actual chart owner");
    preflight(request, copy.members.len(), diagnostic.bindings.len())?;
    let copies = raw_copies(receipt, copy)?;
    let images = raw_images(diagnostic, copy)?;
    let mut slices = Vec::new();
    for row in &copies {
        slices.extend(dispositions(row.selected));
        slices.extend(dispositions(row.copied));
    }
    for row in &images {
        slices.extend(dispositions(row.returned));
    }
    let mapped = map_two_loop_original_parents(request, &slices)
        .map_err(|cause| Box::new(AnnularBindingCause::Region(cause)))?;
    finish(copies, images, mapped)
}
fn preflight(
    request: &TwoLoopRegionRequest<'_, '_>,
    copies: usize,
    images: usize,
) -> AnnularCheck<'static> {
    let rows = copies
        .checked_mul(2)
        .and_then(|n| n.checked_add(images))
        .and_then(|n| n.checked_mul(3));
    let actual = rows.unwrap_or(usize::MAX);
    if rows.is_none() || actual > request.requested.maximum_retained_units {
        return Err(Box::new(AnnularBindingCause::Count {
            required: "bounded original parent mapping rows before allocation",
            actual,
            expected: request.requested.maximum_retained_units,
        }));
    }
    Ok(())
}
fn dispositions(row: &ArrangementRelation) -> [&[OriginalMemberIdentity]; 3] {
    [&row.preserved, &row.modified, &row.generated]
}
fn raw_copies<'a>(
    receipt: &'a PrismReceipt,
    copy: &AnnularCopyCapture,
) -> AnnularCheck<'static, Vec<RawCopy<'a>>> {
    let selected = receipt
        .input
        .selected
        .as_ref()
        .expect("checked selected owner");
    let copied = receipt.copied.as_ref().expect("checked copied owner");
    let mut result = Vec::new();
    for member in &copy.members {
        result.push(RawCopy {
            key: member.key,
            selected: relation(&selected.relations, &member.selected.occurrence)?,
            copied: relation(&copied.relations, &member.copied.occurrence)?,
        });
    }
    Ok(result)
}
fn relation<'a>(
    rows: &'a [ArrangementRelation],
    identity: &PartitionOccurrenceIdentity,
) -> AnnularCheck<'static, &'a ArrangementRelation> {
    let matches: Vec<_> = rows
        .iter()
        .filter(|r| PartitionOccurrenceIdentity::capture(&r.output) == *identity)
        .collect();
    require("one actual original-parent relation", matches.len(), 1)?;
    Ok(matches[0])
}
fn raw_images<'a>(
    diagnostic: &'a PrismDiagnostic,
    copy: &AnnularCopyCapture,
) -> AnnularCheck<'static, Vec<RawImage<'a>>> {
    let mut result = Vec::new();
    for row in &diagnostic.bindings {
        let source = PartitionOccurrenceIdentity::capture(&row.native.query);
        let matches: Vec<_> = copy
            .members
            .iter()
            .filter(|m| m.copied.occurrence == source)
            .collect();
        require(
            "one full copied context for native image parents",
            matches.len(),
            1,
        )?;
        let identity = AnnularChartIdentity {
            source: matches[0].copied.clone(),
            output: correspondence::snapshot(&row.context)?,
            role: row.native.role,
        };
        result.push(RawImage {
            key: matches[0].key,
            identity,
            returned: &row.native.returned,
        });
    }
    result.sort_by(|a, b| {
        (a.key, a.identity.role as u8, &a.identity.output.child_slots).cmp(&(
            b.key,
            b.identity.role as u8,
            &b.identity.output.child_slots,
        ))
    });
    Ok(result)
}
fn finish(
    copies: Vec<RawCopy<'_>>,
    images: Vec<RawImage<'_>>,
    mapped: TwoLoopMappedParents,
) -> AnnularCheck<'static, AnnularParentCapture> {
    let mut rows = mapped.rows.into_iter();
    let mut owned_copies = Vec::new();
    for row in copies {
        owned_copies.push(AnnularCopyParents {
            key: row.key,
            selected: take_parents(&mut rows)?,
            copied: take_parents(&mut rows)?,
        });
    }
    let mut owned_images = Vec::new();
    for row in images {
        owned_images.push(AnnularImageParents {
            key: row.key,
            identity: row.identity,
            parents: take_parents(&mut rows)?,
        });
    }
    require("all original parent mapping rows consumed", rows.count(), 0)?;
    Ok(AnnularParentCapture {
        copies: owned_copies,
        images: owned_images,
        retained_units: mapped.retained_units,
    })
}
fn take_parents(
    rows: &mut impl Iterator<Item = Vec<TwoLoopOriginalUse>>,
) -> AnnularCheck<'static, [Vec<TwoLoopOriginalUse>; 3]> {
    let mut result = [Vec::new(), Vec::new(), Vec::new()];
    for row in &mut result {
        *row = rows.next().ok_or_else(|| {
            Box::new(AnnularBindingCause::Count {
                required: "original mapped parent disposition",
                actual: 0,
                expected: 1,
            })
        })?;
    }
    Ok(result)
}
