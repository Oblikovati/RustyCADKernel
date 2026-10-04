//! Exact 3D images of elementary-surface iso p-curves on `BRepOffset_Offset` rails.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Direction2;

type IsoParameter = (bool, f64);

/// Stores exact elementary isos emitted by the offset rail connector builder.
///
/// The native connector path uses positive axis-aligned p-curves with a zero
/// running-coordinate origin on the exact support handle. Other valid p-curves
/// use the general curve reconstruction path because their parameterization
/// cannot be inferred from a surface iso alone.
pub(super) fn store_exact_iso_curve(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
) -> Result<bool, Failure> {
    let representation = BRepTool::first_curve_on_surface(edge)?;
    if !is_target_surface(&representation.surface, &representation.location, surface) {
        return Ok(false);
    }
    let Some(line) = line_basis(&representation.pcurve) else {
        return Ok(false);
    };
    let Some(basis) = elementary_basis(surface)? else {
        return Ok(false);
    };
    let Some((varies_u, constant)) = iso_parameter(&line, representation.first)? else {
        return Ok(false);
    };
    let curve = iso_curve(&basis, varies_u, constant)?;
    // OCCT retains these pipe boundary isos as analytic curves before vertex projection.
    store_curve_range(
        edge,
        &curve,
        &representation.location,
        representation.first,
        representation.last,
    )?;
    Ok(true)
}

fn is_target_surface(
    represented: &Handle<dyn GeomSurface>,
    location: &rustycad_tkmath::toploc::Location,
    target: &Handle<dyn GeomSurface>,
) -> bool {
    represented.object_address() == target.object_address() && location.is_identity()
}

fn iso_parameter(line: &Geom2dLine, first: f64) -> Result<Option<IsoParameter>, Failure> {
    let point = line.value(first)?;
    let origin = line.location();
    let direction = line.direction();
    if direction.is_equal(Direction2::X, precision::angular())
        && origin.x().abs() <= precision::p_confusion()
        && (point.x() - first).abs() <= precision::p_confusion()
    {
        return Ok(Some((true, point.y())));
    }
    if direction.is_equal(Direction2::Y, precision::angular())
        && origin.y().abs() <= precision::p_confusion()
        && (point.y() - first).abs() <= precision::p_confusion()
    {
        return Ok(Some((false, point.x())));
    }
    Ok(None)
}

fn iso_curve(
    surface: &Handle<dyn GeomSurface>,
    varies_u: bool,
    constant: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    if varies_u {
        return surface.v_iso(constant);
    }
    surface.u_iso(constant)
}

fn line_basis(curve: &Handle<dyn Geom2dCurve>) -> Option<Handle<Geom2dLine>> {
    let line = curve.cast::<Geom2dLine>();
    line.get()
        .map(|line| Handle::new(Geom2dLine::from_point(line.location(), line.direction())))
}

fn elementary_basis(
    surface: &Handle<dyn GeomSurface>,
) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    let basis = surface
        .cast::<GeomRectangularTrimmedSurface>()
        .get()
        .map(|trimmed| trimmed.basis_surface())
        .unwrap_or_else(|| surface.clone());
    let kind = GeomAdaptorSurface::with_surface(&basis)?.get_type()?;
    if matches!(
        kind,
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Torus
    ) {
        return Ok(Some(basis));
    }
    Ok(None)
}

fn store_curve_range(
    edge: &Edge,
    curve: &Handle<dyn GeomCurve>,
    location: &rustycad_tkmath::toploc::Location,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.update_edge_located_curve(edge, curve, location, precision::approximation())?;
    builder.range(edge, first, last, false)
}

#[cfg(test)]
mod tests {
    use super::iso_parameter;
    use rustycad_tkg2d::geom2d::Geom2dLine;
    use rustycad_tkmath::gp::{Direction2, Point2};

    #[test]
    fn accepts_only_native_positive_axis_lines_with_zero_running_origin() {
        let along_u = Geom2dLine::from_point(Point2::new(0.0, 2.0), Direction2::X);
        let along_v = Geom2dLine::from_point(Point2::new(3.0, 0.0), Direction2::Y);

        assert_eq!(iso_parameter(&along_u, 1.5).unwrap(), Some((true, 2.0)));
        assert_eq!(iso_parameter(&along_v, 1.5).unwrap(), Some((false, 3.0)));
    }

    #[test]
    fn rejects_shifted_reversed_and_diagonal_pcurves() {
        let shifted = Geom2dLine::from_point(Point2::new(0.1, 2.0), Direction2::X);
        let reversed = Geom2dLine::from_point(Point2::new(0.0, 2.0), Direction2::X.reversed());
        let diagonal =
            Geom2dLine::from_point(Point2::default(), Direction2::new(1.0, 1.0).unwrap());

        assert!(iso_parameter(&shifted, 1.5).unwrap().is_none());
        assert!(iso_parameter(&reversed, 1.5).unwrap().is_none());
        assert!(iso_parameter(&diagonal, 1.5).unwrap().is_none());
    }
}
