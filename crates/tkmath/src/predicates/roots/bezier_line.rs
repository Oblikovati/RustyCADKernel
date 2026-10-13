// SPDX-License-Identifier: GPL-2.0-only
//! Exact supplied-curve parameter classification; no B-rep is built (ADR 0006).
mod error;
mod projection;
mod ray;
pub use ray::{BezierRayError, classify_bezier_ray_contacts};
mod solve;
mod winding;
mod wire;
use super::{RootIsolation, RootIsolationOptions};
pub use error::{BezierInput, BezierLineError, BezierStage};
pub use winding::{BezierWinding, bezier_winding_contribution};
pub use wire::{
    BezierWireError, BezierWireLocation, BezierWireQuery, BezierWireResult, BezierWireSegment,
    classify_bezier_wire,
};

/// Borrowed controls of a polynomial or rational Bézier curve and an infinite line.
#[derive(Clone, Copy, Debug)]
pub struct BezierLineQuery<'a> {
    /// Finite planar control points in Bernstein order; at least one point.
    pub controls: &'a [[f64; 2]],
    /// Optional finite weights, one per point. None means equal unit weights.
    pub weights: Option<&'a [f64]>,
    /// Two distinct finite points defining an infinite line, not a segment.
    pub line: [[f64; 2]; 2],
    /// Closed parameter query; the Bernstein basis remains fixed at `[0,1]`.
    pub interval: [f64; 2],
    /// Shared degree, width, and total subdivision limits for both stages.
    pub options: RootIsolationOptions,
}
/// A completed contact classification on a denominator-regular interval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BezierLineContact {
    /// Every isolated contact, or a completed empty result. Work includes denominator checks.
    Isolated(RootIsolation),
    /// The entire queried curve lies on the infinite line.
    Coincident {
        /// Subdivisions used to certify the denominator has no roots.
        bisections: u16,
    },
}
/// Certify every Bézier-to-line contact parameter without rounded projection products.
///
/// A denominator zero is refused even if a numerator factor could cancel it.
/// ```
/// use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
/// let query = BezierLineQuery { controls: &[[0., -1.], [1., 1.]], weights: None,
///     line: [[0., 0.], [1., 0.]], interval: [0., 1.],
///     options: RootIsolationOptions::new(0.001) };
/// assert!(matches!(intersect_bezier_line(query, || false)?, BezierLineContact::Isolated(_)));
/// # Ok::<(), BezierLineError>(())
/// ```
pub fn intersect_bezier_line(
    query: BezierLineQuery<'_>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<BezierLineContact, BezierLineError> {
    let (bounds, width) = checked_query(query)?;
    let projected = projection::project(query, &mut cancelled)?;
    solve::classify(projected, bounds, width, query.options, &mut cancelled)
}

fn checked_query(
    query: BezierLineQuery<'_>,
) -> Result<([super::DyadicBound; 2], super::DyadicBound), BezierLineError> {
    super::checked_policy_interval(query.interval, query.options).map_err(|cause| {
        BezierLineError::Root {
            stage: BezierStage::Input,
            cause,
        }
    })
}
