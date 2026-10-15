// SPDX-License-Identifier: GPL-2.0-only
//! Conic sources: their sweep frame, start and end places, and positions.
//!
//! A full conic is swept from its frame axis; a ray arc from its start ray, so its
//! start is the exact reference chart. An angle-bounded arc keeps its exact frame
//! axis as the reference: its bounds are frame-angle keys, and a place before the
//! start angle is marked `turn` so that sweep order runs from the start.
use super::angle::{AngleFrame, FrameAngle};
use super::budget::Work;
use super::failure::SketchIncidenceCause as Cause;
use super::implicit::ConicImplicit;
use super::input::{SketchConicAngles, SketchConicBounds, SketchIncidenceCurve};
use super::place::{AngleKey, KeyValue, SketchSweepChart, SourcePlace, compare_places};
use super::rational_polynomial::ratio;
use super::report::SketchLinePosition;
use super::sweep::{SweepFrame, chart};
use std::cmp::Ordering;

pub(super) struct ConicSource {
    pub implicit: ConicImplicit,
    pub frame: SweepFrame,
    /// Place of the arc start; the reference chart for full conics and ray arcs.
    pub start: SourcePlace,
    /// Place of the arc end; None for a full conic.
    pub end: Option<SourcePlace>,
    angled: bool,
}
struct ConicShape<'a> {
    implicit: ConicImplicit,
    center: [f64; 2],
    axis: [f64; 2],
    radii: [f64; 2],
    bounds: Option<&'a SketchConicBounds>,
}
fn shape(curve: &SketchIncidenceCurve) -> ConicShape<'_> {
    match curve {
        SketchIncidenceCurve::Circle {
            center,
            radius,
            sector,
        } => ConicShape {
            implicit: ConicImplicit::circle(*center, *radius),
            center: *center,
            axis: [1., 0.],
            radii: [*radius; 2],
            bounds: sector.as_ref(),
        },
        SketchIncidenceCurve::Ellipse {
            center,
            major_axis,
            radii,
            sector,
        } => ConicShape {
            implicit: ConicImplicit::ellipse(*center, *major_axis, *radii),
            center: *center,
            axis: *major_axis,
            radii: *radii,
            bounds: sector.as_ref(),
        },
        other => unreachable!("curve {other:?}; expected a circle or ellipse"),
    }
}
impl ConicSource {
    pub fn new(curve: &SketchIncidenceCurve, work: &mut Work<'_>) -> Result<Self, Cause> {
        let shape = shape(curve);
        let reference_start = SourcePlace::sweep(SketchSweepChart::Reference, None);
        let (frame, start, end, angled) = match shape.bounds {
            None => (
                SweepFrame::new(shape.center, shape.axis, true),
                reference_start,
                None,
                false,
            ),
            Some(SketchConicBounds::Rays(sector)) => {
                let frame = SweepFrame::new(shape.center, sector.start, sector.counterclockwise);
                let end = frame.exact_place(&sector.end.map(ratio));
                (frame, reference_start, Some(end), false)
            }
            Some(SketchConicBounds::Angles(angles)) => {
                let frame = SweepFrame::new(shape.center, angles.axis, angles.counterclockwise);
                let [start, end] = angle_bounds(&frame, angles, shape.radii, work)?;
                (frame, start, Some(end), true)
            }
        };
        Ok(Self {
            implicit: shape.implicit,
            frame,
            start,
            end,
            angled,
        })
    }
    /// Mark a place before an angle start as following every other place.
    pub fn normalize(&self, place: &mut SourcePlace, work: &mut Work<'_>) -> Result<(), Cause> {
        if !self.angled {
            return Ok(());
        }
        let before =
            compare_places(&mut place.clone(), &mut self.start.clone(), work)? == Ordering::Less;
        if let SourcePlace::Sweep { turn, .. } = place {
            *turn = before;
        }
        Ok(())
    }
    pub fn position(
        &self,
        place: &mut SourcePlace,
        work: &mut Work<'_>,
    ) -> Result<SketchLinePosition, Cause> {
        let Some(end) = &self.end else {
            return Ok(SketchLinePosition::Interior);
        };
        if compare_places(place, &mut self.start.clone(), work)? == Ordering::Equal {
            return Ok(SketchLinePosition::AtStart);
        }
        Ok(match compare_places(place, &mut end.clone(), work)? {
            Ordering::Less => SketchLinePosition::Interior,
            Ordering::Equal => SketchLinePosition::AtEnd,
            Ordering::Greater => SketchLinePosition::AfterEnd,
        })
    }
}
fn angle_bounds(
    frame: &SweepFrame,
    angles: &SketchConicAngles,
    radii: [f64; 2],
    work: &mut Work<'_>,
) -> Result<[SourcePlace; 2], Cause> {
    let angle_frame = AngleFrame {
        axis: angles.axis,
        counterclockwise: angles.counterclockwise,
        radii,
    };
    let [start, mut end] = [0, 1].map(|index| {
        angle_place(
            frame,
            FrameAngle::new(angle_frame.clone(), angles.bounds[index]),
            work,
        )
    });
    let (start, mut end_place) = (start?, end.take_place()?);
    let before =
        compare_places(&mut end_place.clone(), &mut start.clone(), work)? == Ordering::Less;
    if let SourcePlace::Sweep { turn, .. } = &mut end_place {
        *turn = before;
    }
    Ok([start, end_place])
}
trait TakePlace {
    fn take_place(&mut self) -> Result<SourcePlace, Cause>;
}
impl TakePlace for Result<SourcePlace, Cause> {
    fn take_place(&mut self) -> Result<SourcePlace, Cause> {
        std::mem::replace(self, Err(Cause::Invariant("place taken twice")))
    }
}
/// Place of a frame angle's direction in a sweep frame; `theta = 0` is exact.
pub(super) fn angle_place(
    frame: &SweepFrame,
    angle: FrameAngle,
    work: &mut Work<'_>,
) -> Result<SourcePlace, Cause> {
    if let Some(direction) = angle.exact_direction() {
        return Ok(frame.exact_place(&direction));
    }
    let mut key = AngleKey {
        angle,
        reference: frame.reference.clone(),
        counterclockwise: frame.counterclockwise,
    };
    // A nonzero angle's direction is never on an algebraic line, so `across` separates.
    loop {
        work.charge()?;
        let [across, _] = key.across_along();
        let sign = if across.lower.sign() == Ordering::Greater {
            Ordering::Greater
        } else if across.upper.sign() == Ordering::Less {
            Ordering::Less
        } else {
            key.angle.refine();
            continue;
        };
        let cotangent = Some(KeyValue::Angle(Box::new(key)));
        return Ok(SourcePlace::sweep(chart(sign, Ordering::Equal), cotangent));
    }
}
