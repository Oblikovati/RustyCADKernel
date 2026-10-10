//! The concrete quadric tools: `Prs3d_ToolCylinder`, `Prs3d_ToolDisk`, `Prs3d_ToolSphere`,
//! `Prs3d_ToolTorus` and `Prs3d_ToolSector`.
use super::Prs3dToolQuadric;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point3, StandardDirection3, Transform3};
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
use std::f64::consts::PI;

/// `Create`: a new array filled by a tool.
fn create_with<T: Prs3dToolQuadric>(
    tool: &T,
    trsf: &Transform3,
) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
    let mut array = Handle::default();
    tool.fill_array(&mut array, trsf)?;
    Ok(array)
}

/// A cylinder or cone along Z from the bottom radius at 0 to the top radius at the height.
///
/// Example: `Prs3dToolCylinder::new(1.0, 0.5, 2.0, 12, 1).create_triangulation(&trsf)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prs3dToolCylinder {
    bottom_radius: f64,
    top_radius: f64,
    height: f64,
    slices_nb: i32,
    stacks_nb: i32,
}

impl Prs3dToolCylinder {
    /// A cylinder tool; `Prs3dToolCylinder::new(1.0, 1.0, 2.0, 12, 1)`.
    pub fn new(bottom_rad: f64, top_rad: f64, height: f64, nb_slices: i32, nb_stacks: i32) -> Self {
        Self {
            bottom_radius: bottom_rad,
            top_radius: top_rad,
            height,
            slices_nb: nb_slices,
            stacks_nb: nb_stacks,
        }
    }
    /// `Create`: the triangles of a cylinder; `Prs3dToolCylinder::create(1.0, 1.0, 2.0, 12, 1, &trsf)?`.
    pub fn create(
        bottom_rad: f64,
        top_rad: f64,
        height: f64,
        nb_slices: i32,
        nb_stacks: i32,
        trsf: &Transform3,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        create_with(
            &Self::new(bottom_rad, top_rad, height, nb_slices, nb_stacks),
            trsf,
        )
    }
}

impl Prs3dToolQuadric for Prs3dToolCylinder {
    fn slices_nb(&self) -> i32 {
        self.slices_nb
    }
    fn stacks_nb(&self) -> i32 {
        self.stacks_nb
    }
    fn vertex(&self, u: f64, v: f64) -> Point3 {
        let angle = u * PI * 2.0;
        let radius = self.bottom_radius + (self.top_radius - self.bottom_radius) * v;
        Point3::new(angle.cos() * radius, angle.sin() * radius, v * self.height)
    }
    fn normal(&self, u: f64, _v: f64) -> Result<Direction3, Failure> {
        let angle = u * PI * 2.0;
        Direction3::new(
            angle.cos() * self.height,
            angle.sin() * self.height,
            self.bottom_radius - self.top_radius,
        )
    }
}

/// A flat ring (a disk for a zero inner radius) in the XY plane, facing -Z.
///
/// Example: `let mut disk = Prs3dToolDisk::new(0.0, 1.0, 12, 1); disk.set_angle_range(0.0, PI);`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prs3dToolDisk {
    inner_radius: f64,
    outer_radius: f64,
    start_angle: f64,
    end_angle: f64,
    slices_nb: i32,
    stacks_nb: i32,
}

impl Prs3dToolDisk {
    /// A full disk tool; `Prs3dToolDisk::new(0.5, 1.0, 12, 1)`.
    pub fn new(inner_radius: f64, outer_radius: f64, nb_slices: i32, nb_stacks: i32) -> Self {
        Self {
            inner_radius,
            outer_radius,
            start_angle: 0.0,
            end_angle: PI * 2.0,
            slices_nb: nb_slices,
            stacks_nb: nb_stacks,
        }
    }
    /// Sets the counterclockwise angle range; `disk.set_angle_range(0.0, PI)`.
    pub fn set_angle_range(&mut self, start_angle: f64, end_angle: f64) {
        (self.start_angle, self.end_angle) = (start_angle, end_angle);
    }
    /// `Create`: the triangles of a full disk; `Prs3dToolDisk::create(0.0, 1.0, 12, 1, &trsf)?`.
    pub fn create(
        inner_radius: f64,
        outer_radius: f64,
        nb_slices: i32,
        nb_stacks: i32,
        trsf: &Transform3,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        create_with(
            &Self::new(inner_radius, outer_radius, nb_slices, nb_stacks),
            trsf,
        )
    }
}

impl Prs3dToolQuadric for Prs3dToolDisk {
    fn slices_nb(&self) -> i32 {
        self.slices_nb
    }
    fn stacks_nb(&self) -> i32 {
        self.stacks_nb
    }
    fn vertex(&self, u: f64, v: f64) -> Point3 {
        let angle = self.start_angle + u * (self.end_angle - self.start_angle);
        let radius = self.inner_radius + (self.outer_radius - self.inner_radius) * v;
        Point3::new(angle.cos() * radius, angle.sin() * radius, 0.0)
    }
    fn normal(&self, _u: f64, _v: f64) -> Result<Direction3, Failure> {
        Ok(Direction3::from(StandardDirection3::NegZ))
    }
}

/// A sphere centered at the origin.
///
/// Example: `Prs3dToolSphere::create(1.0, 12, 6, &trsf)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prs3dToolSphere {
    radius: f64,
    slices_nb: i32,
    stacks_nb: i32,
}

impl Prs3dToolSphere {
    /// A sphere tool; `Prs3dToolSphere::new(1.0, 12, 6)`.
    pub fn new(radius: f64, nb_slices: i32, nb_stacks: i32) -> Self {
        Self {
            radius,
            slices_nb: nb_slices,
            stacks_nb: nb_stacks,
        }
    }
    /// `Create`: the triangles of a sphere; `Prs3dToolSphere::create(1.0, 12, 6, &trsf)?`.
    pub fn create(
        radius: f64,
        nb_slices: i32,
        nb_stacks: i32,
        trsf: &Transform3,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        create_with(&Self::new(radius, nb_slices, nb_stacks), trsf)
    }
}

impl Prs3dToolQuadric for Prs3dToolSphere {
    fn slices_nb(&self) -> i32 {
        self.slices_nb
    }
    fn stacks_nb(&self) -> i32 {
        self.stacks_nb
    }
    fn vertex(&self, u: f64, v: f64) -> Point3 {
        let (a_u, a_v) = (u * PI * 2.0, v * PI);
        Point3::new(
            self.radius * a_u.cos() * a_v.sin(),
            -self.radius * a_u.sin() * a_v.sin(),
            self.radius * a_v.cos(),
        )
    }
    fn normal(&self, u: f64, v: f64) -> Result<Direction3, Failure> {
        let (a_u, a_v) = (u * PI * 2.0, v * PI);
        Direction3::new(a_u.cos() * a_v.sin(), -a_u.sin() * a_v.sin(), a_v.cos())
    }
}

/// A torus (or a segment of it) around Z.
///
/// Example: `Prs3dToolTorus::new(2.0, 0.5, 0.0, PI * 2.0, PI * 2.0, 24, 12)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prs3dToolTorus {
    major_radius: f64,
    minor_radius: f64,
    angle: f64,
    v_min: f64,
    v_max: f64,
    slices_nb: i32,
    stacks_nb: i32,
}

impl Prs3dToolTorus {
    /// The general torus tool (`init`): a ring segment `[angle1, angle2]` of a pipe segment
    /// `angle`; `Prs3dToolTorus::new(2.0, 0.5, 0.0, PI, PI * 2.0, 24, 12)`.
    pub fn new(
        major_rad: f64,
        minor_rad: f64,
        angle1: f64,
        angle2: f64,
        angle: f64,
        nb_slices: i32,
        nb_stacks: i32,
    ) -> Self {
        Self {
            major_radius: major_rad,
            minor_radius: minor_rad,
            angle,
            v_min: angle1,
            v_max: angle2,
            slices_nb: nb_slices,
            stacks_nb: nb_stacks,
        }
    }
    /// A full torus; `Prs3dToolTorus::full(2.0, 0.5, 24, 12)`.
    pub fn full(major_rad: f64, minor_rad: f64, nb_slices: i32, nb_stacks: i32) -> Self {
        Self::new(
            major_rad,
            minor_rad,
            0.0,
            PI * 2.0,
            PI * 2.0,
            nb_slices,
            nb_stacks,
        )
    }
    /// A pipe segment of `angle`; `Prs3dToolTorus::with_angle(2.0, 0.5, PI, 24, 12)`.
    pub fn with_angle(
        major_rad: f64,
        minor_rad: f64,
        angle: f64,
        nb_slices: i32,
        nb_stacks: i32,
    ) -> Self {
        Self::new(
            major_rad,
            minor_rad,
            0.0,
            PI * 2.0,
            angle,
            nb_slices,
            nb_stacks,
        )
    }
    /// A ring segment `[angle1, angle2]` of a full pipe; `Prs3dToolTorus::with_ring(2.0, 0.5, 0.0, PI, 24, 12)`.
    pub fn with_ring(
        major_rad: f64,
        minor_rad: f64,
        angle1: f64,
        angle2: f64,
        nb_slices: i32,
        nb_stacks: i32,
    ) -> Self {
        Self::new(
            major_rad,
            minor_rad,
            angle1,
            angle2,
            PI * 2.0,
            nb_slices,
            nb_stacks,
        )
    }
    /// `Create` of the general torus; `Prs3dToolTorus::create(2.0, 0.5, 0.0, PI, PI, 24, 12, &trsf)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        major_rad: f64,
        minor_rad: f64,
        angle1: f64,
        angle2: f64,
        angle: f64,
        nb_slices: i32,
        nb_stacks: i32,
        trsf: &Transform3,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        create_with(
            &Self::new(
                major_rad, minor_rad, angle1, angle2, angle, nb_slices, nb_stacks,
            ),
            trsf,
        )
    }
    /// `Create` of a full torus; `Prs3dToolTorus::create_full(2.0, 0.5, 24, 12, &trsf)?`.
    pub fn create_full(
        major_rad: f64,
        minor_rad: f64,
        nb_slices: i32,
        nb_stacks: i32,
        trsf: &Transform3,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        Self::create(
            major_rad,
            minor_rad,
            0.0,
            PI * 2.0,
            PI * 2.0,
            nb_slices,
            nb_stacks,
            trsf,
        )
    }
    /// `Create` of a pipe segment; `Prs3dToolTorus::create_with_angle(2.0, 0.5, PI, 24, 12, &trsf)?`.
    pub fn create_with_angle(
        major_rad: f64,
        minor_rad: f64,
        angle: f64,
        nb_slices: i32,
        nb_stacks: i32,
        trsf: &Transform3,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        Self::create(
            major_rad,
            minor_rad,
            0.0,
            PI * 2.0,
            angle,
            nb_slices,
            nb_stacks,
            trsf,
        )
    }
    /// `Create` of a ring segment; `Prs3dToolTorus::create_with_ring(2.0, 0.5, 0.0, PI, 24, 12, &trsf)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn create_with_ring(
        major_rad: f64,
        minor_rad: f64,
        angle1: f64,
        angle2: f64,
        nb_slices: i32,
        nb_stacks: i32,
        trsf: &Transform3,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        Self::create(
            major_rad,
            minor_rad,
            angle1,
            angle2,
            PI * 2.0,
            nb_slices,
            nb_stacks,
            trsf,
        )
    }
}

impl Prs3dToolQuadric for Prs3dToolTorus {
    fn slices_nb(&self) -> i32 {
        self.slices_nb
    }
    fn stacks_nb(&self) -> i32 {
        self.stacks_nb
    }
    fn vertex(&self, u: f64, v: f64) -> Point3 {
        let a_u = u * self.angle;
        let a_v = self.v_min + v * (self.v_max - self.v_min);
        Point3::new(
            (self.major_radius + self.minor_radius * a_v.cos()) * a_u.cos(),
            (self.major_radius + self.minor_radius * a_v.cos()) * a_u.sin(),
            self.minor_radius * a_v.sin(),
        )
    }
    fn normal(&self, u: f64, v: f64) -> Result<Direction3, Failure> {
        let a_u = u * self.angle;
        let a_v = self.v_min + v * (self.v_max - self.v_min);
        Direction3::new(a_u.cos() * a_v.cos(), a_u.sin() * a_v.cos(), a_v.sin())
    }
}

/// A quarter disk sector in the XY plane, facing -Z.
///
/// Example: `Prs3dToolSector::create(1.0, 12, 1, &trsf)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prs3dToolSector {
    radius: f64,
    slices_nb: i32,
    stacks_nb: i32,
}

impl Prs3dToolSector {
    /// A sector tool; `Prs3dToolSector::new(1.0, 12, 1)`.
    pub fn new(radius: f64, nb_slices: i32, nb_stacks: i32) -> Self {
        Self {
            radius,
            slices_nb: nb_slices,
            stacks_nb: nb_stacks,
        }
    }
    /// `Create`: the triangles of a sector; `Prs3dToolSector::create(1.0, 12, 1, &trsf)?`.
    pub fn create(
        radius: f64,
        nb_slices: i32,
        nb_stacks: i32,
        trsf: &Transform3,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        create_with(&Self::new(radius, nb_slices, nb_stacks), trsf)
    }
}

impl Prs3dToolQuadric for Prs3dToolSector {
    fn slices_nb(&self) -> i32 {
        self.slices_nb
    }
    fn stacks_nb(&self) -> i32 {
        self.stacks_nb
    }
    fn vertex(&self, u: f64, v: f64) -> Point3 {
        let angle = u * PI / 2.0;
        let radius = self.radius * v;
        Point3::new(angle.cos() * radius, angle.sin() * radius, 0.0)
    }
    fn normal(&self, _u: f64, _v: f64) -> Result<Direction3, Failure> {
        Ok(Direction3::from(StandardDirection3::NegZ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_and_grid_layout() {
        let cylinder = Prs3dToolCylinder::new(1.0, 1.0, 2.0, 4, 2);
        assert_eq!(cylinder.triangles_nb(), 16);
        assert_eq!(cylinder.vertices_nb(true), 15);
        assert_eq!(cylinder.vertices_nb(false), 48);
        let array = cylinder
            .create_triangulation(&Transform3::default())
            .unwrap();
        assert_eq!(array.vertex_number(), 15);
        assert_eq!(array.edge_number(), 48);
        let poly = cylinder
            .create_poly_triangulation(&Transform3::default())
            .unwrap();
        assert_eq!((poly.nb_nodes(), poly.nb_triangles()), (15, 16));
    }

    #[test]
    fn null_normals_raise() {
        let flat = Prs3dToolCylinder::new(1.0, 1.0, 0.0, 4, 1);
        assert!(flat.create_triangulation(&Transform3::default()).is_err());
        let disk = Prs3dToolDisk::create(0.0, 1.0, 4, 1, &Transform3::default()).unwrap();
        assert_eq!(disk.vertex_number(), 10);
    }

    #[test]
    fn sphere_torus_and_sector_vertices() {
        let sphere = Prs3dToolSphere::new(2.0, 4, 2);
        assert_eq!(sphere.vertex(0.0, 0.0), Point3::new(0.0, -0.0, 2.0));
        let torus = Prs3dToolTorus::full(2.0, 0.5, 4, 4);
        assert_eq!(torus.vertex(0.0, 0.0), Point3::new(2.5, 0.0, 0.0));
        let sector = Prs3dToolSector::new(1.0, 2, 1);
        assert_eq!(sector.vertex(0.0, 1.0), Point3::new(1.0, 0.0, 0.0));
        assert!(
            Prs3dToolTorus::create_with_angle(2.0, 0.5, PI, 4, 4, &Transform3::default()).is_ok()
        );
    }
}
