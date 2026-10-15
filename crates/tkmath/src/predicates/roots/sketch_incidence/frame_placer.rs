// SPDX-License-Identifier: GPL-2.0-only
//! Placement of target points on an implicit source (line, line offset, conic).
//!
//! A line (or line offset) source places a point by its line parameter
//! `d.(p - A) / |d|^2`; a conic source by the sweep chart and cotangent key of its
//! direction from the center. Rational target points give rational image maps;
//! offset target points give surd ratios (ADR 0080).
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::implicit::LineImplicit;
use super::rational_polynomial::{RationalPolynomial, Ring};
use super::root_map::{MappedEquation, RationalMap};
use super::source_frame::SourceFrame;
use super::spans::HomogeneousPoint;
use super::surd::Surd;
use super::surd_map::{MappedSurd, SurdMap};
use super::sweep::{KeyMap, PointPlacer};

impl SourceFrame {
    /// The line whose parameter places points, for line and line offset sources.
    pub fn placing_line(&self) -> Option<&LineImplicit> {
        match self {
            Self::Line(line) => Some(&line.implicit),
            Self::OffsetLine(line) => Some(&line.implicit.line),
            _ => None,
        }
    }
    /// Placer of target points given by polynomials of the target parameter.
    pub fn placer(
        &self,
        point: &HomogeneousPoint,
        contact: &RationalPolynomial,
        work: &mut Work<'_>,
    ) -> Result<PointPlacer, Cause> {
        if let Some(line) = self.placing_line() {
            let map = RationalMap {
                numerator: line.along(point),
                denominator: point.w.scale(&line.squared_length()),
            };
            let key = MappedEquation::new(contact, map, work)?;
            return Ok(PointPlacer::Parameter(KeyMap::Rational(key)));
        }
        match self {
            Self::Conic(conic) => {
                let relative = point.relative(&conic.frame.center);
                conic.frame.placer(&relative, contact, work)
            }
            _ => Err(Cause::Invariant(
                "a parametric source has no implicit placer",
            )),
        }
    }
    /// Placer of offset target points given by surds of the target parameter.
    pub fn surd_placer(
        &self,
        point: &HomogeneousPoint<Surd>,
        contact: &RationalPolynomial,
        work: &mut Work<'_>,
    ) -> Result<PointPlacer, Cause> {
        if let Some(line) = self.placing_line() {
            let map = SurdMap {
                numerator: line.along(point),
                denominator: Ring::scale(&point.w, &line.squared_length()),
            };
            let key = MappedSurd::new(contact, map, work)?;
            return Ok(PointPlacer::Parameter(KeyMap::Surd(key)));
        }
        match self {
            Self::Conic(conic) => {
                let relative = point.relative(&conic.frame.center);
                conic.frame.surd_placer(&relative, contact, work)
            }
            _ => Err(Cause::Invariant(
                "a parametric source has no implicit placer",
            )),
        }
    }
}
