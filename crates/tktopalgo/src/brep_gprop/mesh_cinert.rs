// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_MeshCinert.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_MeshCinert`: the global properties of polylines.
use super::gauss_table::gauss_points_and_weights;
use super::line_moments::LineMoments;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{self, Point3, Transform3, Xyz};
use rustycad_tkmath::toploc::Location;
use std::ops::Deref;

/// `BRepGProp_MeshCinert`: the length, centre and inertia of a polyline (a
/// `GProp_GProps`, docs/topalgo-design.md section 9), used for the edges that have
/// only polygons; the inherited members are reached through `Deref`.
///
/// ```
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_gprop::BRepGPropMeshCinert;
/// let mut nodes = Array1::with_bounds(1, 2).unwrap();
/// nodes[1] = Point3::new(0.0, 0.0, 0.0);
/// nodes[2] = Point3::new(3.0, 4.0, 0.0);
/// let mut props = BRepGPropMeshCinert::new();
/// props.perform(&nodes).unwrap();
/// assert_eq!(props.mass(), 5.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BRepGPropMeshCinert {
    base: GProps,
}

impl Deref for BRepGPropMeshCinert {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

impl BRepGPropMeshCinert {
    /// `BRepGProp_MeshCinert()`: an empty framework at the origin;
    /// `BRepGPropMeshCinert::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetLocation(CLocation)`; `props.set_location(p)`.
    pub fn set_location(&mut self, location: Point3) {
        self.base.loc = location;
    }

    /// `Perform(theNodes)`: two-point Gauss integration on each segment of the polyline
    /// (read from index 1, as natively), skipping the segments shorter than
    /// gp::Resolution(); the centre is the last Gauss point when the length is below
    /// gp::Resolution(); Standard_OutOfRange for a polyline not indexed from 1;
    /// `props.perform(&nodes)?`.
    pub fn perform(&mut self, nodes: &Array1<'_, Point3>) -> Result<(), Failure> {
        let (points, weights) = gauss_points_and_weights(2)?;
        let mut totals = LineMoments::default();
        let mut last = Xyz::default();
        for index in 1..nodes.length() {
            let p1 = node(nodes, index)?.xyz();
            let p2 = node(nodes, index + 1)?.xyz();
            let upper = (p2 - p1).modulus();
            if upper < gp::resolution() {
                continue;
            }
            let um = 0.5 * upper;
            let direction = (p2 - p1) / upper;
            let mut local = LineMoments::default();
            for (point, weight) in points.iter().zip(&weights) {
                last = p1 + direction * (um + um * point);
                local.add_point(last - self.base.loc.xyz(), *weight);
            }
            local.scale(um);
            totals.accumulate(&local);
        }
        totals.store(&mut self.base, Point3::from(last));
        Ok(())
    }

    /// `PreparePolygon(theE)`: the 3d points of the first polygon the edge has, in the
    /// order 3D polygon, polygon on triangulation, 2d polygon on surface, moved by their
    /// location; `None` when the edge has none; Standard_OutOfRange for a polygon node
    /// outside its triangulation; `BRepGPropMeshCinert::prepare_polygon(&edge)?`.
    pub fn prepare_polygon(edge: &Edge) -> Result<Option<Array1<'static, Point3>>, Failure> {
        let (polygon, location) = BRepTool::polygon3d(edge)?;
        if let Some(polygon) = polygon.get() {
            return located_points(&polygon.nodes(), &location).map(Some);
        }
        let (on_mesh, mesh, location) = BRepTool::first_polygon_on_triangulation(edge)?;
        if let (Some(on_mesh), Some(mesh)) = (on_mesh.get(), mesh.get()) {
            let mut points = new_points(on_mesh.nb_nodes())?;
            for i in 1..=on_mesh.nb_nodes() {
                points[i] = mesh.node(on_mesh.node(i)?)?;
            }
            return located_points(&points, &location).map(Some);
        }
        let (polygon, surface, location) = BRepTool::first_polygon_on_surface(edge)?;
        let Some(polygon) = polygon.get() else {
            return Ok(None);
        };
        let nodes = polygon.nodes();
        let mut points = new_points(polygon.nb_nodes())?;
        for i in 1..=polygon.nb_nodes() {
            let uv = nodes[i];
            points[i] = surface_of(&surface)?.value(uv.x(), uv.y())?;
        }
        located_points(&points, &location).map(Some)
    }

    /// `PreparePolygon(theE, thePolyg)` (deprecated): [`BRepGPropMeshCinert::prepare_polygon`]
    /// into `polygon`; `BRepGPropMeshCinert::prepare_polygon_into(&edge, &mut polygon)?`.
    pub fn prepare_polygon_into(
        edge: &Edge,
        polygon: &mut Option<Array1<'static, Point3>>,
    ) -> Result<(), Failure> {
        *polygon = Self::prepare_polygon(edge)?;
        Ok(())
    }
}

/// `theNodes(index)`: Standard_OutOfRange outside the array.
fn node(nodes: &Array1<'_, Point3>, index: i32) -> Result<Point3, Failure> {
    nodes
        .value(index)
        .copied()
        .map_err(|error| error.into_failure("BRepGProp_MeshCinert::Perform"))
}

/// `new NCollection_HArray1<gp_Pnt>(1, n)`.
fn new_points(count: i32) -> Result<Array1<'static, Point3>, Failure> {
    Array1::with_bounds(1, count)
        .map_err(|error| error.into_failure("BRepGProp_MeshCinert::PreparePolygon"))
}

/// The points moved by the location (copied when it is the identity), indexed from 1.
fn located_points(
    points: &Array1<'_, Point3>,
    location: &Location,
) -> Result<Array1<'static, Point3>, Failure> {
    let mut located = new_points(points.length())?;
    let transform: Option<Transform3> =
        (!location.is_identity()).then(|| location.transformation());
    for (i, point) in (1..=points.length()).zip(points.iter()) {
        located[i] = match transform {
            Some(transform) => point.transformed(transform),
            None => *point,
        };
    }
    Ok(located)
}

/// The surface of a polygon on surface (never null when the polygon is not).
fn surface_of(surface: &Handle<dyn GeomSurface>) -> Result<&dyn GeomSurface, Failure> {
    surface.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepGProp_MeshCinert::PreparePolygon: the polygon on surface has no surface",
            "",
        )
    })
}
