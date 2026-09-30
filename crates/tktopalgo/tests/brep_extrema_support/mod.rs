//! Meshes for the BRepExtrema behaviour tests: a stand-in for BRepMesh_IncrementalMesh
//! (TKMesh is not ported): grid triangulations of faces and tangential-deflection
//! polygons of free edges, with cell counts derived from the linear deflection.
#![allow(dead_code)]

use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gcpnts::{TangentialCriteria, TangentialDeflection};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Polygon3D, Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;

/// The density of the stand-in: its angular deflection and its linear deflection
/// relative to the asked one.
#[derive(Clone, Copy)]
pub struct Density {
    pub angular: f64,
    pub ratio: f64,
}

/// The angular deflection and linear deflection of BRepMesh (0.5 rad, the asked one).
pub const BREPMESH_LIKE: Density = Density {
    angular: 0.5,
    ratio: 1.0,
};

/// A finer density: the grid and the tangential sampling put fewer nodes near the
/// extreme points than BRepMesh does, so most cases need ten times the density.
pub const FINE: Density = Density {
    angular: 0.1,
    ratio: 0.1,
};

/// A density between the two, for the spheres (the fine grid of a whole sphere is slow).
pub const MEDIUM: Density = Density {
    angular: 0.25,
    ratio: 0.5,
};

/// The number of steps of an arc of `span` radians and radius `radius` whose sagitta
/// stays within `deflection` and whose steps stay within `angular` (at least one).
pub fn arc_steps(span: f64, radius: f64, deflection: f64, angular: f64) -> i32 {
    let step: f64 = 2.0 * (1.0 - deflection / radius).clamp(-1.0, 1.0).acos();
    let by_angle: f64 = (span.abs() / angular).ceil();
    ((span.abs() / step).ceil()).max(by_angle).max(1.0) as i32
}

/// The cells of a face in U and V: one per direction for a plane, arcs of the radius
/// for a cylinder (around U) and a sphere (both directions).
fn face_cells(
    surface: &BRepAdaptorSurface,
    bounds: (f64, f64, f64, f64),
    (deflection, angular): (f64, f64),
) -> Result<(i32, i32), Failure> {
    let (u1, u2, v1, v2) = bounds;
    Ok(match surface.get_type()? {
        SurfaceType::Cylinder => (
            arc_steps(u2 - u1, surface.cylinder()?.radius(), deflection, angular),
            1,
        ),
        SurfaceType::Sphere => {
            let radius: f64 = surface.sphere()?.radius();
            (
                arc_steps(u2 - u1, radius, deflection, angular),
                arc_steps(v2 - v1, radius, deflection, angular),
            )
        }
        _ => (1, 1),
    })
}

/// The grid triangles of `nu` x `nv` cells (nodes numbered row by row from 1).
fn grid_triangles(nu: i32, nv: i32) -> Result<Array1<'static, Triangle>, Failure> {
    let mut triangles: Array1<'static, Triangle> =
        Array1::with_bounds(1, 2 * nu * nv).map_err(|e| e.into_failure("mesh"))?;
    for j in 0..nv {
        for i in 0..nu {
            let a: i32 = j * (nu + 1) + i + 1;
            let cell: i32 = 2 * (j * nu + i) + 1;
            triangles[cell] = Triangle::new(a, a + 1, a + nu + 2);
            triangles[cell + 1] = Triangle::new(a, a + nu + 2, a + nu + 1);
        }
    }
    Ok(triangles)
}

/// A grid triangulation of `face` on its UV bounds, attached to the face.
pub fn mesh_face(face: &Face, deflection: f64, angular: f64) -> Result<(), Failure> {
    let surface = BRepAdaptorSurface::with_face(face, true)?;
    let bounds = brep_tools::uv_bounds(face)?;
    let (nu, nv) = face_cells(&surface, bounds, (deflection, angular))?;
    let (u1, u2, v1, v2) = bounds;
    let mut nodes: Array1<'static, Point3> =
        Array1::with_bounds(1, (nu + 1) * (nv + 1)).map_err(|e| e.into_failure("mesh"))?;
    for j in 0..=nv {
        for i in 0..=nu {
            let u: f64 = u1 + (u2 - u1) * f64::from(i) / f64::from(nu);
            let v: f64 = v1 + (v2 - v1) * f64::from(j) / f64::from(nv);
            nodes[j * (nu + 1) + i + 1] = surface.value(u, v)?;
        }
    }
    let triangulation = Handle::new(Triangulation::from_nodes(&nodes, &grid_triangles(nu, nv)?)?);
    BRepBuilder::default().update_face_triangulation(face, &triangulation, true)
}

/// A tangential-deflection polygon of `edge`, attached as its 3D polygon.
pub fn mesh_edge(edge: &Edge, deflection: f64, angular: f64) -> Result<(), Failure> {
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    let criteria = TangentialCriteria::new(angular, deflection);
    let sampling = TangentialDeflection::with_curve(&curve as &dyn Adaptor3dCurve, criteria)?;
    let count: i32 = sampling.nb_points();
    let mut nodes: Array1<'static, Point3> =
        Array1::with_bounds(1, count).map_err(|e| e.into_failure("mesh"))?;
    for i in 1..=count {
        nodes[i] = sampling.value(i)?;
    }
    let polygon = Handle::new(Polygon3D::from_nodes(&nodes));
    BRepBuilder::default().update_edge_polygon3d(edge, &polygon, &Location::default())
}

/// The mesh of a shape with a density: its faces, or its non-degenerated edges when it
/// has no face (BRepMesh gives free edges a 3D polygon, face edges polygons on the
/// triangulations, which the triangle sets do not read).
pub fn mesh_shape(shape: &Shape, deflection: f64, density: Density) -> Result<(), Failure> {
    let deflection: f64 = deflection * density.ratio;
    let mut has_faces: bool = false;
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        mesh_face(&topods::face(&face)?, deflection, density.angular)?;
        has_faces = true;
    }
    if has_faces {
        return Ok(());
    }
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&edge)?;
        if !BRepTool::degenerated(&edge)? {
            mesh_edge(&edge, deflection, density.angular)?;
        }
    }
    Ok(())
}
