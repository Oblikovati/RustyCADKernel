//! The BRepLib_PointCloudShape probe operation, the Rust twin of
//! tools/oracle/tktopalgo_probe_breplib_pointcloud.cxx (the request layout is documented
//! there).
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, xyz};
use super::shape::tp_shape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tktopalgo::brep_lib::{BRepLibPointCloudShape, BRepLibPointCloudShapeBase};

/// `TpPointCloud`: prints each point as it comes with the index of its face.
struct PointCloud {
    base: BRepLibPointCloudShapeBase,
    faces: IndexedMapOfShape,
}

impl PointCloud {
    fn new(shape: &Shape, tol: f64) -> Result<Self, Failure> {
        let mut faces = IndexedMapOfShape::default();
        if !shape.is_null() {
            topexp::map_shapes_of_type(shape, ShapeEnum::Face, &mut faces)?;
        }
        Ok(Self {
            base: BRepLibPointCloudShapeBase::new(shape.clone(), tol),
            faces,
        })
    }
}

impl BRepLibPointCloudShape for PointCloud {
    fn point_cloud_base(&self) -> &BRepLibPointCloudShapeBase {
        &self.base
    }
    fn point_cloud_base_mut(&mut self) -> &mut BRepLibPointCloudShapeBase {
        &mut self.base
    }
    fn add_point(&mut self, point: Point3, normal: Vector3, uv: Point2, face: &Shape) {
        pnt(point);
        xyz(*normal.xyz());
        pnt2d(uv);
        int(self.faces.find_index(face) as i32);
    }
}

/// `tp_cloud_uv_nodes`: UV nodes (ElSLib parameters) on the meshes of the plane,
/// cylinder and sphere faces.
fn cloud_uv_nodes(shape: &Shape) -> Result<(), Failure> {
    if shape.is_null() {
        return Ok(());
    }
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = topods::face(&face)?;
        let (mesh, _) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
        let surface = BRepTool::surface(&face)?;
        let Some(mesh) = mesh.get() else { continue };
        if surface.is_null() {
            continue;
        }
        let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
        let kind = adaptor.get_type()?;
        if !matches!(
            kind,
            SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::Sphere
        ) {
            continue;
        }
        mesh.add_uv_nodes()?;
        for i in 1..=mesh.nb_nodes() {
            let p = mesh.node(i)?;
            let (u, v) = match kind {
                SurfaceType::Plane => elslib::plane_parameters(adaptor.plane()?.position(), p),
                SurfaceType::Cylinder => {
                    let c = adaptor.cylinder()?;
                    elslib::cylinder_parameters(c.position(), c.radius(), p)
                }
                _ => {
                    let s = adaptor.sphere()?;
                    elslib::sphere_parameters(s.position(), s.radius(), p)
                }
            };
            mesh.set_uv_node(i, Point2::new(u, v))?;
        }
    }
    Ok(())
}

/// One operation of the request on the cloud.
fn run_op(cloud: &mut PointCloud, op: i32, arg: f64) -> Result<(), Failure> {
    match op {
        0 => int(cloud.nb_points_by_density(arg)?),
        1 => flag(cloud.generate_points_by_density(arg)?),
        2 => int(cloud.nb_points_by_triangulation()?),
        3 => flag(cloud.generate_points_by_triangulation()?),
        4 => cloud.set_distance(arg),
        _ => cloud.set_tolerance(arg),
    }
    Ok(())
}

/// `shape tol dist uv n (op arg)...`: the operations on one cloud, then its
/// tolerance and distance.
pub fn point_cloud_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let (tol, dist) = (r.real(), r.real());
        if r.flag() {
            cloud_uv_nodes(&s.shape)?;
        }
        let mut cloud = PointCloud::new(&s.shape, tol)?;
        cloud.set_distance(dist);
        for _ in 0..r.integer() {
            let op = r.integer();
            run_op(&mut cloud, op, r.real())?;
        }
        real(cloud.tolerance());
        real(cloud.get_distance());
        Ok(())
    });
    end();
}
