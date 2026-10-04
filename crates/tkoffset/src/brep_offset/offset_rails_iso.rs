//! Exact 3D images of elementary-surface iso p-curves on `BRepOffset_Offset` rails.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Direction2;

/// Stores exact elementary isos when a rail connector is a parameter-plane line.
pub(super) fn store_exact_iso_curve(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
) -> Result<bool, Failure> {
    let representation = BRepTool::first_curve_on_surface(edge)?;
    let Some(line) = line_basis(&representation.pcurve) else {
        return Ok(false);
    };
    let Some(basis) = elementary_basis(surface)? else {
        return Ok(false);
    };
    let point = line.value(representation.first)?;
    let direction = line.direction();
    let curve = if direction.is_equal(Direction2::X, precision::angular()) {
        basis.v_iso(point.y())?
    } else if direction.is_equal(Direction2::Y, precision::angular()) {
        basis.u_iso(point.x())?
    } else {
        return Ok(false);
    };
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

fn line_basis(curve: &Handle<dyn Geom2dCurve>) -> Option<Handle<Geom2dLine>> {
    let line = curve.cast::<Geom2dLine>();
    if let Some(line) = line.get() {
        return Some(Handle::new(Geom2dLine::from_point(
            line.location(),
            line.direction(),
        )));
    }
    let trimmed = curve.cast::<Geom2dTrimmedCurve>();
    let basis = trimmed.get()?.basis_curve();
    let line = basis.cast::<Geom2dLine>();
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
