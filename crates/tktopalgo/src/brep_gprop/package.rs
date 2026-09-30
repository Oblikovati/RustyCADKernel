// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp.hxx/.cxx (the
// linear and surface properties and roughBaryCenter) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp`: the global properties of shapes, brought together in a
//! `GProp_GProps`. The volume members are in `volume.rs` and `volume_gk.rs`.
use super::cinert::BRepGPropCinert;
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use super::mesh_cinert::BRepGPropMeshCinert;
use super::mesh_props::{BRepGPropMeshProps, MeshObjType};
use super::sinert::BRepGPropSinert;
use crate::brep_check::explore::sub_shapes;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkmath::poly::{Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;

/// `BRepGProp`: the linear, surface and volume properties of a shape, added to a
/// framework (unit struct with associated functions; docs/topalgo-design.md
/// section 9).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkgeombase::gprop::GProps;
/// use rustycad_tktopalgo::brep_gprop::BRepGProp;
/// let mut props = GProps::new();
/// BRepGProp::linear_properties(&Shape::default(), &mut props, false, false).unwrap();
/// assert_eq!(props.mass(), 0.0);
/// ```
pub struct BRepGProp;

/// The origin moved by the location of the shape (`gp_Pnt(0, 0, 0).Transform(S.Location())`).
pub(super) fn located_origin(shape: &Shape) -> Point3 {
    Point3::new(0.0, 0.0, 0.0).transformed(shape.location().transformation())
}

/// `roughBaryCenter(S)`: the mean of the vertex points of the shape, else the first
/// node of the first face triangulation that has nodes (moved by the face location),
/// else the origin.
pub(super) fn rough_bary_center(shape: &Shape) -> Result<Point3, Failure> {
    let vertices = sub_shapes(shape, ShapeEnum::Vertex)?;
    let mut xyz = Xyz::new(0.0, 0.0, 0.0);
    for vertex in &vertices {
        xyz += BRepTool::pnt(&topods::vertex(vertex)?)?.xyz();
    }
    if !vertices.is_empty() {
        xyz /= vertices.len() as f64;
        return Ok(Point3::from(xyz));
    }
    for face in sub_shapes(shape, ShapeEnum::Face)? {
        let (mesh, location) = BRepTool::triangulation(&topods::face(&face)?, mesh_purpose::NONE)?;
        let Some(mesh) = mesh.get().filter(|mesh| mesh.nb_nodes() > 0) else {
            continue;
        };
        xyz = mesh.node(1)?.xyz();
        if !location.is_identity() {
            xyz = location.transformation().apply_coordinates(xyz);
        }
        break;
    }
    Ok(Point3::from(xyz))
}

/// Whether a face has no surface and whether it has no usable triangulation, with
/// that triangulation and the face location.
pub(super) struct FaceSources {
    pub(super) no_surface: bool,
    pub(super) no_mesh: bool,
    pub(super) mesh: Handle<Triangulation>,
    pub(super) location: Location,
}

impl FaceSources {
    pub(super) fn of(face: &Face) -> Result<Self, Failure> {
        let (surface, _) = BRepTool::surface_with_location(face)?;
        let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
        let no_mesh = mesh
            .get()
            .is_none_or(|m| m.nb_nodes() == 0 || m.nb_triangles() == 0);
        Ok(Self {
            no_surface: surface.is_null(),
            no_mesh,
            mesh,
            location,
        })
    }

    /// Whether the mesh is used: asked for and present, or the only source.
    pub(super) fn use_mesh(&self, use_triangulation: bool) -> bool {
        (use_triangulation && !self.no_mesh) || (self.no_surface && !self.no_mesh)
    }
}

impl BRepGProp {
    /// `LinearProperties(S, LProps, SkipShared, UseTriangulation)`: `props` becomes a
    /// framework at the located origin of `shape` holding the lengths of its edges
    /// (each shared edge once with `skip_shared`); an edge uses its polygons when
    /// `use_triangulation` asks or it has no curve, else its curve; a vertex adds
    /// nothing; `BRepGProp::linear_properties(&shape, &mut props, false, false)?`.
    pub fn linear_properties(
        shape: &Shape,
        props: &mut GProps,
        skip_shared: bool,
        use_triangulation: bool,
    ) -> Result<(), Failure> {
        let origin = located_origin(shape);
        *props = GProps::with_location(origin);
        let curve = BRepAdaptorCurve::default();
        let mut edges = MapOfShape::default();
        for shape in sub_shapes(shape, ShapeEnum::Edge)? {
            let edge = topods::edge(&shape)?;
            if skip_shared && !edges.add(shape) {
                continue;
            }
            let is_geometric = BRepTool::is_geometric_edge(&edge)?;
            let nodes = match use_triangulation || !is_geometric {
                true => BRepGPropMeshCinert::prepare_polygon(&edge)?,
                false => None,
            };
            if let Some(nodes) = nodes {
                let mut polyline = BRepGPropMeshCinert::new();
                polyline.set_location(origin);
                polyline.perform(&nodes)?;
                props.add(&polyline, 1.0)?;
            } else if is_geometric {
                curve.initialize(&edge)?;
                let cinert = BRepGPropCinert::with_curve(&curve, origin)?;
                props.add(&cinert, 1.0)?;
            }
        }
        Ok(())
    }

    /// `SurfaceProperties(S, SProps, SkipShared, UseTriangulation)`: `props` becomes a
    /// framework at the located origin holding the areas of the faces (fixed Gauss
    /// orders; meshes when asked or when a face has no surface);
    /// `BRepGProp::surface_properties(&shape, &mut props, false, false)?`.
    pub fn surface_properties(
        shape: &Shape,
        props: &mut GProps,
        skip_shared: bool,
        use_triangulation: bool,
    ) -> Result<(), Failure> {
        *props = GProps::with_location(located_origin(shape));
        surface_props(shape, props, 1.0, (skip_shared, use_triangulation))?;
        Ok(())
    }

    /// `SurfaceProperties(S, SProps, Eps, SkipShared)`: the adaptive integration of
    /// each face with the relative error `eps` (non-adaptive above 0.001); the largest
    /// error reached; `BRepGProp::surface_properties_eps(&shape, &mut props, 1e-6, false)?`.
    pub fn surface_properties_eps(
        shape: &Shape,
        props: &mut GProps,
        eps: f64,
        skip_shared: bool,
    ) -> Result<f64, Failure> {
        *props = GProps::with_location(located_origin(shape));
        surface_props(shape, props, eps, (skip_shared, false))
    }
}

/// `surfaceProperties`: the faces of `shape` about its rough barycentre, one Sinert
/// and one mesh framework reused for every face (so a face whose integration returns
/// early adds the previous face's values, as natively); the largest error.
fn surface_props(
    shape: &Shape,
    props: &mut GProps,
    eps: f64,
    (skip_shared, use_triangulation): (bool, bool),
) -> Result<f64, Failure> {
    let centre = rough_bary_center(shape)?;
    let mut surface = BRepGPropSinert::new();
    surface.set_location(centre);
    let mut mesh = BRepGPropMeshProps::new(MeshObjType::Sinert);
    mesh.set_location(centre);
    let (mut tool, mut domain) = (BRepGPropFace::new(false), BRepGPropDomain::new());
    let mut faces = MapOfShape::default();
    let mut error_max = 0.0;
    for shape in sub_shapes(shape, ShapeEnum::Face)? {
        let face = topods::face(&shape)?;
        if skip_shared && !faces.add(shape) {
            continue;
        }
        let sources = FaceSources::of(&face)?;
        if sources.no_mesh && sources.no_surface {
            continue;
        }
        if sources.use_mesh(use_triangulation) {
            mesh.perform_located(&sources.mesh, &sources.location, face.orientation())?;
            props.add(&mesh, 1.0)?;
            continue;
        }
        let error = surface_face(&face, (&mut tool, &mut domain), &mut surface, eps)?;
        if error_max < error {
            error_max = error;
        }
        props.add(&surface, 1.0)?;
    }
    Ok(error_max)
}

/// The Sinert integration of one face: adaptive below `eps` 1 (its error), else on the
/// natural bounds or along the domain (error 0).
fn surface_face(
    face: &Face,
    (tool, domain): (&mut BRepGPropFace, &mut BRepGPropDomain),
    surface: &mut BRepGPropSinert,
    eps: f64,
) -> Result<f64, Failure> {
    tool.load_face(face)?;
    let is_natural = face.nb_children() == 0;
    if !is_natural {
        domain.init_face(face)?;
    }
    if eps < 1.0 {
        surface.perform_domain_eps(tool, domain, eps)?;
        return Ok(surface.get_epsilon());
    }
    if is_natural {
        surface.perform(tool)?;
    } else {
        surface.perform_domain(tool, domain)?;
    }
    Ok(0.0)
}
