//! Segment geometry sampling keeps the native endpoint-before-start evaluation order.
use super::*;
pub(super) struct PieceSamples {
    pub end: Point3,
    pub start: Point3,
    pub middle: Point3,
    pub first: f64,
    pub last: f64,
}
pub(super) fn at(curves: &DividedCurves, index: usize) -> Result<PieceGeometry, Failure> {
    let spatial = piece_at(&curves.spatial, index)?;
    let planar = piece_at(&curves.planar, index)?;
    let reversed = if curves.seam {
        state::require(&curves.reversed, "reverse seam pieces")?;
        piece_at(&curves.reversed, index)?
    } else {
        Handle::default()
    };
    Ok(PieceGeometry {
        spatial,
        planar,
        reversed,
    })
}
fn piece_at<T: rustycad_tkernel::standard::RuntimeTyped + Send + Sync + ?Sized>(
    array: &Handle<rustycad_tkernel::ncollection::HArray1<Handle<T>>>,
    index: usize,
) -> Result<Handle<T>, Failure> {
    let Some(array) = array.get() else {
        return Ok(Handle::default());
    };
    array
        .array1()
        .value(index as i32)
        .cloned()
        .map_err(|error| {
            Failure::with_type(
                "Standard_OutOfRange",
                &format!("WireDivide piece {index}: {error}; expected array bounds"),
                "",
            )
        })
}
pub(super) fn bounds(
    curve_bounds: (f64, f64),
    bounded: bool,
    knots: &split_values::Knots,
    first: usize,
    last: usize,
) -> Result<(f64, f64), Failure> {
    if bounded {
        return Ok(curve_bounds);
    }
    Ok((
        split_values::value(knots, first)?,
        split_values::value(knots, last)?,
    ))
}
pub(super) fn samples(
    support: &mut WireSupport,
    piece: &PieceGeometry,
    curves: &DividedCurves,
    index: usize,
) -> Result<PieceSamples, Failure> {
    if !piece.spatial.is_null() {
        return spatial_samples(piece, curves, index);
    }
    planar_samples(support, piece, curves, index)
}
fn spatial_samples(
    piece: &PieceGeometry,
    curves: &DividedCurves,
    index: usize,
) -> Result<PieceSamples, Failure> {
    let range = piece.spatial_bounds(curves, index, index + 1)?.1;
    Ok(PieceSamples {
        end: piece.spatial.value(range.1)?,
        start: piece.spatial.value(range.0)?,
        middle: piece.spatial.value((range.0 + range.1) / 2.)?,
        first: range.0,
        last: range.1,
    })
}
fn planar_samples(
    support: &mut WireSupport,
    piece: &PieceGeometry,
    curves: &DividedCurves,
    index: usize,
) -> Result<PieceSamples, Failure> {
    if support.surface.is_null() {
        (support.surface, support.location) = BRepTool::surface_with_location(&support.face)?;
    }
    state::require(&piece.planar, "segment pcurve")?;
    let range = piece.planar_bounds(curves, index, index + 1)?.1;
    let end = piece.planar.value(range.1)?;
    let start = piece.planar.value(range.0)?;
    let middle = piece.planar.value((range.0 + range.1) / 2.)?;
    Ok(PieceSamples {
        end: surface_point(support, end)?,
        start: surface_point(support, start)?,
        middle: surface_point(support, middle)?,
        first: range.0,
        last: range.1,
    })
}
fn surface_point(
    support: &WireSupport,
    point: rustycad_tkmath::gp::Point2,
) -> Result<Point3, Failure> {
    state::require(&support.surface, "supporting surface")?;
    Ok(support
        .surface
        .value(point.x(), point.y())?
        .transformed(support.location.transformation()))
}

impl PieceGeometry {
    pub(super) fn spatial_bounds(
        &self,
        curves: &DividedCurves,
        first: usize,
        last: usize,
    ) -> Result<(bool, (f64, f64)), Failure> {
        let bounded = self.spatial.is_kind_name("Geom_BoundedCurve");
        let range = bounds(
            (
                self.spatial.first_parameter(),
                self.spatial.last_parameter(),
            ),
            bounded,
            &curves.knots3,
            first,
            last,
        )?;
        Ok((bounded, range))
    }
    pub(super) fn planar_bounds(
        &self,
        curves: &DividedCurves,
        first: usize,
        last: usize,
    ) -> Result<(bool, (f64, f64)), Failure> {
        let bounded = self.planar.is_kind_name("Geom2d_BoundedCurve");
        let range = bounds(
            (self.planar.first_parameter(), self.planar.last_parameter()),
            bounded,
            &curves.knots2,
            first,
            last,
        )?;
        Ok((bounded, range))
    }
}
