// Translation of BRepOffset_SimpleOffset::FillEdgeData at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tktopalgo::brep_lib::{BRepLibValidateEdge, BUILD_CURVE3D_DEFAULTS, build_curve3d};

/// An original face and its already computed offset surface, in ancestor-map order.
#[derive(Clone)]
pub struct OffsetFaceSurface {
    /// The original face carrying the input edge p-curve.
    pub face: Face,
    /// Its transformed offset surface with identity location.
    pub surface: Handle<dyn GeomSurface>,
}

/// The curve geometry and tolerance produced for one input edge.
#[derive(Clone, Default)]
pub struct OffsetEdgeGeometry {
    /// The generated 3D curve, null if OCCT could not construct one.
    pub curve: Handle<dyn GeomCurve>,
    /// The location returned by `BRep_Tool::Curve` on the generated edge.
    pub location: Location,
    /// The largest new-edge tolerance or deviation to an adjacent offset face.
    pub tolerance: f64,
}

/// Port of `BRepOffset_SimpleOffset::FillEdgeData`: build the edge's offset curve and
/// measure its deviation on each adjacent offset face.
///
/// ```
/// use rustycad_tkoffset::brep_offset::{OffsetFaceSurface, build_simple_offset_edge};
/// let edge = rustycad_tkbrep::topods::Edge::default();
/// assert!(build_simple_offset_edge(&edge, &[]).unwrap().is_none());
/// ```
pub fn build_simple_offset_edge(
    edge: &Edge,
    adjacent_faces: &[OffsetFaceSurface],
) -> Result<Option<OffsetEdgeGeometry>, Failure> {
    let Some(first_face) = adjacent_faces.first() else {
        return Ok(None);
    };
    let geometry = build_curve_on_first_face(edge, first_face)?;
    if geometry.curve.is_null() {
        return Ok(Some(geometry));
    }
    Ok(Some(validate_adjacent_faces(
        edge,
        adjacent_faces,
        geometry,
    )?))
}

fn build_curve_on_first_face(
    edge: &Edge,
    first_face: &OffsetFaceSurface,
) -> Result<OffsetEdgeGeometry, Failure> {
    let pcurve = BRepTool::curve_on_face(edge, &first_face.face)?;
    let maker = BRepBuilderAPIMakeEdge::from_pcurve_params(
        &pcurve.pcurve,
        &first_face.surface,
        pcurve.first,
        pcurve.last,
    )?;
    let offset_edge: Edge = maker.edge()?;
    build_curve_geometry(edge, &offset_edge)
}

fn build_curve_geometry(edge: &Edge, offset_edge: &Edge) -> Result<OffsetEdgeGeometry, Failure> {
    let input: Shape = edge.clone().into();
    let tolerance: f64 = BRepTool::max_tolerance(&input, ShapeEnum::Vertex)?;
    let (continuity, max_degree, max_segment): (Continuity, i32, i32) = BUILD_CURVE3D_DEFAULTS;
    build_curve3d(offset_edge, tolerance, continuity, max_degree, max_segment)?;
    let result = BRepTool::curve_with_location(offset_edge)?;
    let tolerance = BRepTool::edge_tolerance(offset_edge)?;
    Ok(OffsetEdgeGeometry {
        curve: result.curve,
        location: result.location,
        tolerance,
    })
}

fn validate_adjacent_faces(
    edge: &Edge,
    adjacent_faces: &[OffsetFaceSurface],
    mut geometry: OffsetEdgeGeometry,
) -> Result<OffsetEdgeGeometry, Failure> {
    let mut max_deviation: f64 = 0.0;
    for face_surface in adjacent_faces {
        let deviation: f64 = deviation_on_face(edge, face_surface, &geometry.curve)?;
        max_deviation = max_deviation.max(deviation);
    }
    geometry.tolerance = geometry.tolerance.max(max_deviation);
    Ok(geometry)
}

fn deviation_on_face(
    edge: &Edge,
    face_surface: &OffsetFaceSurface,
    curve: &Handle<dyn GeomCurve>,
) -> Result<f64, Failure> {
    let pcurve = BRepTool::curve_on_face(edge, &face_surface.face)?;
    validate_curve_pair(curve, &pcurve, &face_surface.surface)
}

fn validate_curve_pair(
    curve: &Handle<dyn GeomCurve>,
    pcurve: &rustycad_tkbrep::brep::EdgePCurve,
    surface: &Handle<dyn GeomSurface>,
) -> Result<f64, Failure> {
    let curve2d = Geom2dAdaptorCurve::with_range(&pcurve.pcurve, pcurve.first, pcurve.last)?;
    let curve2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(curve2d).cast();
    let surface = GeomAdaptorSurface::with_surface(surface)?;
    let surface: Handle<dyn Adaptor3dSurface> = Handle::new(surface).cast();
    let on_surface = Handle::new(Adaptor3dCurveOnSurface::with_curve(&curve2d, &surface)?);
    let reference = GeomAdaptorCurve::with_range(curve, pcurve.first, pcurve.last)?;
    let reference: Handle<dyn Adaptor3dCurve> = Handle::new(reference).cast();
    let mut validator = BRepLibValidateEdge::new(&reference, &on_surface, true);
    validator.process()?;
    Ok(if validator.is_done() {
        validator.get_max_distance()
    } else {
        0.0
    })
}
