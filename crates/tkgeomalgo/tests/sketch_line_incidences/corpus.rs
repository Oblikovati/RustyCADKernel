// SPDX-License-Identifier: GPL-2.0-only
//! Shared corpus rows as certified requests and as mapped Geom2d curves.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2d, Axis22d, Direction2, Point2};
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

pub struct CorpusCase {
    pub id: usize,
    pub kind: String,
    pub source: [[f64; 2]; 2],
    pub target: SketchIncidenceCurve,
}
pub type Curve = Handle<dyn Geom2dCurve>;

fn bits(field: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(field, 16).unwrap_or_else(|_| panic!("hex {field}")))
}
pub fn values(fields: &str) -> Vec<f64> {
    fields.split(',').map(bits).collect()
}
pub fn parse_case(row: &str) -> CorpusCase {
    let fields: Vec<&str> = row.split(';').collect();
    let source = values(fields[2]);
    CorpusCase {
        id: fields[0].parse().unwrap(),
        kind: fields[1].to_owned(),
        source: [[source[0], source[1]], [source[2], source[3]]],
        target: parse_target(fields[3]),
    }
}
/// Curve spec shared by targets and curve sources.
pub fn parse_target(spec: &str) -> SketchIncidenceCurve {
    let (kind, rest) = spec.split_once(':').unwrap();
    match kind {
        "circle" => parse_circle(rest),
        "ellipse" => {
            let v = values(rest);
            SketchIncidenceCurve::Ellipse {
                center: [v[0], v[1]],
                major_axis: [v[2], v[3]],
                radii: [v[4], v[5]],
                sector: None,
            }
        }
        "segment" | "line" => {
            let v = values(rest);
            SketchIncidenceCurve::Line {
                points: [[v[0], v[1]], [v[2], v[3]]],
                bounded: kind == "segment",
            }
        }
        "spline" => parse_spline(rest),
        "offset" => {
            let (distance, basis) = rest.split_once(':').unwrap();
            SketchIncidenceCurve::Offset(SketchIncidenceOffset {
                basis: Box::new(parse_target(basis)),
                distance: bits(distance),
            })
        }
        "periodic" => parse_periodic(rest),
        other => panic!("target kind {other}; expected circle, ellipse, segment or spline"),
    }
}
// periodic:degree:poles:knots:multiplicities:bounds (ADR 0080).
fn parse_periodic(rest: &str) -> SketchIncidenceCurve {
    let fields: Vec<&str> = rest.split(':').collect();
    let poles = values(fields[1])
        .chunks(2)
        .map(|pair| [pair[0], pair[1]])
        .collect();
    let bounds = values(fields[4]);
    SketchIncidenceCurve::PeriodicSpline(SketchPeriodicSpline {
        degree: fields[0].parse().unwrap(),
        poles,
        weights: None,
        knots: values(fields[2]),
        multiplicities: fields[3].split(',').map(|m| m.parse().unwrap()).collect(),
        bounds: [bounds[0], bounds[1]],
    })
}
fn parse_circle(rest: &str) -> SketchIncidenceCurve {
    let fields: Vec<&str> = rest.split(',').collect();
    let v: Vec<f64> = fields.iter().take(7).map(|field| bits(field)).collect();
    let sector = (fields.len() == 8).then(|| SketchConicSector {
        start: [v[3], v[4]],
        end: [v[5], v[6]],
        counterclockwise: fields[7] == "1",
    });
    SketchIncidenceCurve::Circle {
        center: [v[0], v[1]],
        radius: v[2],
        sector: sector.map(Into::into),
    }
}
fn parse_spline(rest: &str) -> SketchIncidenceCurve {
    let fields: Vec<&str> = rest.split(':').collect();
    let poles = values(fields[1])
        .chunks(2)
        .map(|pair| [pair[0], pair[1]])
        .collect();
    let bounds = values(fields[3]);
    SketchIncidenceCurve::Spline(SketchIncidenceSpline {
        degree: fields[0].parse().unwrap(),
        poles,
        weights: None,
        flat_knots: values(fields[2]),
        bounds: [bounds[0], bounds[1]],
    })
}
fn point(value: [f64; 2]) -> Point2 {
    Point2::new(value[0], value[1])
}
fn direction(value: [f64; 2]) -> Direction2 {
    Direction2::new(value[0], value[1]).unwrap()
}
/// Infinite mapped line through both source points.
pub fn mapped_line(points: [[f64; 2]; 2]) -> Curve {
    let delta = [points[1][0] - points[0][0], points[1][1] - points[0][1]];
    Handle::new(Geom2dLine::from_point(point(points[0]), direction(delta))).cast()
}
/// Mapped Geom2d representation; arcs use native angle bounds, as an application would.
pub fn mapped_target(curve: &SketchIncidenceCurve) -> Curve {
    match curve {
        SketchIncidenceCurve::Line {
            points,
            bounded: true,
        } => mapped_segment(*points),
        SketchIncidenceCurve::Line { points, .. } => mapped_line(*points),
        SketchIncidenceCurve::Circle {
            center,
            radius,
            sector,
        } => mapped_circle(*center, *radius, sector.as_ref()),
        SketchIncidenceCurve::Ellipse {
            center,
            major_axis,
            radii,
            ..
        } => {
            let frame = Axis22d::new(point(*center), direction(*major_axis), true).unwrap();
            Handle::new(Geom2dEllipse::new(frame, radii[0], radii[1]).unwrap()).cast()
        }
        SketchIncidenceCurve::Spline(spline) => mapped_spline(spline),
        SketchIncidenceCurve::PeriodicSpline(_) | SketchIncidenceCurve::Offset(_) => {
            super::mapped_offsets::mapped_extension(curve)
        }
    }
}
fn mapped_segment(points: [[f64; 2]; 2]) -> Curve {
    let length = (points[1][0] - points[0][0]).hypot(points[1][1] - points[0][1]);
    let line = mapped_line(points);
    Handle::new(Geom2dTrimmedCurve::new(&line, 0., length, true, false).unwrap()).cast()
}
fn mapped_circle(center: [f64; 2], radius: f64, sector: Option<&SketchConicBounds>) -> Curve {
    let axis = Axis2d::new(point(center), direction([1., 0.]));
    let circle: Curve = Handle::new(Geom2dCircle::from_axis(axis, radius, true).unwrap()).cast();
    let sector = match sector {
        None => return circle,
        Some(SketchConicBounds::Angles(angles)) => return angle_circle(center, radius, angles),
        Some(SketchConicBounds::Rays(sector)) => sector,
    };
    let angle = |value: [f64; 2]| value[1].atan2(value[0]);
    let (mut first, mut last) = (angle(sector.start), angle(sector.end));
    if !sector.counterclockwise {
        std::mem::swap(&mut first, &mut last);
    }
    if last <= first {
        last += std::f64::consts::TAU;
    }
    Handle::new(Geom2dTrimmedCurve::new(&circle, first, last, true, false).unwrap()).cast()
}
// The same frame angles as the certified arc: OCCT trims a circle in its own frame.
fn angle_circle(center: [f64; 2], radius: f64, angles: &SketchConicAngles) -> Curve {
    let axis = Axis2d::new(point(center), direction(angles.axis));
    let circle: Curve =
        Handle::new(Geom2dCircle::from_axis(axis, radius, angles.counterclockwise).unwrap()).cast();
    let [first, last] = angles.bounds;
    Handle::new(Geom2dTrimmedCurve::new(&circle, first, last, true, false).unwrap()).cast()
}
fn mapped_spline(spline: &SketchIncidenceSpline) -> Curve {
    let mut knots: Vec<f64> = Vec::new();
    let mut multiplicities: Vec<i32> = Vec::new();
    for &knot in &spline.flat_knots {
        if knots.last() == Some(&knot) {
            *multiplicities.last_mut().unwrap() += 1;
        } else {
            knots.push(knot);
            multiplicities.push(1);
        }
    }
    let poles: Vec<Point2> = spline.poles.iter().copied().map(point).collect();
    let curve = Geom2dBSplineCurve::new(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(&knots, 1),
        &Array1::from_shared(&multiplicities, 1),
        spline.degree as i32,
        false,
    )
    .unwrap();
    Handle::new(curve).cast()
}
