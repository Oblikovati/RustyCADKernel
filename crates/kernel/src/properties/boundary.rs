// SPDX-License-Identifier: GPL-2.0-only
//! The trims of a face in its forward parameter plane, classified for closed-form use.
//! Edge orientation picks the seam side (`BRepTool::curve_on_face`) and the traversal
//! direction; internal and external edges bound no material and are skipped.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::CurveType;

/// The exact shape of a p-curve where it admits closed-form integrals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TrimShape {
    /// `origin + t direction`.
    Line {
        origin: [f64; 2],
        direction: [f64; 2],
    },
    /// `centre + radius (cos t x + sin t y)`.
    Circle {
        centre: [f64; 2],
        x: [f64; 2],
        y: [f64; 2],
        radius: f64,
    },
    /// Any other p-curve: integrated numerically.
    General,
}

/// One oriented use of an edge's p-curve.
pub(crate) struct TrimUse {
    pub(crate) shape: TrimShape,
    pub(crate) curve: Geom2dAdaptorCurve,
    pub(crate) first: f64,
    pub(crate) last: f64,
    /// `+1` traversed first to last, `-1` last to first.
    pub(crate) sign: f64,
}

impl TrimUse {
    /// The p-curve point and derivative at `t`; `trim.d1(trim.first)?`.
    pub(crate) fn d1(&self, t: f64) -> Result<([f64; 2], [f64; 2]), Failure> {
        let (point, tangent) = self.curve.d1(t)?;
        Ok(([point.x(), point.y()], [tangent.x(), tangent.y()]))
    }

    /// `end - start` in the direction of traversal.
    fn displacement(&self) -> Result<[f64; 2], Failure> {
        let (start, _) = self.d1(self.first)?;
        let (end, _) = self.d1(self.last)?;
        Ok([0, 1].map(|k| self.sign * (end[k] - start[k])))
    }
}

/// The trims of `face` taken forward, one list per wire, in exploration order.
/// Example: `face_loops(&face)?.len()` is the number of wires.
pub(crate) fn face_loops(face: &Face) -> Result<Vec<Vec<TrimUse>>, Failure> {
    let forward = topods::face(&face.oriented(Orientation::Forward))?;
    let mut loops = Vec::new();
    for wire in sub_shapes(&forward, ShapeEnum::Wire)? {
        let mut uses = Vec::new();
        for edge in sub_shapes(&wire, ShapeEnum::Edge)? {
            if let Some(trim) = trim_use(&edge, &forward)? {
                uses.push(trim);
            }
        }
        loops.push(uses);
    }
    Ok(loops)
}

/// Whether the trims of one wire close in the parameter plane (Green's theorem applies).
/// A periodic face without seam edges does not close and is integrated natively.
pub(crate) fn closes(uses: &[TrimUse]) -> Result<bool, Failure> {
    let (mut gap, mut extent) = ([0.0, 0.0], 1.0_f64);
    for trim in uses {
        let step = trim.displacement()?;
        gap = [gap[0] + step[0], gap[1] + step[1]];
        extent = extent.max(step[0].abs()).max(step[1].abs());
    }
    Ok(gap[0].hypot(gap[1]) <= 1e-9 * extent)
}

/// Every sub-shape of `kind`, orientation composed; `sub_shapes(&face, ShapeEnum::Wire)?`.
pub(crate) fn sub_shapes(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut found = Vec::new();
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(found)
}

fn trim_use(edge: &Shape, face: &Face) -> Result<Option<TrimUse>, Failure> {
    let sign = match edge.orientation() {
        Orientation::Forward => 1.0,
        Orientation::Reversed => -1.0,
        _ => return Ok(None),
    };
    let pcurve = BRepTool::curve_on_face(&topods::edge(edge)?, face)?;
    if pcurve.pcurve.is_null() {
        return Err(missing_pcurve(edge));
    }
    let curve = Geom2dAdaptorCurve::with_curve(&pcurve.pcurve)?;
    Ok(Some(TrimUse {
        shape: classify(&curve)?,
        curve,
        first: pcurve.first,
        last: pcurve.last,
        sign,
    }))
}

fn classify(curve: &Geom2dAdaptorCurve) -> Result<TrimShape, Failure> {
    Ok(match curve.get_type()? {
        CurveType::Line => {
            let line = curve.line()?;
            let (origin, direction) = (line.location(), line.direction());
            TrimShape::Line {
                origin: [origin.x(), origin.y()],
                direction: [direction.x(), direction.y()],
            }
        }
        CurveType::Circle => circle_shape(curve)?,
        _ => TrimShape::General,
    })
}

fn circle_shape(curve: &Geom2dAdaptorCurve) -> Result<TrimShape, Failure> {
    let circle = curve.circle()?;
    let frame = circle.position();
    let (centre, x, y) = (frame.location(), frame.x_direction(), frame.y_direction());
    Ok(TrimShape::Circle {
        centre: [centre.x(), centre.y()],
        x: [x.x(), x.y()],
        y: [y.x(), y.y()],
        radius: circle.radius(),
    })
}

fn missing_pcurve(edge: &Shape) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!(
            "edge {:?} has no p-curve on its face; expected a stored or planar-projected curve",
            edge.orientation()
        ),
        "",
    )
}
