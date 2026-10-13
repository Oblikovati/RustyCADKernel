// SPDX-License-Identifier: GPL-2.0-only
use super::super::{BezierWinding, winding::WindingEvaluation};
use super::{BezierWireError, BezierWireLocation, BezierWireResult, endpoint::Endpoint};
struct LocatedEndpoint {
    id: u64,
    point: Endpoint,
}
#[derive(Default)]
pub(super) struct WireWalk {
    first: Option<LocatedEndpoint>,
    previous: Option<LocatedEndpoint>,
    boundary_ids: Vec<u64>,
    winding: i64,
    pub used: u16,
}
impl WireWalk {
    pub fn join(&mut self, id: u64, [start, end]: [Endpoint; 2]) -> Result<(), BezierWireError> {
        if let Some(previous) = &self.previous {
            check_join(previous, id, &start)?;
        }
        self.first
            .get_or_insert(LocatedEndpoint { id, point: start });
        self.previous = Some(LocatedEndpoint { id, point: end });
        Ok(())
    }
    pub fn append(
        &mut self,
        id: u64,
        reversed: bool,
        result: WindingEvaluation,
    ) -> Result<(), BezierWireError> {
        self.append_contribution(id, reversed, result.incidence, result.bisections)
    }
    pub(super) fn append_contribution(
        &mut self,
        id: u64,
        reversed: bool,
        incidence: BezierWinding,
        bisections: u16,
    ) -> Result<(), BezierWireError> {
        self.used += bisections;
        match incidence {
            BezierWinding::Boundary => self.boundary_ids.push(id),
            BezierWinding::Clear(count) => {
                let contribution = i64::from(count) * if reversed { -1 } else { 1 };
                self.winding = checked_winding(self.winding, contribution)?;
            }
        }
        Ok(())
    }
    pub fn finish(self) -> Result<BezierWireResult, BezierWireError> {
        let first = self.first.ok_or(BezierWireError::Empty)?;
        let previous = self.previous.ok_or(BezierWireError::Empty)?;
        check_join(&previous, first.id, &first.point)?;
        let location = location(self.boundary_ids, self.winding);
        Ok(BezierWireResult {
            location,
            bisections: self.used,
        })
    }
}
fn check_join(
    previous: &LocatedEndpoint,
    id: u64,
    start: &Endpoint,
) -> Result<(), BezierWireError> {
    if previous.point.matches(start) {
        return Ok(());
    }
    Err(BezierWireError::OpenJoin {
        from: previous.id,
        to: id,
    })
}
fn location(segment_ids: Vec<u64>, winding: i64) -> BezierWireLocation {
    if !segment_ids.is_empty() {
        return BezierWireLocation::Boundary { segment_ids };
    }
    if winding == 0 {
        return BezierWireLocation::Outside;
    }
    BezierWireLocation::Inside { winding }
}
fn checked_winding(partial: i64, contribution: i64) -> Result<i64, BezierWireError> {
    partial
        .checked_add(contribution)
        .ok_or(BezierWireError::WindingOverflow {
            partial,
            contribution,
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    #[test]
    fn winding_overflow_is_a_typed_decline_in_both_directions() {
        for (partial, contribution) in [(i64::MAX, 1), (i64::MIN, -1)] {
            let error = checked_winding(partial, contribution).unwrap_err();
            assert_eq!(
                error,
                BezierWireError::WindingOverflow {
                    partial,
                    contribution
                }
            );
            assert!(error.to_string().contains("expected an i64 sum"));
            assert!(error.source().is_none());
            assert_eq!(checked_winding(partial, 0), Ok(partial));
        }
    }
}
