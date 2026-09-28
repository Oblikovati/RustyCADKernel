//! Reader extensions and printers shared by the TKGeomBase probe; the counterpart of
//! tools/oracle/tkgeombase_probe_common.cxx.
pub use super::print::{Reader, attempt, end, flag, int, real, show};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Axis3, Axis22d, Circle2, Circle3, Cone3, Cylinder3, Direction2,
    Direction3, Ellipse2, Ellipse3, Hyperbola2, Hyperbola3, Line2, Line3, Parabola2, Parabola3,
    Plane3, Point2, Point3, Transform2, Transform3, Vector2, Vector3, Xy, Xyz,
};

pub fn xyz(value: Xyz) {
    real(value.x());
    real(value.y());
    real(value.z());
}
pub fn pnt(value: Point3) {
    xyz(value.xyz());
}
pub fn vec(value: Vector3) {
    xyz(*value.xyz());
}
pub fn dir(value: Direction3) {
    xyz(value.xyz());
}
pub fn ax1(value: Axis1) {
    pnt(value.location());
    dir(value.direction());
}
pub fn ax2(value: Axis2) {
    pnt(value.location());
    dir(value.direction());
    dir(value.x_direction());
    dir(value.y_direction());
}
pub fn ax3(value: Axis3) {
    pnt(value.location());
    dir(value.direction());
    dir(value.x_direction());
    dir(value.y_direction());
}
pub fn xy(value: Xy) {
    real(value.x());
    real(value.y());
}
pub fn pnt2d(value: Point2) {
    xy(value.xy());
}
pub fn vec2d(value: Vector2) {
    xy(value.xy());
}
pub fn dir2d(value: Direction2) {
    xy(value.xy());
}
pub fn ax2d(value: Axis2d) {
    pnt2d(value.location());
    dir2d(value.direction());
}
pub fn ax22d(value: Axis22d) {
    pnt2d(value.location());
    dir2d(value.x_direction());
    dir2d(value.y_direction());
}

/// The GbReader members of tools/oracle/tkgeombase_probe_common.cxx.
impl Reader<'_> {
    pub fn xyz(&mut self) -> Xyz {
        let (x, y) = (self.real(), self.real());
        Xyz::new(x, y, self.real())
    }
    pub fn pnt(&mut self) -> Point3 {
        let (x, y) = (self.real(), self.real());
        Point3::new(x, y, self.real())
    }
    pub fn vec(&mut self) -> Vector3 {
        let (x, y) = (self.real(), self.real());
        Vector3::new(x, y, self.real())
    }
    pub fn dir(&mut self) -> Direction3 {
        let (x, y) = (self.real(), self.real());
        Direction3::new(x, y, self.real()).expect("probe direction")
    }
    pub fn ax1(&mut self) -> Axis1 {
        let p = self.pnt();
        Axis1::new(p, self.dir())
    }
    pub fn ax2(&mut self) -> Axis2 {
        let (p, n) = (self.pnt(), self.dir());
        Axis2::with_x_direction(p, n, self.dir()).expect("probe frame")
    }
    pub fn ax3(&mut self) -> Axis3 {
        let (p, n) = (self.pnt(), self.dir());
        Axis3::new(p, n, self.dir()).expect("probe frame")
    }
    pub fn xy(&mut self) -> Xy {
        let x = self.real();
        Xy::new(x, self.real())
    }
    pub fn pnt2d(&mut self) -> Point2 {
        let x = self.real();
        Point2::new(x, self.real())
    }
    pub fn vec2d(&mut self) -> Vector2 {
        let x = self.real();
        Vector2::new(x, self.real())
    }
    pub fn dir2d(&mut self) -> Direction2 {
        let x = self.real();
        Direction2::new(x, self.real()).expect("probe direction")
    }
    pub fn ax2d(&mut self) -> Axis2d {
        let p = self.pnt2d();
        Axis2d::new(p, self.dir2d())
    }
    pub fn ax22d(&mut self) -> Axis22d {
        let (p, x) = (self.pnt2d(), self.dir2d());
        Axis22d::with_directions(p, x, self.dir2d()).expect("probe frame")
    }
}

/// The gp results of the construction algorithms (the gb_lin ... gb_trsf2d printers).
pub fn lin(value: Line3) {
    ax1(value.position());
}
pub fn lin2d(value: Line2) {
    ax2d(value.position());
}
pub fn circ(value: Circle3) {
    ax2(value.position());
    real(value.radius());
}
pub fn circ2d(value: Circle2) {
    ax22d(value.position());
    real(value.radius());
}
pub fn elips(value: Ellipse3) {
    ax2(value.position());
    real(value.major_radius());
    real(value.minor_radius());
}
pub fn elips2d(value: Ellipse2) {
    ax22d(value.axis());
    real(value.major_radius());
    real(value.minor_radius());
}
pub fn hypr(value: Hyperbola3) {
    ax2(value.position());
    real(value.major_radius());
    real(value.minor_radius());
}
pub fn hypr2d(value: Hyperbola2) {
    ax22d(value.axis());
    real(value.major_radius());
    real(value.minor_radius());
}
pub fn parab(value: Parabola3) {
    ax2(value.position());
    real(value.focal());
}
pub fn parab2d(value: Parabola2) {
    ax22d(value.axis());
    real(value.focal());
}
pub fn pln(value: Plane3) {
    ax3(value.position());
}
pub fn cone(value: Cone3) {
    ax3(value.position());
    real(value.semi_angle());
    real(value.ref_radius());
}
pub fn cylinder(value: Cylinder3) {
    ax3(value.position());
    real(value.radius());
}
pub fn trsf(value: Transform3) {
    int(value.form() as i32);
    real(value.scale_factor());
    for i in 1..=3 {
        for j in 1..=3 {
            real(value.matrix().value(i, j).unwrap());
        }
    }
    xyz(value.translation_part());
}
pub fn trsf2d(value: Transform2) {
    int(value.form() as i32);
    real(value.scale_factor());
    for i in 1..=2 {
        for j in 1..=2 {
            real(value.matrix().value(i, j).unwrap());
        }
    }
    xy(value.translation_part());
}
