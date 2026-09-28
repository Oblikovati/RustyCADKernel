//! ElCLib, ElSLib and gp conic/quadric/axis oracle counterpart; prints the same token stream as
//! tools/oracle/tkmath_elementary_probe.cxx.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Axis3, Axis22d, Circle3, Direction2, Direction3, Line3, Point2, Point3,
    Vector2, Vector3, Xy, Xyz,
};
mod conics2;
mod conics3;
mod curves;
mod quadrics;
mod surfaces;
pub use conics2::gpconic2_case;
pub use conics3::gpconic3_case;
pub use curves::{elclib2_case, elclib3_case, elperiod_case, elto3d_case};
pub use quadrics::{gpaxis_case, gpquadric_case};
pub use surfaces::elslib_case;

/// Sequential reader over one request's numeric fields.
pub struct Fields<'v> {
    values: &'v [f64],
    next: usize,
}
impl<'v> Fields<'v> {
    pub fn new(values: &'v [f64]) -> Self {
        Self { values, next: 0 }
    }
    pub fn real(&mut self) -> f64 {
        self.next += 1;
        self.values[self.next - 1]
    }
    pub fn int(&mut self) -> i32 {
        self.real() as i32
    }
    pub fn point(&mut self) -> Point3 {
        Point3::new(self.real(), self.real(), self.real())
    }
    pub fn xyz(&mut self) -> Xyz {
        Xyz::new(self.real(), self.real(), self.real())
    }
    pub fn point2(&mut self) -> Point2 {
        Point2::new(self.real(), self.real())
    }
    pub fn xy(&mut self) -> Xy {
        Xy::new(self.real(), self.real())
    }
}

/// Runs a fallible block, printing the exception type where the native probe catches one.
pub fn attempt(block: impl FnOnce() -> Result<(), Failure>) {
    if let Err(error) = block() {
        fail(&error);
    }
}
pub fn fail(error: &Failure) {
    print!("{} ", error.exception_type());
}
pub fn end() {
    println!();
}
pub fn real(value: f64) {
    print!("{} ", value.to_bits());
}
pub fn reals(values: &[f64]) {
    values.iter().copied().for_each(real);
}
pub fn flag(value: bool) {
    print!("{} ", i32::from(value));
}
pub fn text(value: &str) {
    print!("{value} ");
}
pub fn xyz(value: Xyz) {
    reals(value.coordinates());
}
pub fn pnt(value: Point3) {
    xyz(value.xyz());
}
pub fn vec(value: Vector3) {
    xyz(*value.xyz());
}
pub fn vecs(values: &[Vector3]) {
    values.iter().copied().for_each(vec);
}
pub fn vecs2(values: &[Vector2]) {
    values.iter().copied().for_each(vec2);
}
pub fn dir(value: Direction3) {
    xyz(Xyz::from(value));
}
pub fn ax1(value: Axis1) {
    pnt(value.location());
    dir(value.direction());
}
pub fn ax2(value: Axis2) {
    ax3(value.axis3());
}
pub fn ax3(value: Axis3) {
    pnt(value.location());
    dir(value.direction());
    dir(value.x_direction());
    dir(value.y_direction());
}
pub fn pnt2(value: Point2) {
    reals(value.xy().coordinates());
}
pub fn vec2(value: Vector2) {
    reals(value.xy().coordinates());
}
pub fn dir2(value: Direction2) {
    reals(value.xy().coordinates());
}
pub fn ax2d(value: Axis2d) {
    pnt2(value.location());
    dir2(value.direction());
}
pub fn ax22d(value: Axis22d) {
    pnt2(value.location());
    dir2(value.x_direction());
    dir2(value.y_direction());
}
pub fn lin(value: Line3) {
    ax1(value.position());
}
pub fn circ(value: Circle3) {
    ax2(value.position());
    real(value.radius());
}
pub fn frame3(fields: &mut Fields) -> (Point3, Xyz, Xyz) {
    (fields.point(), fields.xyz(), fields.xyz())
}
pub fn direction(xyz: Xyz) -> Result<Direction3, Failure> {
    Direction3::from_xyz(xyz)
}
