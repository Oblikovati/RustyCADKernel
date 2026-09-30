// Rust translation of ModelingAlgorithms/TKShHealing/ShapeFix/ShapeFix_ShapeTolerance.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeFix_ShapeTolerance`: limits or forces the tolerances of vertices, edges and faces.
use rustycad_tkbrep::brep::access::{edge_tshape, face_tshape, vertex_tshape};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `ShapeFix_ShapeTolerance`: writes tolerances straight into the TShapes of the
/// sub-shapes (not through `BRep_Builder`, so a tolerance can also decrease).
///
/// `styp` selects the sub-shapes: `Vertex`, `Edge` or `Face` only those, `Wire` the edges
/// and their vertices, any other value the vertices, edges and faces.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// use rustycad_tkshhealing::shape_fix::ShapeFixShapeTolerance;
/// let tool = ShapeFixShapeTolerance::new();
/// assert!(!tool.limit_tolerance(&Shape::default(), 1e-7, 1e-3, ShapeEnum::Shape).unwrap());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeFixShapeTolerance;

const MEMBER: &str = "ShapeFix_ShapeTolerance";

/// The TShape tolerance of a vertex, edge or face.
fn tolerance_of(shape: &Shape, kind: ShapeEnum) -> Result<f64, Failure> {
    Ok(match kind {
        ShapeEnum::Vertex => vertex_tshape(shape, MEMBER)?.tolerance(),
        ShapeEnum::Edge => edge_tshape(shape, MEMBER)?.tolerance(),
        _ => face_tshape(shape, MEMBER)?.tolerance(),
    })
}

/// Writes the TShape tolerance of a vertex, edge or face.
fn write_tolerance(shape: &Shape, kind: ShapeEnum, tolerance: f64) -> Result<(), Failure> {
    match kind {
        ShapeEnum::Vertex => vertex_tshape(shape, MEMBER)?.set_tolerance(tolerance),
        ShapeEnum::Edge => edge_tshape(shape, MEMBER)?.set_tolerance(tolerance),
        _ => face_tshape(shape, MEMBER)?.set_tolerance(tolerance),
    }
    Ok(())
}

/// The vertex, edge and face kinds, handled one by one.
fn is_simple_kind(kind: ShapeEnum) -> bool {
    matches!(kind, ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face)
}

impl ShapeFixShapeTolerance {
    /// `ShapeFix_ShapeTolerance()`; `ShapeFixShapeTolerance::new()`.
    pub fn new() -> Self {
        Self
    }

    /// `LimitTolerance(shape, tmin, tmax, styp)`: raises the tolerances below `tmin` to it
    /// and, when `tmax >= tmin`, lowers those above `tmax` to it; true when a tolerance
    /// changed (false for a null shape or a negative `tmin`). With `Wire`, natively only
    /// the vertex changes count in the result, not the edge ones;
    /// `tool.limit_tolerance(&shape, 1e-7, 1e-3, ShapeEnum::Shape)?`.
    pub fn limit_tolerance(
        &self,
        shape: &Shape,
        tmin: f64,
        tmax: f64,
        styp: ShapeEnum,
    ) -> Result<bool, Failure> {
        if shape.is_null() || tmin < 0.0 {
            return Ok(false);
        }
        if is_simple_kind(styp) {
            return limit_kind(shape, tmin, tmax, styp);
        }
        if styp == ShapeEnum::Wire {
            return self.limit_wire(shape, tmin, tmax);
        }
        let mut fait = false;
        for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
            fait |= self.limit_tolerance(shape, tmin, tmax, kind)?;
        }
        Ok(fait)
    }

    /// The `WIRE` branch of `LimitTolerance`.
    fn limit_wire(&self, shape: &Shape, tmin: f64, tmax: f64) -> Result<bool, Failure> {
        let mut fait = false;
        for current in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = topods::edge(&current)?;
            self.limit_tolerance(&edge, tmin, tmax, ShapeEnum::Edge)?;
            let (v1, v2) = topexp::vertices(&edge, false);
            for v in [v1, v2].iter().filter(|v| !v.is_null()) {
                fait |= self.limit_tolerance(v, tmin, tmax, ShapeEnum::Vertex)?;
            }
        }
        Ok(fait)
    }

    /// `SetTolerance(shape, preci, styp)`: forces the tolerances of the selected
    /// sub-shapes to `preci` (nothing for a null shape or a non-positive `preci`);
    /// `tool.set_tolerance(&shape, 1e-5, ShapeEnum::Shape)?`.
    pub fn set_tolerance(&self, shape: &Shape, preci: f64, styp: ShapeEnum) -> Result<(), Failure> {
        if shape.is_null() || preci <= 0.0 {
            return Ok(());
        }
        if is_simple_kind(styp) {
            for current in Explorer::new(shape, styp, ShapeEnum::Shape)? {
                write_tolerance(&current, styp, preci)?;
            }
            return Ok(());
        }
        if styp == ShapeEnum::Wire {
            for current in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
                write_tolerance(&current, ShapeEnum::Edge, preci)?;
                let (v1, v2) = topexp::vertices(&topods::edge(&current)?, false);
                for v in [v1, v2].iter().filter(|v| !v.is_null()) {
                    write_tolerance(v, ShapeEnum::Vertex, preci)?;
                }
            }
            return Ok(());
        }
        for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
            self.set_tolerance(shape, preci, kind)?;
        }
        Ok(())
    }
}

/// The `VERTEX`/`EDGE`/`FACE` branch of `LimitTolerance`.
fn limit_kind(shape: &Shape, tmin: f64, tmax: f64, kind: ShapeEnum) -> Result<bool, Failure> {
    let iamax = tmax >= tmin;
    let mut fait = false;
    for current in Explorer::new(shape, kind, ShapeEnum::Shape)? {
        let prec = tolerance_of(&current, kind)?;
        let target = if iamax && prec > tmax {
            tmax
        } else if prec < tmin {
            tmin
        } else {
            continue;
        };
        write_tolerance(&current, kind, target)?;
        fait = true;
    }
    Ok(fait)
}
