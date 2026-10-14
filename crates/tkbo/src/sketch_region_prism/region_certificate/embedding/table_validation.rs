// SPDX-License-Identifier: GPL-2.0-only
//! ADR0039 checks retained coverage using the existing theorem table, without another geometry query.
use super::super::hypothesis::RegionHypothesisCause;
use super::*;

#[derive(Debug)]
pub(crate) enum PrismEmbeddingTableFailure {
    Cancelled {
        completed: usize,
    },
    Reconstruction(Box<RegionHypothesisCause>),
    DomainMultiplicity {
        role: PrismImageRole,
        actual: usize,
        expected: usize,
    },
    DomainCorrespondence {
        actual: Box<PrismFaceDomain>,
        expected: Box<PrismFaceDomain>,
    },
    RowCount {
        actual: usize,
        expected: usize,
    },
    PairMultiplicity {
        first: PrismImageRole,
        second: PrismImageRole,
        actual: usize,
        expected: usize,
    },
    Correspondence {
        actual: Box<PrismFaceIncidence>,
        expected: Box<PrismFaceIncidence>,
    },
}
type TableCheck = Result<(), PrismEmbeddingTableFailure>;

/// Check all retained named conclusions; `check_prism_embedding_table(join.embedding(), |_| false)`.
/// This verifies the existing theorem's table correspondence, not a new independent embedding theorem.
pub(crate) fn check_prism_embedding_table(
    evidence: &PrismEmbeddingEvidence,
    mut cancelled: impl FnMut(usize) -> bool,
) -> TableCheck {
    poll_table(&mut cancelled, 0)?;
    let expected = domains::capture_domains(evidence.binding())
        .map_err(|cause| PrismEmbeddingTableFailure::Reconstruction(Box::new(cause)))?;
    check_domains(evidence.domains(), &expected)?;
    check_rows(&expected, evidence.incidences(), |step| cancelled(step + 1))
        .map_err(offset_row_checkpoint)
}

fn offset_row_checkpoint(cause: PrismEmbeddingTableFailure) -> PrismEmbeddingTableFailure {
    match cause {
        PrismEmbeddingTableFailure::Cancelled { completed } => {
            PrismEmbeddingTableFailure::Cancelled {
                completed: completed + 1,
            }
        }
        other => other,
    }
}

pub(super) fn check_domains(
    actual: &[PrismFaceDomain; 3],
    expected: &[PrismFaceDomain; 3],
) -> TableCheck {
    for domain in expected {
        check_domain(actual, domain)?;
    }
    Ok(())
}

fn check_domain(actual: &[PrismFaceDomain; 3], domain: &PrismFaceDomain) -> TableCheck {
    let candidates = actual.iter().filter(|row| row.role == domain.role);
    let matching: Vec<_> = candidates.collect();
    if matching.len() != 1 {
        return Err(PrismEmbeddingTableFailure::DomainMultiplicity {
            role: domain.role,
            actual: matching.len(),
            expected: 1,
        });
    }
    if matching[0] != domain {
        return Err(PrismEmbeddingTableFailure::DomainCorrespondence {
            actual: Box::new(matching[0].clone()),
            expected: Box::new(domain.clone()),
        });
    }
    Ok(())
}

pub(super) fn check_rows(
    domains: &[PrismFaceDomain; 3],
    actual: &[PrismFaceIncidence],
    mut cancelled: impl FnMut(usize) -> bool,
) -> TableCheck {
    poll_table(&mut cancelled, 0)?;
    let expected = table::six_rows(domains)
        .map_err(|cause| PrismEmbeddingTableFailure::Reconstruction(Box::new(cause)))?;
    require_row_count(actual.len(), expected.len())?;
    for (index, row) in expected.iter().enumerate() {
        poll_table(&mut cancelled, index + 1)?;
        check_pair(actual, row)?;
    }
    poll_table(&mut cancelled, expected.len() + 1)
}

fn poll_table(cancelled: &mut impl FnMut(usize) -> bool, completed: usize) -> TableCheck {
    if cancelled(completed) {
        return Err(PrismEmbeddingTableFailure::Cancelled { completed });
    }
    Ok(())
}

fn require_row_count(actual: usize, expected: usize) -> TableCheck {
    if actual != expected {
        return Err(PrismEmbeddingTableFailure::RowCount { actual, expected });
    }
    Ok(())
}

fn check_pair(actual: &[PrismFaceIncidence], expected: &PrismFaceIncidence) -> TableCheck {
    let candidates = actual.iter().filter(|row| same_pair(row, expected));
    let matches: Vec<_> = candidates.collect();
    if matches.len() != 1 {
        return Err(PrismEmbeddingTableFailure::PairMultiplicity {
            first: expected.first.role,
            second: expected.second.role,
            actual: matches.len(),
            expected: 1,
        });
    }
    if same_correspondence(matches[0], expected) {
        return Ok(());
    }
    Err(PrismEmbeddingTableFailure::Correspondence {
        actual: Box::new(matches[0].clone()),
        expected: Box::new(expected.clone()),
    })
}

fn same_pair(actual: &PrismFaceIncidence, expected: &PrismFaceIncidence) -> bool {
    let direct =
        actual.first.role == expected.first.role && actual.second.role == expected.second.role;
    let reverse =
        actual.first.role == expected.second.role && actual.second.role == expected.first.role;
    direct || reverse
}

fn same_correspondence(actual: &PrismFaceIncidence, expected: &PrismFaceIncidence) -> bool {
    let direct = actual.first == expected.first && actual.second == expected.second;
    let reverse = actual.first == expected.second && actual.second == expected.first;
    actual.conclusion == expected.conclusion && (direct || reverse)
}
