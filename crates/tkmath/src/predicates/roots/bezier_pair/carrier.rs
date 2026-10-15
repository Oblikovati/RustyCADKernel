// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0067: two arcs of one proper conic or one line meet exactly where their
//! parameter arcs meet on the projective line; no resultant is formed.
//!
//! Homogeneous power coefficients `Q(t) = sum_k V_k t^k` with `V_k` in Z^3 classify a
//! source: degree two with `det[V0 V1 V2] != 0` is a proper conic parametrization (the
//! Veronese map composed with an invertible linear map), degree one with `V0 x V1 != 0`
//! a proper line. A second source lies on the same carrier when its whole power form
//! satisfies the first carrier's exact implicit equation identically. Each point of a
//! proper carrier has one projective preimage, so the second source's parameter
//! interval maps to one exact projective arc of the first parameter line.
use super::super::{
    DyadicBound, RootBracket, RootIsolationOptions,
    certificate_arithmetic::{ExactRatio, RatioInterval, world::HomogeneousCurve},
};
use super::operation::{PairContact, PairOperationFailure, PairOperationProof, PairProofEvidence};
use super::source::{HomogeneousPolynomials, OwnedBezierSource};
use super::work_budget::PairWorkBudget;
use num_bigint::{BigInt, Sign};

type Vector = [BigInt; 3];
type Point = [ExactRatio; 3];

/// Irreducible carrier shared by both sources of a common-carrier certificate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierPairCarrierKind {
    /// Both sources are proper parametrizations of one line.
    Line,
    /// Both sources are proper rational quadratic parametrizations of one conic.
    Conic,
}
/// Exact point of the first source's projective parameter line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ProjectiveParameter {
    Finite(ExactRatio),
    Infinite,
}
/// Exact carrier class and the first-parameter preimages of the second source's
/// start, midpoint and end; together they fix the parameter correspondence.
pub(super) struct CommonCarrierEvidence {
    pub kind: BezierPairCarrierKind,
    pub preimages: [ProjectiveParameter; 3],
}
enum Carrier {
    Line {
        normal: Vector,
        first: Vector,
        second: Vector,
    },
    Conic {
        adjugate: [Vector; 3],
    },
}

/// Certify a distinct pair on one carrier, or return `None` to keep the generic route.
pub(super) fn certify(
    sources: &[OwnedBezierSource; 2],
    options: RootIsolationOptions,
    requested_error: f64,
    budget: &PairWorkBudget,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Option<PairOperationProof>, PairOperationFailure> {
    let Some((carrier, kind, right)) = shared_carrier(sources, cancelled)? else {
        return Ok(None);
    };
    let certificates = [
        super::source_certificate::certify(&sources[0], options, budget, cancelled)?,
        super::source_certificate::certify(&sources[1], options, budget, cancelled)?,
    ];
    let preimages = preimages(&carrier, &right, sources[1].interval);
    let touches = super::carrier_spans::touches(&preimages, sources)?;
    let contacts = contacts(&sources[0], touches, requested_error, cancelled)?;
    let evidence = CommonCarrierEvidence { kind, preimages };
    Ok(Some(PairOperationProof {
        sources: sources.clone(),
        evidence: PairProofEvidence::CommonCarrier(Box::new(evidence)),
        contacts,
        source_certificates: certificates,
        work: budget.receipt(),
    }))
}
type SharedCarrier = (Carrier, BezierPairCarrierKind, HomogeneousPolynomials);
/// Both sources classify to the same proper carrier and the second lies on the first's.
fn shared_carrier(
    sources: &[OwnedBezierSource; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Option<SharedCarrier>, PairOperationFailure> {
    let left = sources[0]
        .homogeneous(cancelled)
        .map_err(PairOperationFailure::Source)?;
    let right = sources[1]
        .homogeneous(cancelled)
        .map_err(PairOperationFailure::Source)?;
    let Some((carrier, kind)) = classify(&left) else {
        return Ok(None);
    };
    let same = classify(&right).map(|(_, other)| other) == Some(kind) && contains(&carrier, &right);
    Ok(same.then_some((carrier, kind, right)))
}
fn columns(source: &HomogeneousPolynomials) -> Vec<Vector> {
    let degree = source.components.iter().filter_map(|c| c.degree()).max();
    let count = degree.map_or(0, |degree| degree + 1);
    (0..count)
        .map(|power| std::array::from_fn(|axis| source.components[axis].coefficient(power)))
        .collect()
}
fn classify(source: &HomogeneousPolynomials) -> Option<(Carrier, BezierPairCarrierKind)> {
    let columns = columns(source);
    match columns.as_slice() {
        [first, second] => {
            let normal = cross(first, second);
            let proper = normal.iter().any(|value| value.sign() != Sign::NoSign);
            let line = Carrier::Line {
                normal,
                first: first.clone(),
                second: second.clone(),
            };
            proper.then_some((line, BezierPairCarrierKind::Line))
        }
        [c0, c1, c2] => {
            let adjugate = [cross(c1, c2), cross(c2, c0), cross(c0, c1)];
            let proper = dot(&adjugate[0], c0).sign() != Sign::NoSign;
            proper.then_some((Carrier::Conic { adjugate }, BezierPairCarrierKind::Conic))
        }
        _ => None,
    }
}
/// The second source's power form satisfies the carrier equation identically.
fn contains(carrier: &Carrier, source: &HomogeneousPolynomials) -> bool {
    let columns = columns(source);
    match carrier {
        Carrier::Line { normal, .. } => columns
            .iter()
            .all(|v| dot(normal, v).sign() == Sign::NoSign),
        Carrier::Conic { adjugate } => conic_identity(adjugate, &columns),
    }
}
/// `L1(u)^2 - L0(u) L2(u) == 0` with `L_i(u) = sum_k (adj_i . V_k) u^k`.
fn conic_identity(adjugate: &[Vector; 3], columns: &[Vector]) -> bool {
    let rows: [Vec<BigInt>; 3] =
        std::array::from_fn(|i| columns.iter().map(|v| dot(&adjugate[i], v)).collect());
    let square = convolve(&rows[1], &rows[1]);
    let product = convolve(&rows[0], &rows[2]);
    square
        .iter()
        .zip(&product)
        .all(|(left, right)| left == right)
}
fn convolve(left: &[BigInt], right: &[BigInt]) -> Vec<BigInt> {
    let mut result = vec![BigInt::from(0); (left.len() + right.len()).saturating_sub(1)];
    for (i, a) in left.iter().enumerate() {
        for (j, b) in right.iter().enumerate() {
            result[i + j] += a * b;
        }
    }
    result
}
fn preimages(
    carrier: &Carrier,
    source: &HomogeneousPolynomials,
    interval: [f64; 2],
) -> [ProjectiveParameter; 3] {
    let [start, end] = interval.map(ExactRatio::from_finite);
    let middle = start.add(&end).multiply(&ExactRatio::from_finite(0.5));
    let columns = columns(source);
    [start, middle, end].map(|u| preimage(carrier, &evaluate(&columns, &u)))
}
fn evaluate(columns: &[Vector], parameter: &ExactRatio) -> Point {
    std::array::from_fn(|axis| {
        columns
            .iter()
            .rev()
            .fold(ExactRatio::from_finite(0.), |sum, column| {
                sum.multiply(parameter).add(&integer(&column[axis]))
            })
    })
}
/// Unique projective preimage of a carrier point.
fn preimage(carrier: &Carrier, point: &Point) -> ProjectiveParameter {
    let (numerator, denominator) = match carrier {
        // X = alpha V0 + beta V1 gives t = beta / alpha.
        Carrier::Line {
            normal,
            first,
            second,
        } => (
            exact_dot(normal, &cross_point(first, point)),
            exact_dot(normal, &cross_point_right(point, second)),
        ),
        // adj X is proportional to (1, t, t^2).
        Carrier::Conic { adjugate } => (
            exact_dot(&adjugate[1], point),
            exact_dot(&adjugate[0], point),
        ),
    };
    if denominator.sign() == std::cmp::Ordering::Equal {
        return ProjectiveParameter::Infinite;
    }
    ProjectiveParameter::Finite(numerator.divide(&denominator))
}
fn contacts(
    first: &OwnedBezierSource,
    touches: Vec<(f64, f64)>,
    requested_error: f64,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<PairContact>, PairOperationFailure> {
    let curve = HomogeneousCurve::from_controls(&first.controls, first.weights.as_deref());
    let mut contacts = Vec::with_capacity(touches.len());
    for (candidate_index, (t, u)) in touches.into_iter().enumerate() {
        let exact = DyadicBound::from_f64(t).expect("finite source interval endpoint");
        let root = RootBracket {
            lower: exact.clone(),
            upper: exact,
            multiplicity: 1,
        };
        let world = curve
            .enclose(&root, cancelled)
            .map_err(PairOperationFailure::World)?;
        let limit = ExactRatio::from_finite(requested_error);
        if world.error_squared > limit.multiply(&limit) || world.achieved_error > requested_error {
            return Err(PairOperationFailure::WorldResolution {
                requested_error,
                last_root: Box::new(root),
                enclosure: Some((world.bounds, world.point, world.achieved_error)),
            });
        }
        let exact_u = RatioInterval::point(ExactRatio::from_finite(u));
        contacts.push(PairContact {
            candidate_index,
            root,
            u_bounds: [u, u],
            exact_u,
            world,
        });
    }
    Ok(contacts)
}
fn cross(left: &Vector, right: &Vector) -> Vector {
    std::array::from_fn(|axis| {
        let (next, last) = ((axis + 1) % 3, (axis + 2) % 3);
        &left[next] * &right[last] - &left[last] * &right[next]
    })
}
fn dot(left: &Vector, right: &Vector) -> BigInt {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}
fn integer(value: &BigInt) -> ExactRatio {
    ExactRatio::new(value.clone(), BigInt::from(1))
}
fn exact_dot(left: &Vector, right: &Point) -> ExactRatio {
    left.iter()
        .zip(right)
        .fold(ExactRatio::from_finite(0.), |sum, (a, b)| {
            sum.add(&integer(a).multiply(b))
        })
}
/// `V x X` for an integer `V` and exact `X`.
fn cross_point(left: &Vector, right: &Point) -> Point {
    std::array::from_fn(|axis| {
        let (next, last) = ((axis + 1) % 3, (axis + 2) % 3);
        integer(&left[next])
            .multiply(&right[last])
            .subtract(&integer(&left[last]).multiply(&right[next]))
    })
}
/// `X x V` for an exact `X` and integer `V`.
fn cross_point_right(left: &Point, right: &Vector) -> Point {
    let opposite = cross_point(right, left);
    opposite.map(|value| value.negated())
}
