// SPDX-License-Identifier: GPL-2.0-only
//! Chart equations of a conic source against a conic target, and their contacts.
use super::super::certificate_arithmetic::ExactRatio;
use super::algebraic::IsolatedRoot;
use super::budget::Work;
use super::conic_pair::{ConicPair, half_turn};
use super::conic_point::{ChartContact, ContactPoint, quadratic_contacts};
use super::failure::SketchIncidenceCause as Cause;
use super::pieces::piece_roots;
use super::place::{KeyValue, SketchSweepChart, SourcePlace};
use super::rational_polynomial::{RationalPolynomial, ratio};
use super::root_sign::sign_at_root;
use super::spans::HomogeneousPoint;
use super::sweep::SweepFrame;
use num_bigint::Sign;
use std::cmp::Ordering;

/// The direction chart `d(K) = -K r + o r_perp` of one reference vector.
pub(super) struct Chart {
    reference: [ExactRatio; 2],
    counterclockwise: bool,
}
/// Coefficients of `f` and `g` in `lambda` as polynomials of `K`.
pub(super) struct ChartEquations {
    pub direction: [RationalPolynomial; 2],
    pub source_form: RationalPolynomial,
    pub target_form: RationalPolynomial,
    pub bilinear: RationalPolynomial,
    pub constant: ExactRatio,
    pub source_constant: ExactRatio,
}
impl Chart {
    /// The source's own chart: its places are read directly.
    pub fn primary(frame: &SweepFrame) -> Self {
        Self {
            reference: frame.reference.clone(),
            counterclockwise: frame.counterclockwise,
        }
    }
    /// The chart of `r_perp`; its `K' = 0` is the source reference line.
    pub fn secondary(frame: &SweepFrame) -> Self {
        let [x, y] = &frame.reference;
        Self {
            reference: [y.negated(), x.clone()],
            counterclockwise: frame.counterclockwise,
        }
    }
    fn sense(&self) -> ExactRatio {
        ratio(if self.counterclockwise { 1. } else { -1. })
    }
    /// `d(K)` as two linear polynomials.
    pub fn direction(&self) -> [RationalPolynomial; 2] {
        let [x, y] = &self.reference;
        let sense = self.sense();
        [
            RationalPolynomial::linear(y.negated().multiply(&sense), x.negated()),
            RationalPolynomial::linear(x.multiply(&sense), y.negated()),
        ]
    }
    /// `d(K)` at a rational key.
    pub fn direction_at(&self, key: &ExactRatio) -> [ExactRatio; 2] {
        self.direction().map(|coordinate| coordinate.evaluate(key))
    }
    pub fn equations(&self, pair: &ConicPair<'_>) -> ChartEquations {
        let direction = self.direction();
        let source = &pair.source.implicit;
        let offset: [ExactRatio; 2] =
            std::array::from_fn(|axis| source.center[axis].subtract(&pair.target.center[axis]));
        let applied = pair.target.form.apply(&offset);
        let bilinear = direction[0]
            .scale(&applied[0])
            .add(&direction[1].scale(&applied[1]));
        ChartEquations {
            source_form: source.form.compose(&direction),
            target_form: pair.target.form.compose(&direction),
            bilinear,
            constant: pair
                .target
                .form
                .value(&offset)
                .subtract(&pair.target.constant),
            source_constant: source.constant.clone(),
            direction,
        }
    }
    /// Every contact off the reference line, in this (primary) chart.
    pub fn contacts(
        &self,
        pair: &ConicPair<'_>,
        work: &mut Work<'_>,
    ) -> Result<Vec<ChartContact>, Cause> {
        let equations = self.equations(pair);
        let roots = piece_roots(&equations.resultant(), None, 0, work)?
            .ok_or(Cause::Invariant("distinct conics have a nonzero resultant"))?;
        let point = equations.point(&pair.source.implicit.center);
        let mut found = Vec::new();
        let mut exceptional = false;
        for mut root in roots {
            let bilinear = sign_at_root(&mut root.contact, &equations.bilinear, work)?;
            if bilinear == Sign::NoSign {
                if !std::mem::replace(&mut exceptional, true) {
                    found.extend(self.exceptional(pair, &equations, work)?);
                }
                continue;
            }
            let shared = sign_at_root(&mut root.contact, &equations.shared(), work)?;
            // lambda = -E / (2 Q_S B) and Q_S > 0, so its sign is -sign(E) sign(B).
            let chart = half_turn(shared * bilinear == Sign::Minus);
            let place = SourcePlace::sweep(chart, Some(KeyValue::Root(root.contact.clone())));
            found.push(ChartContact::regular(root, point.clone(), place));
        }
        Ok(found)
    }
    // Both half turns of the line `B(d) = 0` meet the target; its key is rational.
    fn exceptional(
        &self,
        pair: &ConicPair<'_>,
        equations: &ChartEquations,
        work: &mut Work<'_>,
    ) -> Result<Vec<ChartContact>, Cause> {
        let [constant, slope] = [0, 1].map(|power| equations.bilinear.coefficient(power));
        let key = constant.negated().divide(&slope);
        let places = [true, false].map(|positive| {
            SourcePlace::sweep(half_turn(positive), Some(KeyValue::Ratio(key.clone())))
        });
        quadratic_contacts(pair, &self.direction_at(&key), places, None, work)
    }
    /// Contacts on the source reference line, read at `K' = 0` of this secondary chart.
    pub fn contacts_at_zero(
        &self,
        pair: &ConicPair<'_>,
        work: &mut Work<'_>,
    ) -> Result<Vec<ChartContact>, Cause> {
        let equations = self.equations(pair);
        let zero = ratio(0.);
        let direction = self.direction_at(&zero);
        // d'(0) = -o r: positive lambda' points to the reference ray iff o < 0.
        let places = [!self.counterclockwise, self.counterclockwise].map(on_reference_line);
        let resultant = equations.resultant();
        if equations.bilinear.coefficient(0).sign() == Ordering::Equal {
            return quadratic_contacts(pair, &direction, places, None, work);
        }
        let root = exact_zero_root(&resultant, work)?;
        let lambda = equations
            .shared()
            .evaluate(&zero)
            .negated()
            .divide(&equations.fibre_denominator().evaluate(&zero));
        let place = places[usize::from(lambda.sign() == Ordering::Less)].clone();
        let point = equations.point(&pair.source.implicit.center);
        Ok(vec![ChartContact::regular(root, point, place)])
    }
}
fn on_reference_line(positive_along_reference: bool) -> SourcePlace {
    let chart = match positive_along_reference {
        true => SketchSweepChart::Reference,
        false => SketchSweepChart::Opposite,
    };
    SourcePlace::sweep(chart, None)
}
// The root `K' = 0` of a resultant that vanishes there, with its exact sides.
pub(super) fn exact_zero_root(
    resultant: &RationalPolynomial,
    work: &mut Work<'_>,
) -> Result<super::pieces::PieceRoot, Cause> {
    let roots = piece_roots(resultant, None, 0, work)?
        .ok_or(Cause::Invariant("distinct conics have a nonzero resultant"))?;
    let zero = super::pieces::dyadic(0.);
    roots
        .into_iter()
        .find(|root| root.contact.exact() == Some(&zero) || brackets(&root.contact, &zero))
        .ok_or(Cause::Invariant(
            "reference line contact missed its chart root",
        ))
        .and_then(|mut root| {
            narrow_to(&mut root.contact, &zero, work)?;
            Ok(root)
        })
}
fn brackets(root: &IsolatedRoot, value: &super::super::DyadicBound) -> bool {
    root.lower <= *value && *value <= root.upper
}
fn narrow_to(
    root: &mut IsolatedRoot,
    value: &super::super::DyadicBound,
    work: &mut Work<'_>,
) -> Result<(), Cause> {
    match root.split_at(value, work)? {
        Ordering::Equal => Ok(()),
        _ => Err(Cause::Invariant(
            "reference line contact is not at the chart origin",
        )),
    }
}
impl ChartEquations {
    /// `E = Q_S G + c_S Q_T`.
    pub fn shared(&self) -> RationalPolynomial {
        self.source_form
            .scale(&self.constant)
            .add(&self.target_form.scale(&self.source_constant))
    }
    /// `R = E^2 - 4 c_S Q_S B^2`.
    pub fn resultant(&self) -> RationalPolynomial {
        let shared = self.shared();
        let bilinear = self.bilinear.multiply(&self.bilinear);
        let scale = self.source_constant.multiply(&ratio(-4.));
        shared
            .multiply(&shared)
            .add(&self.source_form.multiply(&bilinear).scale(&scale))
    }
    /// `W = 2 Q_S B`, the denominator of the fibre point `lambda = -E / W`.
    pub fn fibre_denominator(&self) -> RationalPolynomial {
        self.source_form.multiply(&self.bilinear).scale(&ratio(2.))
    }
    /// `C + lambda d` as `(C W^2 - E d W) / W^2`, with a positive weight.
    pub fn point(&self, center: &[ExactRatio; 2]) -> ContactPoint {
        let weight = self.fibre_denominator();
        let squared = weight.multiply(&weight);
        let along = self.shared().multiply(&weight).scale(&ratio(-1.));
        let coordinate = |axis: usize| {
            squared
                .scale(&center[axis])
                .add(&along.multiply(&self.direction[axis]))
        };
        ContactPoint::Homogeneous(HomogeneousPoint {
            x: coordinate(0),
            y: coordinate(1),
            w: squared,
        })
    }
}
