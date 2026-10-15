//! The quadric parts of `AIS_Manipulator` (disk, sphere, sector) and its scaling cube.
use super::AisManipulatorSkin;
use crate::prs3d::{Prs3dToolDisk, Prs3dToolQuadric, Prs3dToolSector, Prs3dToolSphere};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{
    self, Axis1, Axis2, Axis3, Direction3, Point3, StandardDirection3, Transform3, Vector3,
};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tkservice::graphic3d::*;

/// `Graphic3d_ArrayFlags_VertexNormal`.
const VERTEX_NORMAL: Graphic3dArrayFlags = GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL;

/// `gp_Trsf::SetTransformation(theSystem, gp_Ax3())`: from a local system to the world.
pub(super) fn local_system(system: Axis3) -> Transform3 {
    let mut trsf = Transform3::default();
    trsf.set_transformation_between(system, Axis3::default());
    trsf
}

/// `AIS_Manipulator::Quadric`: a triangulation and the triangle array of a quadric part.
#[derive(Clone, Default)]
pub struct AisManipulatorQuadric {
    /// `myTriangulation`.
    pub triangulation: Handle<Triangulation>,
    /// `myArray`.
    pub array: Handle<Graphic3dArrayOfTriangles>,
}
impl AisManipulatorQuadric {
    /// `Triangulation`; `quadric.triangulation()`.
    pub fn triangulation(&self) -> &Handle<Triangulation> {
        &self.triangulation
    }
    /// `Array`; `quadric.array()`.
    pub fn array(&self) -> &Handle<Graphic3dArrayOfTriangles> {
        &self.array
    }
}

/// `AIS_Manipulator::Disk`: a ring (sector of a ring) around an axis.
#[derive(Clone)]
pub struct AisManipulatorDisk {
    /// The quadric geometry.
    pub quadric: AisManipulatorQuadric,
    /// `myPosition`.
    pub position: Axis1,
    /// `myInnerRad`.
    pub inner_rad: f32,
    /// `myOuterRad`.
    pub outer_rad: f32,
}
impl Default for AisManipulatorDisk {
    fn default() -> Self {
        Self {
            quadric: AisManipulatorQuadric::default(),
            position: Axis1::default(),
            inner_rad: 0.0,
            outer_rad: 1.0,
        }
    }
}
impl AisManipulatorDisk {
    /// `Init`: the ring between the radii from angle 0 to `angle`, in the plane normal to
    /// the axis; `disk.init(1.0, 2.0, &axis, PI, 20, 20)?`.
    pub fn init(
        &mut self,
        inner_radius: f32,
        outer_radius: f32,
        position: &Axis1,
        angle: f64,
        slices_nb: i32,
        stacks_nb: i32,
    ) -> Result<(), Failure> {
        self.position = *position;
        self.inner_rad = inner_radius;
        self.outer_rad = outer_radius;
        let mut tool = Prs3dToolDisk::new(
            f64::from(inner_radius),
            f64::from(outer_radius),
            slices_nb,
            stacks_nb,
        );
        tool.set_angle_range(0.0, angle);
        let trsf = local_system(Axis3::from_normal(
            position.location(),
            position.direction(),
        )?);
        self.quadric.array = tool.create_triangulation(&trsf)?;
        self.quadric.triangulation = tool.create_poly_triangulation(&trsf)?;
        Ok(())
    }
    /// `Array`; `disk.array()`.
    pub fn array(&self) -> &Handle<Graphic3dArrayOfTriangles> {
        &self.quadric.array
    }
    /// `Triangulation`; `disk.triangulation()`.
    pub fn triangulation(&self) -> &Handle<Triangulation> {
        &self.quadric.triangulation
    }
}

/// `AIS_Manipulator::Sphere`: the center sphere.
#[derive(Clone)]
pub struct AisManipulatorSphere {
    /// The quadric geometry.
    pub quadric: AisManipulatorQuadric,
    /// `myPosition`.
    pub position: Point3,
    /// `myRadius`.
    pub radius: f32,
}
impl Default for AisManipulatorSphere {
    fn default() -> Self {
        Self {
            quadric: AisManipulatorQuadric::default(),
            position: Point3::default(),
            radius: 1.0,
        }
    }
}
impl AisManipulatorSphere {
    /// `Init`: a sphere at a point (half the radius in the flat skin).
    /// Example: `sphere.init(1.0, &point, AisManipulatorSkin::Shaded, 20, 20)?`.
    pub fn init(
        &mut self,
        radius: f32,
        position: &Point3,
        skin: AisManipulatorSkin,
        slices_nb: i32,
        stacks_nb: i32,
    ) -> Result<(), Failure> {
        self.position = *position;
        self.radius = radius;
        let mut trsf = Transform3::default();
        trsf.set_translation(Vector3::between(gp::origin(), *position));
        let radius = if skin == AisManipulatorSkin::Flat {
            f64::from(radius) * 0.5
        } else {
            f64::from(radius)
        };
        let tool = Prs3dToolSphere::new(radius, slices_nb, stacks_nb);
        self.quadric.array = tool.create_triangulation(&trsf)?;
        self.quadric.triangulation = tool.create_poly_triangulation(&trsf)?;
        Ok(())
    }
    /// `Array`; `sphere.array()`.
    pub fn array(&self) -> &Handle<Graphic3dArrayOfTriangles> {
        &self.quadric.array
    }
}

/// `AIS_Manipulator::Cube`: the scaling handle (a box, or a disk in the flat skin).
#[derive(Clone, Default)]
pub struct AisManipulatorCube {
    /// `myTriangulation`.
    pub triangulation: Handle<Triangulation>,
    /// `myArray`.
    pub array: Handle<Graphic3dArrayOfTriangles>,
}
impl AisManipulatorCube {
    /// `Init`: a box of a size on the axis (a disk at the size along the axis in the flat
    /// skin); `cube.init(&axis, 2.0, AisManipulatorSkin::Shaded)?`.
    pub fn init(
        &mut self,
        position: &Axis1,
        size: f32,
        skin: AisManipulatorSkin,
    ) -> Result<(), Failure> {
        let size = f64::from(size);
        let direction = position.direction();
        if skin == AisManipulatorSkin::Flat {
            let x_direction = flat_x_direction(&direction);
            let location = position
                .location()
                .translated(Vector3::from(direction.xyz() * size));
            let trsf = local_system(Axis3::new(location, x_direction, direction)?);
            let tool = Prs3dToolDisk::new(0.0, size, 40, 40);
            self.array = tool.create_triangulation(&trsf)?;
            self.triangulation = tool.create_poly_triangulation(&trsf)?;
            return Ok(());
        }
        self.array = Handle::new(Graphic3dArrayOfTriangles::with_options(
            12 * 3,
            0,
            true,
            false,
            false,
        )?);
        let points = vec![Point3::default(); 36];
        let triangles = vec![Triangle::default(); 12];
        self.triangulation = Handle::new(Triangulation::from_nodes(
            &Array1::from_shared(&points, 1),
            &Array1::from_shared(&triangles, 1),
        )?);
        let plane = Axis2::new(position.location(), direction)?;
        let (px, py) = (plane.x_direction().xyz(), plane.y_direction().xyz());
        let location = position.location().xyz();
        let bottom_left = location - px * size * 0.5 - py * size * 0.5;
        let v2 = bottom_left + py * size;
        let v3 = bottom_left + py * size + px * size;
        let v4 = bottom_left + px * size;
        let top_right = location + direction.xyz() * size + px * size * 0.5 + py * size * 0.5;
        let v5 = top_right - py * size;
        let v6 = top_right - py * size - px * size;
        let v7 = top_right - px * size;
        let point = Point3::from;
        let right = Direction3::from_xyz((v7 - top_right).crossed(v2 - top_right))?;
        let front = Direction3::from_xyz((v4 - v3).crossed(v5 - v3))?;
        let (bl, tr) = (point(bottom_left), point(top_right));
        let (v2, v3, v4, v5, v6, v7) = (
            point(v2),
            point(v3),
            point(v4),
            point(v5),
            point(v6),
            point(v7),
        );
        let down = direction.reversed();
        self.add_triangle(0, bl, v2, v3, down)?;
        self.add_triangle(1, bl, v3, v4, down)?;
        self.add_triangle(2, v3, v5, v4, front.reversed())?;
        self.add_triangle(3, v3, tr, v5, front.reversed())?;
        self.add_triangle(4, bl, v7, v2, front)?;
        self.add_triangle(5, bl, v6, v7, front)?;
        self.add_triangle(6, v7, v6, v5, direction)?;
        self.add_triangle(7, tr, v7, v5, direction)?;
        self.add_triangle(8, v6, v4, v5, right)?;
        self.add_triangle(9, bl, v4, v6, right)?;
        self.add_triangle(10, v3, v7, tr, right.reversed())?;
        self.add_triangle(11, v3, v2, v7, right.reversed())
    }
    /// `addTriangle` (private): three nodes and a triangle of the triangulation, three
    /// vertices with the normal in the array; `cube.add_triangle(0, p1, p2, p3, normal)?`.
    pub fn add_triangle(
        &mut self,
        index: i32,
        p1: Point3,
        p2: Point3,
        p3: Point3,
        normal: Direction3,
    ) -> Result<(), Failure> {
        let triangulation = self.triangulation.get().ok_or_else(null_part)?;
        triangulation.set_node(index * 3 + 1, p1)?;
        triangulation.set_node(index * 3 + 2, p2)?;
        triangulation.set_node(index * 3 + 3, p3)?;
        triangulation.set_triangle(
            index + 1,
            Triangle::new(index * 3 + 1, index * 3 + 2, index * 3 + 3),
        )?;
        let array = self.array.get().ok_or_else(null_part)?;
        for point in [p1, p2, p3] {
            array.add_vertex_pnt_normal(&point, &normal)?;
        }
        Ok(())
    }
    /// `Triangulation`; `cube.triangulation()`.
    pub fn triangulation(&self) -> &Handle<Triangulation> {
        &self.triangulation
    }
    /// `Array`; `cube.array()`.
    pub fn array(&self) -> &Handle<Graphic3dArrayOfTriangles> {
        &self.array
    }
}

/// `AIS_Manipulator::Sector`: the plane-dragging handle (a quarter disk, or a small square
/// in the flat skin).
#[derive(Clone, Default)]
pub struct AisManipulatorSector {
    /// The quadric geometry.
    pub quadric: AisManipulatorQuadric,
    /// `myPosition`.
    pub position: Axis1,
    /// `myRadius`.
    pub radius: f32,
}
impl AisManipulatorSector {
    /// `Init`: the sector of a radius in the plane normal to the axis, from the X direction;
    /// `sector.init(5.0, &axis, x, AisManipulatorSkin::Shaded, 40, 5)?`.
    pub fn init(
        &mut self,
        radius: f32,
        position: &Axis1,
        x_direction: &Direction3,
        skin: AisManipulatorSkin,
        slices_nb: i32,
        stacks_nb: i32,
    ) -> Result<(), Failure> {
        let trsf = local_system(Axis3::new(
            position.location(),
            position.direction(),
            *x_direction,
        )?);
        if skin == AisManipulatorSkin::Flat {
            let array = Handle::new(Graphic3dArrayOfTriangles::new(4, 6, VERTEX_NORMAL)?);
            let triangulation = Handle::new(Triangulation::with_sizes(4, 2, false, false)?);
            let indent = f64::from(radius) / 3.0;
            let corner = |x: f64, y: f64| Point3::new(x, y, 0.0).transformed(trsf);
            let v1 = corner(indent, indent);
            let v2 = corner(indent, indent * 2.0);
            let v3 = corner(indent * 2.0, indent * 2.0);
            let v4 = corner(indent * 2.0, indent);
            let normal = Direction3::from(StandardDirection3::NegZ).transformed(trsf);
            for v in [v1, v2, v3, v4] {
                array.add_vertex_pnt_normal(&v, &normal)?;
            }
            array.add_triangle_edges(3, 1, 2)?;
            array.add_triangle_edges(1, 3, 4)?;
            for (index, v) in [v1, v2, v3, v4].into_iter().enumerate() {
                triangulation.set_node(index as i32 + 1, v)?;
            }
            triangulation.set_triangle(1, Triangle::new(3, 1, 2))?;
            triangulation.set_triangle(2, Triangle::new(1, 3, 4))?;
            self.quadric.array = array;
            self.quadric.triangulation = triangulation;
        } else {
            let tool = Prs3dToolSector::new(f64::from(radius), slices_nb, stacks_nb);
            self.quadric.array = tool.create_triangulation(&trsf)?;
            self.quadric.triangulation = tool.create_poly_triangulation(&trsf)?;
        }
        Ok(())
    }
    /// `Array`; `sector.array()`.
    pub fn array(&self) -> &Handle<Graphic3dArrayOfTriangles> {
        &self.quadric.array
    }
}

pub(super) fn null_part() -> Failure {
    Failure::with_message("Standard_NullObject", "AIS_Manipulator: null part geometry")
}

/// The X direction of a flat part on an axis: Y for the X axis, Z for the Y axis, else X.
pub(super) fn flat_x_direction(direction: &Direction3) -> Direction3 {
    if direction.x() > 0.0 {
        gp::dy()
    } else if direction.y() > 0.0 {
        gp::dz()
    } else {
        gp::dx()
    }
}
