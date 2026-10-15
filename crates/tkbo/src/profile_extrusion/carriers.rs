// SPDX-License-Identifier: GPL-2.0-only
//! Native carriers of caller curves in world coordinates: `gp` circles and ellipses whose
//! parameter increases in the arc sense, and `Geom_BSplineCurve` splines (ADR 0083).
use super::admitted::{CircleCarrier, WorldFrame};
use super::ellipse::EllipseCarrier;
use super::profile::ArcSense;
use super::spline::AdmittedSpline;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Ellipse3, Point3};

pub(crate) fn direction([x, y, z]: [f64; 3]) -> Result<Direction3, Failure> {
    Direction3::new(x, y, z)
}

/// Frame at `center` whose x is `x_direction` and whose normal follows the arc sense.
fn sensed_axis(
    frame: &WorldFrame,
    center: [f64; 2],
    x_direction: [f64; 3],
    sense: ArcSense,
) -> Result<Axis2, Failure> {
    let [x, y, z] = frame.world(center);
    let normal = match sense {
        ArcSense::CounterClockwise => frame.normal,
        ArcSense::Clockwise => frame.normal.map(|c| -c),
    };
    Axis2::with_x_direction(
        Point3::new(x, y, z),
        direction(normal)?,
        direction(x_direction)?,
    )
}

/// World circle whose native parameter increases in the arc sense; parameter 0 lies on +x.
pub(crate) fn circle(
    frame: &WorldFrame,
    carrier: CircleCarrier,
    sense: ArcSense,
) -> Result<Circle3, Failure> {
    let axis = sensed_axis(frame, carrier.center, frame.x, sense)?;
    Circle3::new(axis, carrier.radius)
}

/// World ellipse whose native parameter increases in the arc sense; parameter 0 lies at the
/// end of the major semi-axis.
pub(crate) fn ellipse(
    frame: &WorldFrame,
    carrier: EllipseCarrier,
    sense: ArcSense,
) -> Result<Ellipse3, Failure> {
    let major = frame.direction(carrier.axis);
    let axis = sensed_axis(frame, carrier.center, major, sense)?;
    Ellipse3::new(axis, carrier.major, carrier.minor)
}

/// World B-spline with the admitted poles (end poles at the vertices), weights and knots.
pub(crate) fn spline(
    frame: &WorldFrame,
    spline: &AdmittedSpline,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let points: Vec<Point3> = spline
        .poles
        .iter()
        .map(|p| {
            let [x, y, z] = frame.world(*p);
            Point3::new(x, y, z)
        })
        .collect();
    let (knots, mults) = spline.distinct_knots();
    let (poles, knots, mults) = (
        Array1::from_shared(&points, 1),
        Array1::from_shared(&knots, 1),
        Array1::from_shared(&mults, 1),
    );
    let degree = spline.degree as i32;
    let curve = match &spline.weights {
        None => GeomBSplineCurve::new(&poles, &knots, &mults, degree, false)?,
        Some(weights) => {
            let weights = Array1::from_shared(weights, 1);
            GeomBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, false)?
        }
    };
    Ok(Handle::new(curve).cast())
}
