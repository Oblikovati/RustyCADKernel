//! Native 23-point planar pcurve deviation pass (ShapeFix.cxx, ProSTEP TR8).
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{self, Edge, Face, Shape, ShapeIterator},
};
use rustycad_tkernel::{
    message::ProgressScope,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface},
    geom::{GeomCurve, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
    top_abs::ShapeEnum,
};

pub(super) fn update_faces(shape: &Shape, scope: &ProgressScope<'_>) -> Result<bool, Failure> {
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() && scope.more() {
        let face = topods::face(faces.current())?;
        update_face(&face)?;
        faces.next()?;
        scope.next(1.);
    }
    Ok(scope.more())
}
fn planar_surface(face: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surface = BRepTool::surface(face)?;
    if !surface.cast::<GeomPlane>().is_null() {
        return Ok(surface);
    }
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    if trimmed.is_null() {
        return Ok(Handle::default());
    }
    let basis = trimmed.basis_surface();
    Ok(basis.cast::<GeomPlane>().cast())
}
fn update_face(face: &Face) -> Result<bool, Failure> {
    let plane = planar_surface(face)?;
    if plane.is_null() {
        return Ok(false);
    }
    let adaptor: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plane)?).cast();
    let mut edges = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while edges.more() {
        update_edge(&topods::edge(edges.current())?, face, &adaptor)?;
        edges.next()?;
    }
    Ok(true)
}
pub(super) fn update_edge(
    edge: &Edge,
    face: &Face,
    surface: &Handle<dyn Adaptor3dSurface>,
) -> Result<(), Failure> {
    let (curve, _, _) = BRepTool::curve(edge)?;
    if curve.is_null() {
        return Ok(());
    }
    let pcurve = BRepTool::curve_on_face(edge, face)?;
    if pcurve.pcurve.is_null() {
        return Ok(());
    }
    let projected = projected_curve(&pcurve, surface)?;
    let original = BRepTool::edge_tolerance(edge)?;
    let square_tolerance =
        sampled_tolerance(&curve, &projected, (pcurve.first, pcurve.last), original)?;
    if square_tolerance > original * original {
        grow_tolerance(edge, 1.00005 * square_tolerance.sqrt())?;
    }
    Ok(())
}
fn grow_tolerance(edge: &Edge, tolerance: f64) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.update_edge_tolerance(edge, tolerance)?;
    let mut vertices = ShapeIterator::new(edge, true, true);
    while vertices.more() {
        builder.update_vertex_tolerance(&topods::vertex(vertices.value()?)?, tolerance)?;
        vertices.next();
    }
    Ok(())
}

fn projected_curve(
    pcurve: &rustycad_tkbrep::brep::EdgePCurve,
    surface: &Handle<dyn Adaptor3dSurface>,
) -> Result<Adaptor3dCurveOnSurface, Failure> {
    let adaptor = Handle::new(Geom2dAdaptorCurve::with_range(
        &pcurve.pcurve,
        pcurve.first,
        pcurve.last,
    )?)
    .cast();
    Adaptor3dCurveOnSurface::with_curve(&adaptor, surface)
}
fn sampled_tolerance(
    curve: &Handle<dyn GeomCurve>,
    projected: &Adaptor3dCurveOnSurface,
    (first, last): (f64, f64),
    original: f64,
) -> Result<f64, Failure> {
    let mut square_tolerance = original * original;
    for index in 0..23 {
        let parameter = (first * f64::from(22 - index) + last * f64::from(index)) / 22.;
        let distance = curve
            .value(parameter)?
            .square_distance(&projected.value(parameter)?);
        if square_tolerance < distance {
            square_tolerance = distance;
        }
    }
    Ok(square_tolerance)
}
