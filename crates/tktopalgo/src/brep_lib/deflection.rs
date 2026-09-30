// Rust translation of BRepLib::UpdateDeflection and its EvalDeflection and Link helpers
// (BRepLib.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::UpdateDeflection`: the deflection of the face triangulations measured
//! against their surfaces.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point2, Point3, Transform3, Xy, Xyz};
use rustycad_tkmath::poly::{Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;
use std::collections::HashSet;

/// `EvalDeflection`: the square distance of a 3d point to the surface point of its UV.
struct EvalDeflection {
    surface: BRepAdaptorSurface,
}

impl EvalDeflection {
    fn eval(&self, point2d: Point2, point3d: Point3) -> Result<f64, Failure> {
        let on_surface: Point3 = self.surface.d0(point2d.x(), point2d.y())?;
        Ok((point3d.xyz() - on_surface.xyz()).square_modulus())
    }
}

/// `BRepLib::UpdateDeflection(S)`: for every face with a surface and a triangulation
/// with UV nodes, sets the deflection of the triangulation to the largest distance of
/// the centroids of its triangles and of the middles of its inner links to the surface,
/// skipping the triangles on the nodes of degenerated edges;
/// `brep_lib::update_deflection(&shape)?`.
pub fn update_deflection(shape: &Shape) -> Result<(), Failure> {
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while explorer.more() {
        let face = Face::unchecked(explorer.current().clone());
        explorer.next()?;
        if BRepTool::surface(&face)?.is_null() {
            continue;
        }
        let (triangulation, location) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
        let Some(mesh) = triangulation.get() else {
            continue;
        };
        if !mesh.has_uv_nodes() {
            continue;
        }
        let degenerated_nodes: HashSet<i32> = degenerated_nodes(&face, &triangulation, &location)?;
        let tool = EvalDeflection {
            surface: BRepAdaptorSurface::with_face(&face, true)?,
        };
        let sq_deflection =
            mesh_deflection(mesh, &tool, &degenerated_nodes, location.transformation())?;
        mesh.set_deflection(sq_deflection.sqrt());
    }
    Ok(())
}

/// "Collect all nodes of degenerative edges": their elements are skipped because of
/// the distortions of the passage from UV space to 3D.
fn degenerated_nodes(
    face: &Face,
    triangulation: &rustycad_tkernel::standard::Handle<Triangulation>,
    location: &Location,
) -> Result<HashSet<i32>, Failure> {
    let mut nodes: HashSet<i32> = HashSet::new();
    let mut explorer = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        let edge = Edge::unchecked(explorer.current().clone());
        explorer.next()?;
        if !BRepTool::degenerated(&edge)? {
            continue;
        }
        let polygon = BRepTool::polygon_on_triangulation(&edge, triangulation, location)?;
        let Some(polygon) = polygon.get() else {
            continue;
        };
        let array = polygon.nodes();
        for index in array.lower()..=array.upper() {
            nodes.insert(polygon.node(index)?);
        }
    }
    Ok(nodes)
}

/// The point of coordinates `xyz`.
fn point_of(xyz: Xyz) -> Point3 {
    Point3::new(xyz.x(), xyz.y(), xyz.z())
}

/// The largest square deviation of the triangle centroids and of the middles of the
/// links met twice (inner links; the boundary ones are distorted).
fn mesh_deflection(
    mesh: &Triangulation,
    tool: &EvalDeflection,
    degenerated_nodes: &HashSet<i32>,
    trsf: Transform3,
) -> Result<f64, Failure> {
    // `NCollection_Map<Link>`: a link is its unordered node pair.
    let mut links: HashSet<(i32, i32)> = HashSet::new();
    let mut sq_deflection: f64 = 0.;
    for tri_index in 1..=mesh.nb_triangles() {
        let (n0, n1, n2) = mesh.triangle(tri_index)?.get();
        let node = [n0, n1, n2];
        if node.iter().any(|n| degenerated_nodes.contains(n)) {
            continue;
        }
        let p3d: [Point3; 3] = [
            mesh.node(n0)?.transformed(trsf),
            mesh.node(n1)?.transformed(trsf),
            mesh.node(n2)?.transformed(trsf),
        ];
        let p2d: [Point2; 3] = [mesh.uv_node(n0)?, mesh.uv_node(n1)?, mesh.uv_node(n2)?];
        // Check midpoint of triangle.
        let mid3d = point_of((p3d[0].xyz() + p3d[1].xyz() + p3d[2].xyz()) / 3.);
        let mid2d = Point2::from((p2d[0].xy() + p2d[1].xy() + p2d[2].xy()) / 3.);
        sq_deflection = std_max(sq_deflection, tool.eval(mid2d, mid3d)?);
        for i in 0..3 {
            let j = (i + 1) % 3;
            if links.insert((node[i].min(node[j]), node[i].max(node[j]))) {
                continue;
            }
            // Do not estimate boundary links due to high distortions at the edge.
            let mid3d_l: Xyz = (p3d[i].xyz() + p3d[j].xyz()) / 2.;
            let mid2d_l: Xy = (p2d[i].xy() + p2d[j].xy()) / 2.;
            sq_deflection = std_max(
                sq_deflection,
                tool.eval(Point2::from(mid2d_l), point_of(mid3d_l))?,
            );
        }
    }
    Ok(sq_deflection)
}
