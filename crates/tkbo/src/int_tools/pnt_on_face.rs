// IntTools_PntOnFace.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! A point on a face; initialization preserves its explicit validity flag.
use rustycad_tkbrep::topods::Face;
use rustycad_tkmath::gp::Point3;
/// `IntTools_PntOnFace`; `PntOnFace::default().valid() == false`.
/// The native header's `IsValid()` has no linked definition; only `Valid()` is executable.
#[derive(Clone, Debug)]
pub struct PntOnFace {
    valid: bool,
    point: Point3,
    u: f64,
    v: f64,
    face: Face,
}
impl Default for PntOnFace {
    fn default() -> Self {
        Self {
            valid: false,
            point: Point3::default(),
            u: 99.0,
            v: 99.0,
            face: Face::default(),
        }
    }
}
impl PntOnFace {
    /// Replaces face, point, and parameters without changing validity; `point.init(&face, p, 1.0, 2.0)`.
    pub fn init(&mut self, face: &Face, point: Point3, u: f64, v: f64) {
        self.face = face.clone();
        self.point = point;
        self.u = u;
        self.v = v;
    }
    /// Replaces the face; `point.set_face(&face)`.
    pub fn set_face(&mut self, face: &Face) {
        self.face = face.clone();
    }
    /// Replaces the spatial point; `point.set_pnt(Point3::default())`.
    pub fn set_pnt(&mut self, point: Point3) {
        self.point = point;
    }
    /// Replaces UV parameters; `point.set_parameters(1.0, 2.0)`.
    pub fn set_parameters(&mut self, u: f64, v: f64) {
        self.u = u;
        self.v = v;
    }
    /// Replaces the validity flag; `point.set_valid(true)`.
    pub fn set_valid(&mut self, valid: bool) {
        self.valid = valid;
    }
    /// Reads the validity flag; `point.valid()`.
    pub fn valid(&self) -> bool {
        self.valid
    }
    /// Borrows the associated face; `point.face()`.
    pub fn face(&self) -> &Face {
        &self.face
    }
    /// Borrows the spatial point; `point.pnt()`.
    pub fn pnt(&self) -> &Point3 {
        &self.point
    }
    /// Reads both UV parameters; `point.parameters()`.
    pub fn parameters(&self) -> (f64, f64) {
        (self.u, self.v)
    }
}
