// SPDX-License-Identifier: GPL-2.0-only
//! Owned body summary: each published face is named through the issued dictionary and paired with
//! its certified area by full occurrence key, never by position.
use super::super::names::AnnularNameMeaning;
use super::*;
use crate::sketch_region_prism::frozen::FrozenSweepRole;
use crate::sketch_region_prism::frozen::annular_join::NamedAnnularFace;
use crate::sketch_region_prism::frozen::ordered_validation::OrderedPrismStage;
use crate::sketch_region_prism::publication::annular::PublishedAnnularPrism;

type BodyCheck<T> = Result<T, StageRefusal>;

/// Project the accepted publication and volume into the owned outcome.
pub(super) fn project(
    request: &OwnedAnnularPrismRequest,
    published: &PublishedAnnularPrism,
    volume: &AnnularPrismVolume,
    dictionary: AnnularNameDictionary,
) -> BodyCheck<Box<OwnedAnnularPrism>> {
    let faces = faces(request, published, &dictionary)?;
    let total = published.measures().total_area();
    let history = published.history();
    let trace = published.validation().trace();
    Ok(Box::new(OwnedAnnularPrism {
        request: request.clone(),
        faces,
        total_area: CertifiedQuantity {
            enclosure: total.enclosure(),
            absolute_error_bound: total.absolute_error_bound(),
        },
        volume: OwnedAnnularVolume {
            volume: CertifiedQuantity {
                enclosure: volume.enclosure(),
                absolute_error_bound: volume.absolute_error_bound(),
            },
            representative: volume.representative(),
            jacobian_sign: volume.jacobian_sign(),
        },
        validation: OrderedPrismStage::ALL.map(|stage| trace.status(stage)),
        history: OwnedAnnularHistorySummary {
            continued_originals: history.continued().len(),
            retired_wires: history.retired_wires().len(),
            deleted_originals: history.deleted_originals().len(),
        },
        dictionary,
        publication_replay: published.replay_bytes().to_vec(),
        volume_replay: volume.replay_bytes().to_vec(),
    }))
}
fn refused(cause: AnnularBodyCause) -> StageRefusal {
    let stage = AnnularOperationStage::Body;
    let diagnostic = AnnularOperationDiagnostic::Body(cause);
    StageRefusal::owned(stage, PrismDiagnosticClass::Incomplete, diagnostic, true)
}
fn faces(
    request: &OwnedAnnularPrismRequest,
    published: &PublishedAnnularPrism,
    dictionary: &AnnularNameDictionary,
) -> BodyCheck<Vec<OwnedAnnularFace>> {
    let named = published.join().faces();
    if named.len() != 4 {
        let (required, actual, expected) = ("four named annular faces", named.len(), 4);
        return Err(refused(AnnularBodyCause::Count {
            required,
            actual,
            expected,
        }));
    }
    named
        .iter()
        .map(|face| {
            let name = face_name(request, face, dictionary)?;
            let area = face_area(published, face)?;
            Ok(OwnedAnnularFace { name, area })
        })
        .collect()
}
fn face_name(
    request: &OwnedAnnularPrismRequest,
    face: &NamedAnnularFace,
    dictionary: &AnnularNameDictionary,
) -> BodyCheck<AnnularFaceName> {
    let Some((key, _)) = face.chart.occurrence.0.last() else {
        return Err(refused(AnnularBodyCause::UnresolvedFace { source: 0 }));
    };
    let unresolved = || {
        refused(AnnularBodyCause::UnresolvedFace {
            source: key.source.0,
        })
    };
    match (key.role, dictionary.meaning(key.source)) {
        (FrozenSweepRole::Generated, Some(AnnularNameMeaning::Boundary { role, boundary, .. })) => {
            let regions = &request.regions;
            let wire = match role {
                super::super::names::AnnularBoundaryRole::Outer => regions.annulus_outer_wire,
                _ => regions.annulus_inner_wire,
            };
            Ok(AnnularFaceName::Side { boundary, wire })
        }
        (
            sweep @ (FrozenSweepRole::First | FrozenSweepRole::Last),
            Some(AnnularNameMeaning::Annulus { region }),
        ) => Ok(AnnularFaceName::Cap { region, sweep }),
        _ => Err(unresolved()),
    }
}
fn face_area(
    published: &PublishedAnnularPrism,
    face: &NamedAnnularFace,
) -> BodyCheck<CertifiedQuantity> {
    let occurrence = &face.chart.occurrence;
    let found: Vec<_> = published
        .measures()
        .faces()
        .iter()
        .filter(|row| &row.named.chart.occurrence == occurrence)
        .collect();
    let [row] = found.as_slice() else {
        let source = occurrence.0.last().map_or(0, |(key, _)| key.source.0);
        return Err(refused(AnnularBodyCause::UnmeasuredFace { source }));
    };
    Ok(CertifiedQuantity {
        enclosure: row.measure.enclosure(),
        absolute_error_bound: row.measure.absolute_error_bound(),
    })
}
