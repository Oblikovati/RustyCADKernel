// SPDX-License-Identifier: GPL-2.0-only
//! Admission: every caller value is checked before any geometry is constructed.
use super::admitted::*;
use super::decline::{DeclineCause, DeclineCause as Cause};
use super::plane_math::*;
use super::profile::*;
use super::resolution::Resolution;
use std::collections::BTreeMap;

type Admit<T> = Result<T, DeclineCause>;

/// Check `profile` under `resolution`; `admit_profile(&profile, resolution)?`.
pub(crate) fn admit_profile(
    profile: &SketchProfile,
    resolution: Resolution,
) -> Admit<AdmittedProfile> {
    admit_resolution(&resolution)?;
    let frame = admit_frame(&profile.plane, resolution.angular)?;
    if profile.curves.is_empty() {
        return Err(Cause::EmptyProfile);
    }
    let vertices = admit_vertices(&profile.vertices)?;
    let curves = admit_curves(&profile.curves, &vertices, &resolution)?;
    require_separated_vertices(&curves, &vertices, resolution.weld)?;
    Ok(AdmittedProfile {
        frame,
        resolution,
        vertices,
        curves,
    })
}

pub(crate) fn admit_resolution(resolution: &Resolution) -> Admit<()> {
    resolution
        .admit()
        .map_err(|(field, value)| Cause::InvalidResolution { field, value })
}

/// Orthonormal frame; the x direction must be orthogonal to the normal within `angular`.
pub(crate) fn admit_frame(plane: &ProfilePlane, angular: f64) -> Admit<WorldFrame> {
    if !plane.origin.iter().all(|c| c.is_finite()) {
        return Err(Cause::InvalidPlane {
            reason: "nonfinite origin",
        });
    }
    let normal = unit3(plane.normal).ok_or(Cause::InvalidPlane {
        reason: "zero normal",
    })?;
    let x = unit3(plane.x_direction).ok_or(Cause::InvalidPlane {
        reason: "zero x direction",
    })?;
    if dot3(x, normal).abs() > angular.sin() {
        return Err(Cause::InvalidPlane {
            reason: "x direction not orthogonal to normal",
        });
    }
    let y = unit3(cross3(normal, x)).ok_or(Cause::InvalidPlane {
        reason: "degenerate frame",
    })?;
    Ok(WorldFrame {
        origin: plane.origin,
        x: cross3(y, normal),
        y,
        normal,
    })
}

fn admit_vertices(vertices: &[ProfileVertex]) -> Admit<BTreeMap<ProfileVertexId, [f64; 2]>> {
    let mut admitted = BTreeMap::new();
    for vertex in vertices {
        if !vertex.point.iter().all(|c| c.is_finite()) {
            return Err(Cause::NonFiniteVertex { vertex: vertex.id });
        }
        if admitted.insert(vertex.id, vertex.point).is_some() {
            return Err(Cause::DuplicateVertex { vertex: vertex.id });
        }
    }
    Ok(admitted)
}

fn admit_curves(
    curves: &[ProfileCurve],
    vertices: &BTreeMap<ProfileVertexId, [f64; 2]>,
    resolution: &Resolution,
) -> Admit<Vec<AdmittedCurve>> {
    let mut seen = std::collections::BTreeSet::new();
    let mut admitted = Vec::with_capacity(curves.len());
    for curve in curves {
        if !seen.insert(curve.id()) {
            return Err(Cause::DuplicateCurve { curve: curve.id() });
        }
        require_known_ends(curve, vertices)?;
        admitted.push(admit_curve(curve, vertices, resolution)?);
    }
    Ok(admitted)
}

fn require_known_ends(
    curve: &ProfileCurve,
    vertices: &BTreeMap<ProfileVertexId, [f64; 2]>,
) -> Admit<()> {
    let ends = curve.ends().into_iter().flatten();
    match ends
        .into_iter()
        .find(|vertex| !vertices.contains_key(vertex))
    {
        Some(vertex) => Err(Cause::UnknownVertex {
            curve: curve.id(),
            vertex,
        }),
        None => Ok(()),
    }
}

fn admit_curve(
    curve: &ProfileCurve,
    vertices: &BTreeMap<ProfileVertexId, [f64; 2]>,
    resolution: &Resolution,
) -> Admit<AdmittedCurve> {
    let point = |vertex: &ProfileVertexId| vertices[vertex];
    match curve {
        ProfileCurve::Line { id, start, end } => {
            admit_line(*id, [*start, *end], [point(start), point(end)], resolution)
        }
        ProfileCurve::Arc {
            id,
            start,
            end,
            center,
            sense,
        } => {
            let arc = ArcInput {
                id: *id,
                ends: [*start, *end],
                center: *center,
                sense: *sense,
            };
            admit_arc(arc, [point(start), point(end)], resolution)
        }
        ProfileCurve::Circle { id, center, radius } => {
            admit_circle(*id, *center, *radius, resolution)
        }
        ProfileCurve::Ellipse { id, .. } => unsupported(*id, "ellipse"),
        ProfileCurve::Spline { id, .. } => unsupported(*id, "spline"),
    }
}

fn unsupported(curve: ProfileCurveId, kind: &'static str) -> Admit<AdmittedCurve> {
    Err(Cause::UnsupportedCurve { curve, kind })
}

fn admit_line(
    id: ProfileCurveId,
    ends: [ProfileVertexId; 2],
    points: [[f64; 2]; 2],
    resolution: &Resolution,
) -> Admit<AdmittedCurve> {
    let size = length2(sub2(points[1], points[0]));
    if ends[0] == ends[1] || size < resolution.weld {
        return Err(Cause::DegenerateCurve {
            curve: id,
            size,
            weld: resolution.weld,
        });
    }
    Ok(AdmittedCurve::Line { id, ends })
}

struct ArcInput {
    id: ProfileCurveId,
    ends: [ProfileVertexId; 2],
    center: [f64; 2],
    sense: ArcSense,
}

fn admit_arc(
    arc: ArcInput,
    points: [[f64; 2]; 2],
    resolution: &Resolution,
) -> Admit<AdmittedCurve> {
    if !arc.center.iter().all(|c| c.is_finite()) {
        return Err(Cause::NonFiniteCurve { curve: arc.id });
    }
    let radii = points.map(|point| length2(sub2(point, arc.center)));
    if (radii[0] - radii[1]).abs() > resolution.sew {
        return Err(Cause::ArcRadiusMismatch {
            curve: arc.id,
            radii,
            sew: resolution.sew,
        });
    }
    let [a0, a1] = points.map(|point| polar_angle(arc.center, point));
    let sweep = signed_sweep(a0, a1, arc.sense).abs();
    let size = radii[0].min(radii[1]) * sweep.min(1.0);
    if arc.ends[0] == arc.ends[1] || size < resolution.weld || sweep < resolution.angular {
        return Err(Cause::DegenerateCurve {
            curve: arc.id,
            size,
            weld: resolution.weld,
        });
    }
    let carrier = CircleCarrier {
        center: arc.center,
        radius: 0.5 * (radii[0] + radii[1]),
    };
    Ok(AdmittedCurve::Arc {
        id: arc.id,
        ends: arc.ends,
        carrier,
        sense: arc.sense,
    })
}

fn admit_circle(
    id: ProfileCurveId,
    center: [f64; 2],
    radius: f64,
    resolution: &Resolution,
) -> Admit<AdmittedCurve> {
    if !(center.iter().all(|c| c.is_finite()) && radius.is_finite()) {
        return Err(Cause::NonFiniteCurve { curve: id });
    }
    if radius < resolution.weld {
        return Err(Cause::DegenerateCurve {
            curve: id,
            size: radius,
            weld: resolution.weld,
        });
    }
    Ok(AdmittedCurve::Circle {
        id,
        carrier: CircleCarrier { center, radius },
    })
}

/// Distinct used vertices closer than `weld` would merge silently; decline instead.
fn require_separated_vertices(
    curves: &[AdmittedCurve],
    vertices: &BTreeMap<ProfileVertexId, [f64; 2]>,
    weld: f64,
) -> Admit<()> {
    let used: std::collections::BTreeSet<_> =
        curves.iter().filter_map(|c| c.ends()).flatten().collect();
    let used: Vec<_> = used.into_iter().map(|id| (id, vertices[&id])).collect();
    for (index, (first, a)) in used.iter().enumerate() {
        for (second, b) in &used[index + 1..] {
            let distance = length2(sub2(*a, *b));
            if distance < weld {
                return Err(Cause::CoincidentVertices {
                    vertices: [*first, *second],
                    distance,
                    weld,
                });
            }
        }
    }
    Ok(())
}
